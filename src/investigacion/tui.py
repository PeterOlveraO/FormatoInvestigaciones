"""Menu de texto basico para generar investigaciones sin escribir el comando.

Solo recolecta los valores y arma los argumentos que recibe ``cli.main``: toda
la logica (respaldos del .env, validacion, avisos, copias) vive alli.
"""

from __future__ import annotations

import textwrap
from typing import Callable, NamedTuple

from .cli import main as cli_main


class Field(NamedTuple):
    key: str
    label: str
    option: str | None  # opcion del CLI; None es el argumento posicional
    help: str
    example: str
    empty: str  # que ocurre si se deja vacio (solo campos opcionales)


FIELDS: list[Field] = [
    Field(
        "markdown", "Archivo Markdown", None,
        "Nombre o ruta del .md con el contenido. Basta el nombre: se busca dentro de input/.",
        "Actividad1.md", "",
    ),
    Field("titulo", "Titulo del trabajo", "--titulo", "Titulo que aparece en la portada.",
          "Introduccion a las bases de datos", ""),
    Field("materia", "Materia", "--materia", "Nombre de la materia.", "Nombre de la materia", ""),
    Field("docente", "Docente", "--docente", "Nombre del docente.",
          "Nombre del docente", "se usa DOCENTE del .env; si no esta, se omite"),
    Field("integrantes", "Integrantes", "--integrantes",
          "Nombres del equipo separados por comas. Si hay integrantes, la portada no muestra al alumno.",
          "Ana Ruiz, Luis Paz", "se usa INTEGRANTES del .env; si no esta, se muestra el alumno"),
    Field("grupo", "Grupo", "--grupo", "Grupo de la materia.", "7-A",
          "se usa GRUPO del .env; si no esta, se omite"),
    Field("salida", "Directorio de salida", "--salida", "Carpeta donde se guarda el PDF.",
          "output", "output"),
    Field("copia", "Copias del PDF", "--copia",
          "Directorios extra donde guardar una copia; varios separados por ;.",
          "copias/uno; copias/dos", "no se hacen copias"),
    Field("env_file", "Archivo .env", "--env-file",
          "Archivo con los datos fijos (universidad, facultad, semestre...).", ".env", ".env"),
    Field("permitir_latex", "Permitir LaTeX", "--permitir-latex",
          "Si esta en si, los comandos LaTeX escritos en el Markdown se interpretan. Se alterna al elegirlo.",
          "si / no", "no"),
    Field("plantilla", "Plantilla LaTeX", "--plantilla",
          "Ruta de una plantilla alternativa.", "mi-plantilla.ltx", "se usa la plantilla incluida"),
    Field("logos", "Directorio de logos", "--logos",
          "Carpeta con logo-universidad.png y logo-facultad.png.", "logos",
          "se usa LOGOS del .env o la carpeta logos/ de la plantilla"),
]
REQUIRED = ("markdown", "titulo", "materia")
VALID_KEYS = f"1-{len(FIELDS)}, g o s"

WIDTH = 74  # ancho interior de las cajas; solo ASCII para que funcione en cualquier consola


def line(char: str = "-") -> str:
    return "+" + char * WIDTH + "+"


def row(text: str = "", center: bool = False) -> str:
    text = " " + text if not center else text
    return "|" + (text.center(WIDTH) if center else text.ljust(WIDTH)) + "|"


def box(title: str, rows: list[str]) -> list[str]:
    return [line("="), row(title, center=True), line("="), *[row(r) for r in rows], line()]


HEADER = box(
    "GENERADOR DE INVESTIGACIONES  (Markdown -> PDF)",
    [
        "Paso 1: llena los campos obligatorios (*).",
        "Paso 2: opcional: ajusta los demas campos.",
        "Paso 3: escribe g para generar el PDF.",
    ],
)


def build_argv(values: dict[str, str]) -> list[str]:
    """Convierte el formulario en argumentos del CLI, omitiendo lo vacio."""

    def get(key: str) -> str:
        return values.get(key, "").strip()

    argv: list[str] = [get("markdown")]
    for field in FIELDS:
        if field.option is None:
            continue
        if field.key == "permitir_latex":
            if get(field.key) == "si":
                argv.append(field.option)
        elif field.key == "copia":
            for path in get(field.key).split(";"):
                if path.strip():
                    argv += [field.option, path.strip()]
        elif get(field.key):
            argv += [field.option, get(field.key)]
    return argv


def missing_fields(values: dict[str, str]) -> list[Field]:
    return [f for f in FIELDS if f.key in REQUIRED and not values.get(f.key, "").strip()]


def show_menu(values: dict[str, str], print_fn: Callable[..., None]) -> None:
    print_fn("")
    for text in HEADER:
        print_fn(text)
    for title, required in (("PASO 1 - OBLIGATORIOS", True), ("PASO 2 - OPCIONALES", False)):
        print_fn(f"\n[ {title} ]")
        print_fn(line())
        for number, field in enumerate(FIELDS, start=1):
            if (field.key in REQUIRED) != required:
                continue
            value = values.get(field.key, "").strip()
            if not value and field.key == "permitir_latex":
                value = "no"
            if value:
                shown = value
            elif required:
                shown = "[FALTA]"
            else:
                shown = f"(vacio: {field.empty})"
            label = f"{number:2}) {field.label}{' *' if required else ''}"
            lines = textwrap.wrap(shown, WIDTH - 32) or [""]
            print_fn(row(label.ljust(30) + lines[0]))
            for extra in lines[1:]:
                print_fn(row(" " * 30 + extra))
        print_fn(line())
    print_fn("\n[ PASO 3 ]   g = Generar PDF     s = Salir")
    print_fn(line("."))
    missing = missing_fields(values)
    if missing:
        print_fn("Falta: " + ", ".join(f.label for f in missing) + ". Elige su numero.")
    else:
        print_fn("Listo. Escribe g para generar.")


def edit_field(field: Field, values: dict[str, str], input_fn: Callable[[str], str],
               print_fn: Callable[..., None]) -> None:
    if field.key == "permitir_latex":
        values[field.key] = "no" if values.get(field.key) == "si" else "si"
        print_fn(f"{field.label}: {values[field.key]}")
        return
    print_fn("\n" + line("-"))
    print_fn(row(field.label.upper()))
    for text in (field.help, f"Ejemplo: {field.example}",
                 f"Valor actual: {values.get(field.key, '') or '(vacio)'}"):
        for wrapped in textwrap.wrap(text, WIDTH - 2):
            print_fn(row(wrapped))
    print_fn(line("-"))
    values[field.key] = input_fn("Nuevo valor (Enter vacio = borrar): ").strip()


def run(input_fn: Callable[[str], str] = input, print_fn: Callable[..., None] = print) -> int:
    values: dict[str, str] = {}
    while True:
        show_menu(values, print_fn)
        choice = input_fn("> ").strip().lower()

        if choice == "s":
            return 0
        if choice == "g":
            missing = missing_fields(values)
            if missing:
                print_fn("No se puede generar. Falta: " + ", ".join(f.label for f in missing))
                continue
            print_fn("\n>>> Generando el PDF, puede tardar unos segundos...")
            code = cli_main(build_argv(values))
            print_fn("\n[ OK ] Termino correctamente." if code == 0 else f"\n[ ERROR ] Termino con error (codigo {code}).")
            input_fn("Enter para volver al menu")
        elif choice.isdigit() and 1 <= int(choice) <= len(FIELDS):
            edit_field(FIELDS[int(choice) - 1], values, input_fn, print_fn)
        else:
            print_fn(f"Opcion no valida. Escribe {VALID_KEYS}.")


def main() -> int:
    try:
        return run()
    except (EOFError, KeyboardInterrupt):
        print()
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
