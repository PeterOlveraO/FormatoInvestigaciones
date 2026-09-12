"""Core del generador de PDF."""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
import unicodedata
from collections.abc import Callable, Iterable
from dataclasses import dataclass
from datetime import date
from pathlib import Path


REQUIRED_HEADINGS = ("introduccion", "desarrollo", "conclusion", "referencias")
MONTHS = (
    "Enero",
    "Febrero",
    "Marzo",
    "Abril",
    "Mayo",
    "Junio",
    "Julio",
    "Agosto",
    "Septiembre",
    "Octubre",
    "Noviembre",
    "Diciembre",
)


class GenerationError(RuntimeError):
    """Error esperado que puede mostrarse directamente a quien usa el CLI."""


@dataclass(frozen=True)
class DocumentData:
    """Valores que se insertan en la portada y en la plantilla.

    `alumno`, `integrantes` y `grupo` son opcionales: la portada omite por
    completo la linea correspondiente cuando van vacios. Los integrantes se
    guardan como tupla y no como lista porque el dataclass es `frozen`: una
    lista lo dejaria sin hash y romperia esa congelacion.
    """

    universidad: str
    facultad: str
    alumno: str
    semestre: str
    titulo: str
    materia: str
    docente: str
    fecha: str
    integrantes: tuple[str, ...] = ()
    grupo: str = ""


def decode_text(raw: bytes, path: Path) -> str:
    """Decodifica archivos de texto comunes en proyectos creados en Windows."""

    try:
        return raw.decode("utf-8-sig")
    except UnicodeDecodeError:
        try:
            return raw.decode("cp1252")
        except UnicodeDecodeError as error:
            raise GenerationError(
                f"No se pudo leer {path}: usa codificacion UTF-8 o Windows-1252."
            ) from error


def decode_process_output(raw: bytes) -> str:
    """Decodifica salidas de herramientas que pueden seguir la configuracion regional."""

    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return raw.decode("cp1252", errors="replace")


def decode_latex_log(raw: bytes) -> str:
    """Decodifica el log de pdflatex, que mezcla dos codificaciones en el mismo archivo.

    Los mensajes van en UTF-8, pero las lineas de division silabica y de cajas
    desbordadas salen en la codificacion interna de la fuente (T1), donde `i`
    es el byte 0xED. Decodificar el archivo entero como UTF-8 falla por esas
    lineas, y caer a cp1252 para todo el log convierte los avisos en jeroglificos
    («â””» en vez de «└»). Por eso cada linea se decodifica por separado.
    """

    return "\n".join(
        line.decode("utf-8") if _is_utf8(line) else line.decode("cp1252", errors="replace")
        for line in raw.split(b"\n")
    )


def _is_utf8(line: bytes) -> bool:
    try:
        line.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return True


def load_dotenv(path: Path) -> dict[str, str]:
    """Carga un archivo dotenv sencillo sin sobrescribir el entorno actual."""

    if not path.exists():
        return {}
    if not path.is_file():
        raise GenerationError(f"El archivo de entorno no es un archivo: {path}")

    loaded: dict[str, str] = {}
    text = decode_text(path.read_bytes(), path)
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[7:].lstrip()
        if "=" not in line:
            raise GenerationError(
                f"Sintaxis invalida en {path}, linea {line_number}; se esperaba CLAVE=VALOR"
            )
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key):
            raise GenerationError(f"Nombre de variable invalido en {path}, linea {line_number}: {key}")
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
            value = value[1:-1]
        elif " #" in value:
            value = value.split(" #", 1)[0].rstrip()
        loaded[key] = value
        os.environ.setdefault(key, value)
    return loaded


# Espacios que se ven como un espacio normal pero no lo son: duro, de figura,
# estrecho y de ancho cero. Word, Notion y las IA los sueltan con frecuencia.
DISGUISED_SPACES = "   ​"
# Lo que se recorta al final de cada linea cuando el documento viene sucio:
# los disfrazados, el tabulador y tambien el espacio normal.
INVISIBLE_SPACES = " \t" + DISGUISED_SPACES
# Marcador de Markdown (encabezado, vineta, lista numerada o cita) separado
# del texto por espacios invisibles en vez de por uno normal.
MARKER_SEPARATOR = re.compile(r"^(\s{0,3})(#{1,6}|[-*+]|\d+[.)]|>)[    ​	]+")


def has_disguised_spaces(markdown: str) -> bool:
    """Indica si el documento trae el espacio invisible que hay que recortar.

    La senal es que alguna linea termine en un espacio disfrazado, lo que
    incluye a la que parece vacia pero solo lleva uno. En ese documento el
    espacio final sobra siempre y hay que quitarlo todo; en los demas, los dos
    espacios finales son un salto de linea deliberado y se respetan.
    """

    for line in markdown.splitlines():
        visible = line.rstrip(" \t")
        if visible and visible[-1] in DISGUISED_SPACES:
            return True
    return False


def normalize_markdown(markdown: str) -> str:
    """Quita el espacio invisible que impide a Pandoc ver la estructura.

    Muchos documentos exportados de otras herramientas traen lineas que parecen
    vacias pero llevan un espacio duro (U+00A0), y dos espacios al final de cada
    renglon. Sin lineas realmente vacias Pandoc no separa los parrafos, y
    entonces los encabezados y las tablas se convierten en texto corriente: el
    PDF sale con los `###` a la vista y las tablas como filas de barras.

    Ademas, un marcador seguido de espacio duro (`### Titulo`, `- item`)
    tampoco lo reconoce Pandoc, porque Markdown exige un espacio normal despues
    del marcador; ese separador se sustituye por un espacio corriente.

    Por lo demas solo se recorta el final de cada linea, asi que un espacio duro
    dentro del texto se conserva. Los bloques delimitados por ``` o ~~~ se dejan
    intactos, porque ahi el espacio final puede ser parte del codigo.

    Los dos espacios finales son a la vez basura y sintaxis: en Markdown
    significan «salto de linea». Por eso solo se recortan cuando el documento
    trae de verdad espacios disfrazados; si esta limpio se respetan y el salto
    llega al PDF.
    """

    trailing = INVISIBLE_SPACES if has_disguised_spaces(markdown) else DISGUISED_SPACES + "\t"
    lines: list[str] = []
    fence: str | None = None
    for line in markdown.splitlines():
        fence_match = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if fence_match:
            marker = fence_match.group(1)
            if fence is None:
                fence = marker[0]
            elif marker[0] == fence:
                fence = None
            lines.append(line)
            continue
        if fence is not None:
            lines.append(line)
            continue
        line = MARKER_SEPARATOR.sub(r"\1\2 ", line.rstrip(trailing))
        lines.append(line)
    normalized = "\n".join(lines)
    return normalized + "\n" if markdown.endswith("\n") else normalized


def read_markdown(path: Path) -> str:
    """Lee Markdown UTF-8, acepta Windows-1252 y normaliza el espacio invisible."""

    return normalize_markdown(decode_text(path.read_bytes(), path))


# Carpeta donde se guardan los trabajos en Markdown. Escribir la ruta completa en
# cada comando es tedioso, asi que el archivo se busca tambien aqui.
INPUT_DIRECTORY = "input"


def input_directory() -> Path:
    """Carpeta `input/` del proyecto, exista o no."""

    return project_root() / INPUT_DIRECTORY


def find_markdown(path: Path) -> Path:
    """Busca el Markdown tal cual, dentro de `input/` y por nombre bajo `input/`.

    Los tres intentos, en orden, para que `investigacion Actividad1.md` funcione
    sin escribir la carpeta y aunque el archivo este en una subcarpeta por
    materia (`input/IS/Actividad1.md`). Una ruta que ya existe siempre gana: la
    busqueda por nombre es el ultimo recurso.
    """

    candidate = path.expanduser()
    if candidate.exists():
        return candidate.resolve()

    base = input_directory()
    inside = base / candidate
    if inside.exists():
        return inside.resolve()

    if not base.is_dir() or candidate.name != str(candidate):
        return candidate.resolve()

    matches = sorted(match for match in base.rglob(candidate.name) if match.is_file())
    if len(matches) == 1:
        return matches[0].resolve()
    if len(matches) > 1:
        opciones = ", ".join(str(match.relative_to(base)) for match in matches)
        raise GenerationError(
            f"Hay varios archivos que se llaman {candidate.name} en {base}: {opciones}. "
            "Indica cual con su subcarpeta."
        )
    return candidate.resolve()


def resolve_markdown_path(path: Path) -> Path:
    """Valida la ruta de entrada y devuelve su forma absoluta."""

    resolved = find_markdown(path)
    if not resolved.exists():
        raise GenerationError(f"No existe el archivo Markdown: {resolved}")
    if not resolved.is_file():
        raise GenerationError(f"La ruta indicada no es un archivo: {resolved}")
    if resolved.suffix.casefold() != ".md":
        raise GenerationError("El archivo de entrada debe tener extension .md.")
    return resolved


def latex_escape(value: str) -> str:
    """Escapa texto controlado por el usuario antes de insertarlo en LaTeX."""

    replacements = {
        "\\": r"\textbackslash{}",
        "&": r"\&",
        "%": r"\%",
        "$": r"\$",
        "#": r"\#",
        "_": r"\_",
        "{": r"\{",
        "}": r"\}",
        "~": r"\textasciitilde{}",
        "^": r"\textasciicircum{}",
    }
    return "".join(replacements.get(character, character) for character in value)


def parse_integrantes(value: str) -> tuple[str, ...]:
    """Separa los nombres del equipo escritos en un solo argumento.

    Acepta comas y punto y coma como separadores, recorta los espacios y
    descarta los vacios, de modo que "Ana,,Luis," devuelve dos nombres.
    """

    return tuple(name.strip() for name in re.split(r"[,;]", value) if name.strip())


def format_delivery_date(today: date | None = None) -> str:
    """Devuelve la fecha de ejecucion con mes en espanol y formato largo."""

    today = today or date.today()
    return f"{MONTHS[today.month - 1]} {today.day}, {today.year}"


def normalize_heading(heading: str) -> str:
    """Normaliza encabezados para validarlos sin depender de mayusculas o acentos."""

    normalized = unicodedata.normalize("NFKD", heading)
    normalized = "".join(char for char in normalized if not unicodedata.combining(char))
    normalized = re.sub(r"\s+#+\s*$", "", normalized)
    normalized = re.sub(r"^\s*(?:\d+(?:\.\d+)*[.)]?\s*)", "", normalized)
    return re.sub(r"\s+", " ", normalized).strip().casefold()


def markdown_headings(markdown: str) -> list[str]:
    """Extrae encabezados ATX del Markdown, ignorando los bloques de codigo."""

    headings: list[str] = []
    fence: str | None = None
    for line in markdown.splitlines():
        fence_match = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if fence_match:
            marker = fence_match.group(1)
            if fence is None:
                fence = marker[0]
                continue
            if marker[0] == fence:
                fence = None
            continue
        if fence is not None:
            continue
        match = re.match(r"^\s{0,3}#{1,6}\s+(.+?)\s*$", line)
        if match:
            headings.append(normalize_heading(match.group(1)))
    return headings


def validate_markdown(markdown: str) -> list[str]:
    """Devuelve advertencias de estructura sin impedir documentos validos con otra estructura."""

    headings = markdown_headings(markdown)
    warnings: list[str] = []
    found: list[tuple[int, str]] = []
    for required in REQUIRED_HEADINGS:
        if required not in headings:
            warnings.append(f"No se encontro el encabezado recomendado: {required.title()}.")
        else:
            found.append((headings.index(required), required))
    if len(found) > 1 and found != sorted(found):
        esperado = ", ".join(name.title() for _, name in found)
        warnings.append(
            f"Los encabezados recomendados no estan en el orden esperado: {esperado}."
        )
    return warnings


def find_template(explicit_path: Path | None = None) -> Path:
    """Resuelve la plantilla del proyecto o una ruta indicada por la persona usuaria."""

    if explicit_path is not None:
        template = explicit_path.expanduser().resolve()
        if not template.is_file():
            raise GenerationError(f"No existe la plantilla LaTeX: {template}")
        return template

    candidates = (
        Path.cwd() / "Latex" / "base.ltx",
        Path(__file__).resolve().parents[2] / "Latex" / "base.ltx",
    )
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    raise GenerationError(
        "No se encontro Latex/base.ltx. Ejecuta el comando desde el proyecto o usa --plantilla."
    )


# Carpeta del proyecto donde se guardan las imagenes descargadas de la web y los
# diagramas dibujados con Graphviz. Es una cache: se puede borrar entera y lo
# unico que pasa es que la proxima generacion vuelva a bajarlos o dibujarlos.
MEDIA_DIRECTORY = "imagenes"
REMOTE_MEDIA = "remotas"
DIAGRAM_MEDIA = "diagramas"


def project_root() -> Path:
    """Raiz del proyecto, con el mismo criterio que find_template()."""

    for candidate in (Path.cwd(), Path(__file__).resolve().parents[2]):
        if (candidate / "Latex" / "base.ltx").is_file():
            return candidate
    return Path.cwd()


def media_directories() -> tuple[Path, Path]:
    """Crea y devuelve las carpetas de imagenes remotas y de diagramas."""

    base = project_root() / MEDIA_DIRECTORY
    remote = base / REMOTE_MEDIA
    diagrams = base / DIAGRAM_MEDIA
    for directory in (remote, diagrams):
        directory.mkdir(parents=True, exist_ok=True)
    return remote, diagrams


# Ninguna pasada de pdflatex sobre un trabajo escolar tarda tanto: si lo hace es
# que se quedo dando vueltas (soul, por ejemplo, se cuelga dentro de una tabla).
# Mas vale un error legible que una terminal congelada para siempre.
TOOL_TIMEOUT_SECONDS = 180
# Extensiones de Pandoc que completan la sintaxis extendida de Markdown. El
# dialecto `markdown` de Pandoc ya trae tablas, notas al pie, listas de
# definicion, listas de tareas, tachado, sub/superindice e identificadores de
# encabezado; estas tres faltaban:
#   mark               ==resaltado==
#   emoji              :joy: y demas codigos de emoji
#   autolink_bare_uris una URL suelta se vuelve enlace sin escribir <>
MARKDOWN_EXTENSIONS = ("mark", "emoji", "autolink_bare_uris")
# Filtros Lua que completan lo que Pandoc no hace por su cuenta. Viajan con el
# paquete, no con la plantilla, porque no dependen de la que se elija con
# --plantilla:
#   html_en_linea  rescata el HTML (<br>, <mark>, <sub>...) que el escritor de
#                  LaTeX descarta sin avisar
#   imagenes       descarga las imagenes de la web a la cache del proyecto y
#                  avisa de las locales que no existen
#   diagramas      dibuja con Graphviz los bloques ```dot
#   graficas       deja pasar a LaTeX los bloques ```pgfplot y ```tikz
#   bloques        cajas de nota (::: nota) y sangria francesa en las referencias
FILTERS_DIRECTORY = Path(__file__).parent / "filtros"
LUA_FILTERS = ("html_en_linea", "imagenes", "diagramas", "graficas", "bloques")
# Prefijo con el que los filtros marcan sus avisos en la salida de error, para
# distinguirlos de los mensajes de Pandoc.
FILTER_WARNING_PREFIX = "[investigacion]"
# Agente con el que se piden las imagenes de la web.
USER_AGENT = "investigacion/0.1 (generador de trabajos academicos; pandoc)"


def filter_warnings(stderr: str) -> list[str]:
    """Separa los avisos de los filtros Lua del resto de la salida de Pandoc."""

    return [
        line[len(FILTER_WARNING_PREFIX):].strip()
        for line in stderr.splitlines()
        if line.startswith(FILTER_WARNING_PREFIX)
    ]


def pandoc_to_latex(
    markdown_path: Path,
    markdown: str,
    allow_raw_latex: bool = False,
    on_warning: Callable[[str], None] | None = None,
) -> str:
    """Convierte Markdown a un fragmento LaTeX usando Pandoc.

    Por omision se desactiva `raw_tex`: sin eso, cualquier contrabarra suelta del
    texto (una ruta como C:\\Users, por ejemplo) se envia a LaTeX como si fuera un
    comando y la compilacion falla. Las formulas con $...$ siguen funcionando.

    `on_warning` recibe los avisos de los filtros Lua (una imagen que no se pudo
    descargar, un diagrama que no se pudo dibujar); como en compile_pdf(), el
    generador no imprime nada por su cuenta.
    """

    remote_media, diagram_media = media_directories()
    # Las imagenes se buscan en la carpeta del Markdown y en la del proyecto, de
    # modo que `![](diagrama.png)` funcione desde cualquier trabajo.
    resources = [markdown_path.parent, remote_media.parent, remote_media, diagram_media]

    source_format = "markdown" if allow_raw_latex else "markdown-raw_tex"
    source_format += "".join(f"+{extension}" for extension in MARKDOWN_EXTENSIONS)
    command = [
        "pandoc",
        "-",
        f"--from={source_format}",
        "--to=latex",
        "--wrap=none",
        f"--resource-path={os.pathsep.join(str(path) for path in resources)}",
        # Algunos sitios rechazan una peticion sin agente; Pandoc no manda ninguno.
        f"--request-header=User-Agent:{USER_AGENT}",
    ]
    for name in LUA_FILTERS:
        filter_path = FILTERS_DIRECTORY / f"{name}.lua"
        if filter_path.is_file():
            command.append(f"--lua-filter={filter_path}")

    environment = os.environ.copy()
    environment["INVESTIGACION_IMAGENES"] = str(remote_media)
    environment["INVESTIGACION_DIAGRAMAS"] = str(diagram_media)
    # Una carpeta por linea: asi la ruta puede llevar cualquier separador.
    environment["INVESTIGACION_RECURSOS"] = "\n".join(str(path) for path in resources)

    try:
        result = subprocess.run(
            command,
            cwd=markdown_path.parent,
            capture_output=True,
            input=markdown.encode("utf-8"),
            check=False,
            timeout=TOOL_TIMEOUT_SECONDS,
            env=environment,
        )
    except FileNotFoundError as error:
        raise GenerationError(
            "No se encontro Pandoc. Instala Pandoc y asegurate de que este disponible en PATH."
        ) from error
    except subprocess.TimeoutExpired as error:
        raise GenerationError(
            f"Pandoc no respondio en {TOOL_TIMEOUT_SECONDS} segundos y se detuvo."
        ) from error

    stderr = decode_process_output(result.stderr)
    if result.returncode != 0:
        detail = stderr.strip() or decode_process_output(result.stdout).strip()
        raise GenerationError(f"Pandoc no pudo convertir el Markdown{': ' + detail if detail else '.'}")
    if on_warning is not None:
        for warning in filter_warnings(stderr):
            on_warning(warning)
    return decode_process_output(result.stdout).strip()


def render_template(template: str, content: str, data: DocumentData) -> str:
    """Reemplaza los marcadores controlados de la plantilla base."""

    # Cada nombre se escapa por separado y solo despues se unen con el \\ de
    # LaTeX: si se escapara la cadena ya unida, latex_escape() convertiria esas
    # contrabarras en \textbackslash{} y los nombres saldrian en un solo renglon.
    integrantes = r"\\".join(latex_escape(nombre) for nombre in data.integrantes)

    replacements = {
        "%%UNIVERSIDAD%%": latex_escape(data.universidad),
        "%%FACULTAD%%": latex_escape(data.facultad),
        "%%TITULO%%": latex_escape(data.titulo),
        "%%ALUMNO%%": latex_escape(data.alumno),
        "%%INTEGRANTES%%": integrantes,
        "%%MATERIA%%": latex_escape(data.materia),
        "%%GRUPO%%": latex_escape(data.grupo),
        "%%DOCENTE%%": latex_escape(data.docente),
        "%%SEMESTRE%%": latex_escape(data.semestre),
        "%%FECHA_ENTREGA%%": latex_escape(data.fecha),
        "%%CONTENIDO_MARKDOWN%%": content,
    }
    content_marker = "%%CONTENIDO_MARKDOWN%%"
    rendered = template
    for marker, replacement in replacements.items():
        if marker == content_marker:
            continue
        rendered = rendered.replace(marker, replacement)

    # Se revisa antes de insertar el contenido: el Markdown puede mencionar un
    # marcador y eso no significa que la plantilla haya quedado incompleta.
    pending = sorted(set(re.findall(r"%%[A-Z_]+%%", rendered)) - {content_marker})
    if pending:
        raise GenerationError(f"La plantilla contiene marcadores sin reemplazar: {', '.join(pending)}")
    if content_marker not in rendered:
        raise GenerationError(f"La plantilla no contiene el marcador {content_marker}.")
    return rendered.replace(content_marker, content)


MAX_LATEX_RUNS = 4


def summarize_latex_errors(log: str) -> str:
    """Extrae los errores reales del log de pdflatex y descarta el ruido de los paquetes."""

    lines = log.splitlines()
    blocks: list[str] = []
    for index, line in enumerate(lines):
        # -file-line-error produce "archivo.tex:123: mensaje"; el resto empieza con "!".
        if not (re.match(r"^.+?\.\w+:\d+: ", line) or line.startswith("! ")):
            continue
        block = [line.strip()]
        for extra in lines[index + 1 : index + 6]:
            stripped = extra.strip()
            if not stripped or re.match(r"^.+?\.\w+:\d+: ", extra) or extra.startswith("! "):
                break
            block.append(stripped)
            if stripped.startswith("l."):
                break
        blocks.append("\n".join(block))
        if len(blocks) == 3:
            break
    return "\n\n".join(blocks)


def keep_failure_artifacts(tex_path: Path, destination: Path) -> Path | None:
    """Conserva el .tex y el .log fuera del directorio temporal para poder revisarlos."""

    saved: Path | None = None
    try:
        destination.mkdir(parents=True, exist_ok=True)
        for suffix in (".tex", ".log"):
            source = tex_path.with_suffix(suffix)
            if source.is_file():
                copy = destination / f"ultimo-error{suffix}"
                shutil.copy2(source, copy)
                if suffix == ".log":
                    saved = copy
    except OSError:
        return None
    return saved


# Nombres que busca la portada. Quien use el programa solo tiene que dejar sus
# dos archivos con estos nombres en la carpeta de logos.
LOGO_FILES = ("logo-universidad.png", "logo-facultad.png")
LOGOS_DIRECTORY = "logos"


def resolve_logos_directory(template_file: Path, explicit_path: Path | None = None) -> Path | None:
    """Decide de donde salen los logos de la portada.

    Con `--logos` (o la variable LOGOS del .env) se puede tener la carpeta fuera
    del proyecto, que es lo habitual: los logos de una institucion no suelen ser
    redistribuibles y por eso `Latex/logos/` no se versiona. Si no se indica
    nada se usa la carpeta `logos/` que este al lado de la plantilla.
    """

    if explicit_path is not None:
        directory = explicit_path.expanduser().resolve()
        if not directory.is_dir():
            raise GenerationError(f"No existe el directorio de logos: {directory}")
        return directory

    default = template_file.parent / LOGOS_DIRECTORY
    return default if default.is_dir() else None


def copy_template_assets(
    template_file: Path, destination: Path, logos_directory: Path | None = None
) -> None:
    """Lleva los logos junto al .tex temporal.

    pdflatex se ejecuta con el directorio del Markdown como cwd, asi que una ruta
    relativa como `Latex/logos/logo-universidad.png` no se encontraria. Los
    archivos se copian al temporal y compile_pdf() lo agrega a TEXINPUTS, de modo
    que la plantilla solo escribe `{logo-universidad}`. Si no hay directorio de
    logos no pasa nada: la portada los envuelve en \\IfFileExists.
    """

    assets = logos_directory if logos_directory is not None else template_file.parent / LOGOS_DIRECTORY
    if assets is None or not assets.is_dir():
        return
    for asset in sorted(assets.iterdir()):
        if asset.is_file():
            shutil.copy2(asset, destination / asset.name)


def latex_search_path(directory: Path) -> str:
    """Arma un TEXINPUTS que busca primero en `directory` sin perder las rutas por omision.

    En TEXINPUTS una entrada vacia (es decir, el separador final) representa la
    ruta de busqueda por defecto de TeX; sin ella pdflatex dejaria de encontrar
    sus propios paquetes.
    """

    previous = os.environ.get("TEXINPUTS", "")
    if previous and not previous.endswith(os.pathsep):
        previous += os.pathsep
    return f"{directory}{os.pathsep}{previous}"


def unsupported_character_warnings(log: str) -> list[str]:
    """Extrae del log los simbolos Unicode que la plantilla no supo componer.

    La plantilla convierte el error de inputenc en un aviso y deja un [?] en el
    PDF; sin leer esos avisos el simbolo perdido pasaria inadvertido.
    """

    found = re.findall(r"Caracter Unicode sin definir:\s*(\S+)\s*\(U\+([0-9A-F]+)\)", log)
    warnings: list[str] = []
    for character, codepoint in dict.fromkeys(found):
        warnings.append(
            f"El simbolo {character} (U+{codepoint}) no se pudo componer y salio como [?] "
            "en el PDF. Reemplazalo en el Markdown o declaralo en Latex/base.ltx."
        )
    return warnings


def compile_pdf(
    tex_path: Path,
    output_pdf: Path,
    working_directory: Path,
    on_warning: Callable[[str], None] | None = None,
) -> None:
    """Compila las veces necesarias para estabilizar el indice y las referencias.

    `on_warning` recibe los avisos de simbolos Unicode sin definir; el generador
    no imprime nada por su cuenta, de eso se encarga el CLI.
    """

    environment = os.environ.copy()
    environment["TEXINPUTS"] = latex_search_path(tex_path.parent)
    command = [
        "pdflatex",
        "-interaction=nonstopmode",
        "-halt-on-error",
        "-file-line-error",
        f"-output-directory={tex_path.parent}",
        str(tex_path),
    ]
    log_path = tex_path.with_suffix(".log")
    toc_path = tex_path.with_suffix(".toc")
    previous_toc: str | None = None

    for run in range(1, MAX_LATEX_RUNS + 1):
        try:
            result = subprocess.run(
                command,
                cwd=working_directory,
                capture_output=True,
                check=False,
                env=environment,
                timeout=TOOL_TIMEOUT_SECONDS,
            )
        except FileNotFoundError as error:
            raise GenerationError(
                "No se encontro pdflatex. Instala TeX Live y asegurate de que pdflatex este disponible en PATH."
            ) from error
        except subprocess.TimeoutExpired as error:
            saved_log = keep_failure_artifacts(tex_path, output_pdf.parent)
            extra = f"\n\nLog completo: {saved_log}" if saved_log else ""
            raise GenerationError(
                f"pdflatex se quedo trabajando mas de {TOOL_TIMEOUT_SECONDS} segundos "
                f"y se detuvo; revisa el documento en busca de algo que LaTeX no pueda "
                f"componer.{extra}"
            ) from error

        log = decode_latex_log(log_path.read_bytes()) if log_path.is_file() else ""
        if not log:
            log = decode_process_output(result.stdout)

        if result.returncode != 0:
            detail = summarize_latex_errors(log)
            if not detail:
                detail = (log.strip() or decode_process_output(result.stderr).strip())[-2000:]
            saved_log = keep_failure_artifacts(tex_path, output_pdf.parent)
            extra = f"\n\nLog completo: {saved_log}" if saved_log else ""
            raise GenerationError(f"La compilacion LaTeX fallo:\n{detail}{extra}")

        current_toc = toc_path.read_bytes().decode("utf-8", "replace") if toc_path.is_file() else ""
        needs_rerun = "Rerun to get" in log or "Rerun LaTeX" in log or current_toc != previous_toc
        previous_toc = current_toc
        if run > 1 and not needs_rerun:
            break

    generated_pdf = tex_path.with_suffix(".pdf")
    if not generated_pdf.is_file():
        raise GenerationError("pdflatex termino sin producir el archivo PDF esperado.")
    if on_warning is not None:
        for warning in unsupported_character_warnings(log):
            on_warning(warning)
    shutil.copy2(generated_pdf, output_pdf)


def generate_pdf(
    markdown_path: Path,
    output_directory: Path,
    data: DocumentData,
    template_path: Path | None = None,
    markdown: str | None = None,
    allow_raw_latex: bool = False,
    on_warning: Callable[[str], None] | None = None,
    logos_path: Path | None = None,
) -> Path:
    """Genera y devuelve el PDF final."""

    markdown_path = resolve_markdown_path(markdown_path)
    if markdown is None:
        markdown = read_markdown(markdown_path)
    if not markdown.strip():
        raise GenerationError("El archivo Markdown esta vacio.")

    template_file = find_template(template_path)
    logos_directory = resolve_logos_directory(template_file, logos_path)
    template = decode_text(template_file.read_bytes(), template_file)
    content = pandoc_to_latex(markdown_path, markdown, allow_raw_latex, on_warning)
    rendered = render_template(template, content, data)
    output_directory = output_directory.expanduser().resolve()
    output_directory.mkdir(parents=True, exist_ok=True)
    output_pdf = output_directory / f"{slugify(data.titulo)}.pdf"

    with tempfile.TemporaryDirectory(prefix="investigacion-") as temporary_directory:
        tex_path = Path(temporary_directory) / "trabajo.tex"
        tex_path.write_text(rendered, encoding="utf-8")
        copy_template_assets(template_file, tex_path.parent, logos_directory)
        compile_pdf(tex_path, output_pdf, markdown_path.parent, on_warning)
    return output_pdf


def copy_pdf_to(pdf: Path, directories: Iterable[Path]) -> list[Path]:
    """Deja una copia del PDF en cada directorio adicional indicado."""

    copies: list[Path] = []
    seen = {pdf.parent.resolve()}
    for directory in directories:
        destination = directory.expanduser().resolve()
        if destination in seen:
            continue
        seen.add(destination)
        if destination.exists() and not destination.is_dir():
            raise GenerationError(f"La ruta de copia no es un directorio: {destination}")
        destination.mkdir(parents=True, exist_ok=True)
        copy = destination / pdf.name
        shutil.copy2(pdf, copy)
        copies.append(copy)
    return copies


def slugify(value: str) -> str:
    """Genera un nombre de archivo estable y seguro a partir del titulo."""

    normalized = unicodedata.normalize("NFKD", value)
    normalized = "".join(char for char in normalized if not unicodedata.combining(char))
    slug = re.sub(r"[^a-zA-Z0-9]+", "-", normalized).strip("-").lower()
    return slug or "investigacion"
