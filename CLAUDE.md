# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Idioma

El proyecto es de un estudiante hispanohablante. El README, los mensajes del CLI,
los docstrings y los comentarios están en español; los mensajes de error del CLI
se escriben sin acentos. Mantén esa convención al añadir código.

## Comandos

```bash
# Entorno (el proyecto se instala en modo editable)
python -m venv .venv && source .venv/bin/activate && python -m pip install -e .

# Pruebas (usar el python del venv: los tests importan el paquete instalado)
.venv/bin/python -m unittest discover -s tests

# Una sola prueba o una sola clase
.venv/bin/python -m unittest tests.test_generator.CopyTests.test_copies_the_pdf_to_each_directory
.venv/bin/python -m unittest tests.test_generator.ArgumentTests

# Ejecutar el CLI
.venv/bin/investigacion Crudo/Economia.md --titulo "Tema" --materia "Materia" --docente "Docente"
```

No hay linter ni formateador configurados, ni dependencias de terceros: el
paquete es solo biblioteca estándar (ver `AGENTS.md`). Pandoc y pdflatex son
dependencias **del sistema**, no de Python.

## Arquitectura

Tubería de una sola dirección, sin estado intermedio persistente:

```
archivo.md
  → read_markdown()      decodifica UTF-8 con respaldo CP1252
  → normalize_markdown() quita el espacio invisible (lo llama read_markdown)
  → validate_markdown()  solo advertencias, nunca bloquea
  → pandoc_to_latex()    Pandoc emite un FRAGMENTO de LaTeX (sin preámbulo);
                         con las extensiones de MARKDOWN_EXTENSIONS y el filtro
                         Lua de filtros/html_en_linea.lua
  → render_template()    sustituye los marcadores %%NOMBRE%% de Latex/base.ltx
  → compile_pdf()        pdflatex en un directorio temporal, varias pasadas
  → copy_pdf_to()        copias opcionales en otros directorios
```

`src/investigacion/generator.py` contiene toda la lógica y es puro; lanza
`GenerationError` para todo fallo esperado. `src/investigacion/cli.py` solo
traduce argumentos y variables de entorno a un `DocumentData`, e imprime.
`main()` captura `GenerationError`, `OSError` y `UnicodeError` y devuelve 1.

### El contrato crítico: plantilla ↔ salida de Pandoc

Pandoc genera un **fragmento**, no un documento completo, así que
`Latex/base.ltx` tiene que aportar por su cuenta todo el preámbulo que la salida
de Pandoc da por supuesto. Es la fuente de fallos más habitual del proyecto: un
documento con tablas, imágenes o bloques de código falla mientras uno de solo
texto compila bien.

El bloque `PAQUETES QUE NECESITA LA SALIDA DE PANDOC` de `Latex/base.ltx` cubre
hoy tablas (`calc`, `\newcounter{none}`, parche de `longtable`), imágenes
(`\pandocbounded`), código (`Shaded`/`Highlighting` y los comandos `\...Tok`),
tachado (`soul`), matemáticas (`amsmath`) y notas al pie en tablas
(`footnotehyper`). **No quites nada de ese bloque.** Antes de tocarlo, compara
con lo que espera la versión instalada de Pandoc:

```bash
pandoc archivo.md -s --to=latex | sed -n '/documentclass/,/begin{document}/p'
```

Un bloque de código sin lenguaje no llega como `Shaded` sino como `verbatim`,
que no pasa por esa definición y hereda el `\doublespacing` del documento: el
arte ASCII sale estirado y los conectores verticales con hueco. Por eso
`\AtBeginEnvironment{verbatim}{\singlespacing\small}` le da el mismo trato.

Tres avisos sobre las figuras, que ahora pasan por `floatrow`:

- `floatrow` y el paquete `float` **no pueden convivir**: cargar los dos aborta
  la compilación. La colocación `H` (la figura se queda donde se escribió, en
  vez de flotar) la trae el propio `floatrow`.
- Esa colocación se fija en la línea `\@ifundefined{floatsetup}...\fps@figure`,
  que va **donde Pandoc define la suya**; puesta antes, Pandoc la sobrescribe y
  las figuras vuelven a flotar.
- `floatrow` deja las tablas largas pegadas al margen izquierdo, de ahí el
  `\LTleft`/`\LTright` a `\fill` dentro de su bloque.

Otras dos trampas de la plantilla:

- Babel-spanish fija los títulos en `\begin{document}`, así que un
  `\renewcommand{\contentsname}{...}` suelto en el preámbulo no surte efecto;
  hay que envolverlo en `\addto\captionsspanish{...}`.
- `hyperref` y `footnotehyper` van al final del preámbulo, en ese orden.

### Marcadores de la plantilla

`render_template()` sustituye `%%UNIVERSIDAD%%`, `%%FACULTAD%%`, `%%TITULO%%`,
`%%ALUMNO%%`, `%%INTEGRANTES%%`, `%%MATERIA%%`, `%%GRUPO%%`, `%%DOCENTE%%`,
`%%SEMESTRE%%`, `%%FECHA_ENTREGA%%` y `%%CONTENIDO_MARKDOWN%%`. Los datos del usuario pasan por
`latex_escape()`; el contenido convertido por Pandoc **no** se escapa, y se
inserta al final para que un `%%...%%` mencionado dentro del trabajo no se
confunda con un marcador de la plantilla sin resolver. Al añadir un marcador
nuevo hay que tocar `DocumentData`, el diccionario de `render_template()` y
`base.ltx` a la vez.

`%%INTEGRANTES%%` es el único que no sale de un solo `latex_escape()`:
`DocumentData.integrantes` es una `tuple[str, ...]` (tupla y no lista porque el
dataclass es `frozen`) y la lista se arma escapando **cada nombre por separado**
y uniéndolos después con un `\\` literal. Al revés no funciona: `latex_escape()`
convertiría esas contrabarras en `\textbackslash{}` y los nombres saldrían en un
solo renglón.

### Datos opcionales de la portada: alumno, integrantes y grupo

Del comando solo son obligatorios `--titulo` y `--materia`. `--docente` sigue el
mismo patrón que `--grupo` (opción con `default=None`, respaldo en la variable
del `.env`, la opción gana) y su línea se omite cuando queda vacío, así que en la
portada solo `MATERIA` y `SEMESTRE` aparecen siempre.

`ALUMNO` ya no está en `REQUIRED_ENV` (solo quedan `UNIVERSIDAD`, `FACULTAD` y
`SEMESTRE`); `--integrantes` acepta los nombres separados por comas o punto y
coma y `parse_integrantes()` los normaliza. Como respaldo se lee la variable
`INTEGRANTES` del `.env`, pero `--integrantes` manda.

`--grupo` sigue exactamente el mismo patrón (opción con `default=None`, respaldo
en la variable `GRUPO` del `.env`, la opción gana) y su línea cierra el bloque
con el mismo `\ifdefempty`. Es el molde a copiar para el próximo dato opcional
de la portada; `--logos` lo repite con una ruta en vez de un texto.

El orden del bloque de datos es **fijo** —alumno (o integrantes), materia,
docente, semestre, grupo— y vive en la plantilla, no en el CLI: el orden en que
se escriban las opciones del comando no lo altera. La prueba `PortadaTests` lo
comprueba leyendo las posiciones de las etiquetas en `base.ltx`.

Las dos líneas son **excluyentes** en la portada, no acumulativas: si hay
integrantes se lista al equipo y `ALUMNO:` no aparece aunque `ALUMNO` siga
puesto en el `.env` (un trabajo de equipo no repite la firma individual); sin
integrantes se muestra el alumno; y si faltan los dos no aparece ninguna. Eso es
un `\ifdefempty{\ListaIntegrantes}` externo con el caso del alumno anidado en su
rama vacía: la decisión vive en la plantilla, no en Python, así que
`DocumentData` conserva ambos datos y `render_template()` no los filtra.

El `\\[0.25cm]` de separación va **dentro** de cada rama condicional: un `\\`
suelto al principio del nodo de TikZ deja un renglón vacío o da error. Los
integrantes van en un `tabular[t]` para que queden en columna; el
`\raisebox{0pt}[0pt][\depth]{}` que lo envuelve anula solo su altura, porque si
no la fila alta del tabular supera el `\topskip` y baja todo el bloque (ahora
los integrantes son siempre la primera línea cuando existen). Los metadatos del
PDF usan `\AutorPDF`, que sigue el mismo criterio que la portada —integrantes y
si no alumno— para no firmar a alguien que la portada ya no menciona; los `\\`
se traducen a comas vía `\pdfstringdefDisableCommands`.

### La sintaxis de Markdown que se soporta

El criterio es cubrir entera la [Markdown Guide](https://www.markdownguide.org)
(sintaxis básica y extendida). El dialecto `markdown` de Pandoc ya trae casi
todo; lo que falta se añade en tres sitios distintos, y conviene saber cuál toca
antes de tirar del hilo:

1. `MARKDOWN_EXTENSIONS` en `generator.py` — extensiones que Pandoc no activa
   por omisión: `mark` (`==resaltado==`), `emoji` (`:joy:`) y
   `autolink_bare_uris` (una URL suelta se vuelve enlace).
2. Los filtros Lua de `src/investigacion/filtros/`, que se aplican en el orden
   de `LUA_FILTERS`. Van en el paquete y no en `Latex/`, porque no dependen de
   la plantilla que se elija con `--plantilla`, y trabajan sobre el árbol ya
   analizado, así que nunca tocan lo que hay dentro de un bloque de código:
   - `html_en_linea` — el HTML en línea que Markdown permite escribir a mano
     (`<br>`, `<mark>`, `<sub>`, `<kbd>`, `<img>`…). El escritor de LaTeX
     **descarta el HTML sin avisar**, así que sin el filtro ese contenido
     desaparece del PDF en silencio.
   - `imagenes` — descarga las imágenes de la web y avisa de las locales que
     faltan (ver más abajo).
   - `diagramas` — dibuja con Graphviz los bloques ```` ```dot ````.
   - `graficas` — deja pasar a LaTeX los bloques ```` ```pgfplot ```` y
     ```` ```tikz ````, envueltos en `tikzpicture` y, si llevan `caption`, en
     una figura. Es la única pieza donde un error del documento **detiene** la
     compilación: son comandos de LaTeX de verdad, no un lenguaje aparte como
     el de Graphviz.
   - `bloques` — las cajas `::: nota` y la sangría francesa de las referencias.
     Emiten `\begin{CajaMarcada}` y `\begin{ReferenciasAPA}`, dos entornos que
     define `base.ltx`: si se toca uno hay que tocar el otro.
3. `Latex/base.ltx` — los comandos que Pandoc da por definidos (ver más abajo).

Los avisos de los filtros salen por stderr con el prefijo `[investigacion]`;
`filter_warnings()` los separa del ruido de Pandoc y `pandoc_to_latex()` los
pasa al mismo callback `on_warning` que ya usaba `compile_pdf()`.

Para ver qué produce hoy Pandoc con las extensiones activas:

```bash
pandoc archivo.md --from=markdown-raw_tex+mark+emoji+autolink_bare_uris \
  --to=latex --wrap=none --lua-filter=src/investigacion/filtros/html_en_linea.lua
```

### La trampa de soul: \st, \hl y \ul

Pandoc emite `\st` (tachado), `\hl` (resaltado) y `\ul` (subrayado) contando con
que los defina `soul`, que es lo que hace su plantilla por omisión. Pero soul
analiza el texto letra por letra y eso falla en dos sitios que un trabajo
escolar usa a diario:

- dentro de `longtable` —la tabla que genera Pandoc— `\st` y `\hl` dejan a
  pdflatex **dando vueltas para siempre**: sin error, sin PDF y sin log que
  crezca. Un `~~tachado~~` en una celda basta;
- en un título el comando viaja al índice y la compilación aborta.

Por eso `base.ltx` redefine `\st` y `\ul` sobre `ulem` (`\sout` y `\uline`, que
no hacen ese análisis) y los declara con `\DeclareRobustCommand` para que no se
expandan al escribir el índice. El resaltado no tiene sustituto que corte
renglón, así que se conserva el de soul en el texto corriente y
`\AtBeginEnvironment{longtable}` lo cambia por un `\colorbox` dentro de las
tablas. Si algún día hay que tocar esto, la prueba que lo cubre es
`test_extended_syntax_compiles`.

Como red de seguridad —soul no es el único paquete capaz de colgarse—
`compile_pdf()` y `pandoc_to_latex()` corren con `timeout=TOOL_TIMEOUT_SECONDS`:
más vale un error legible que una terminal congelada.

### Gráficas con pgfplots

El bloque `GRÁFICAS DE DATOS` de `base.ltx` carga `pgfplots` y fija un estilo
sobrio (escala de grises, rejilla tenue) que cada gráfica puede sobrescribir en
su propio `\begin{axis}[...]`, porque va en `every axis/.append style`.

Dos detalles que costaron encontrarse:

- Las barras **no usan `cycle list`** sino `bar cycle list`, que por omisión es
  de colores; sin redefinirla un `ybar` sale azul y rojo.
- En `cycle list` no puede haber `fill`: en una gráfica de líneas rellenaría el
  área bajo la curva. El relleno vive solo en `bar cycle list`.

### Imágenes y diagramas: la caché del proyecto

pdflatex no descarga nada, así que una imagen de la web llegaría como
`\includegraphics{https://…}` y la compilación fallaría con un «file not found»
que no explica la causa. `filtros/imagenes.lua` la baja antes con
`pandoc.mediabag.fetch` y reescribe la ruta a la copia local; a partir de la
segunda vez el trabajo se genera sin internet.

`media_directories()` crea y devuelve `imagenes/remotas/` e
`imagenes/diagramas/` en la raíz del proyecto (`project_root()`, mismo criterio
que `find_template()`). Las dos son **caché**: se pueden borrar enteras. El
nombre de cada archivo es el `sha1` de la URL o del código del diagrama, que es
lo que permite saber si ya está hecho sin volver a pedirlo.

Los filtros reciben esas rutas por variables de entorno
(`INVESTIGACION_IMAGENES`, `INVESTIGACION_DIAGRAMAS`, `INVESTIGACION_RECURSOS`),
que pone `pandoc_to_latex()` junto con un `--resource-path` que incluye la
carpeta del Markdown y la del proyecto.

Cuando algo falla —no hay red, la imagen no existe, falta Graphviz— el filtro
**no rompe nada**: sustituye la imagen por su texto alternativo o deja el
diagrama como bloque de código, y avisa. Mismo criterio que el `[?]` de los
símbolos Unicode.

Dos cosas que conviene recordar antes de tocar esto:

- pdflatex solo compone PNG, JPG y PDF. El filtro rechaza lo demás con un aviso
  en vez de dejar que falle la compilación.
- Pandoc no manda ningún `User-Agent` al descargar y hay sitios que por eso
  responden 400; de ahí el `--request-header` de `pandoc_to_latex()`.

### La TUI

`src/investigacion/tui.py` es un menú de `input()` (sin `curses`, que no existe
en Windows). No duplica lógica: arma un `argv` con `build_argv()` y llama a
`cli.main()`. Los campos vacíos no se pasan, para que siga valiendo el respaldo
del `.env`. Al añadir una opción al CLI hay que añadirla también a `FIELDS` (cada `Field`
lleva su ayuda, un ejemplo genérico y qué pasa si se deja vacío).

### Dónde se busca el Markdown

`resolve_markdown_path()` delega en `find_markdown()`, que prueba tres cosas en
orden: la ruta tal cual, la ruta dentro de `input/` y, si lo que se pasó es solo
un nombre, una búsqueda recursiva por ese nombre bajo `input/`. La idea es que
`investigacion Actividad1.md` funcione aunque el archivo esté en
`input/IS/Actividad1.md`.

Dos decisiones que conviene no invertir:

- **Una ruta que ya existe siempre gana.** La búsqueda por nombre es el último
  recurso, para que nada se vuelva ambiguo cuando se escribe la ruta completa.
- **Dos archivos con el mismo nombre son un error**, no una elección arbitraria:
  el mensaje lista las subcarpetas para que la persona decida.

### Espacio invisible en el Markdown

`normalize_markdown()` existe por un fallo real y difícil de ver: los documentos
exportados de otras herramientas (o escritos por una IA) suelen traer **espacios
duros** U+00A0 en las líneas que separan párrafos y dos espacios al final de
cada renglón. Esas líneas no están vacías, así que Pandoc no separa párrafos y
convierte el documento entero en un bloque continuo: los `###` salen literales
en el PDF, las tablas quedan como filas de barras y el índice se queda con una
sola entrada. Compila sin error, y por eso pasa inadvertido.

La función recorta el final de cada línea (`INVISIBLE_SPACES` cubre duro, de
figura, estrecho y de ancho cero) y repara los marcadores separados del texto
por un espacio duro (`MARKER_SEPARATOR`: `###`, viñetas, listas numeradas y
citas), porque Markdown exige un espacio normal después del marcador.

Los dos espacios finales son a la vez basura y sintaxis (en Markdown son un
salto de línea), así que el recorte es condicional: `has_disguised_spaces()`
decide. Si el documento trae algún espacio disfrazado al final de una línea se
recorta todo, como antes; si viene limpio solo se recortan los disfrazados y el
tabulador, y el salto de línea llega al PDF. `DISGUISED_SPACES` es ese conjunto
sin el espacio normal.

Lo que **no** hace: tocar el espacio duro dentro del texto (ahí es legítimo) ni
entrar en los bloques ``` o ~~~, donde el espacio final puede ser parte del
código. `read_markdown()` la aplica, así que la usan tanto el CLI como
`generate_pdf()`.

### Símbolos Unicode

pdflatex con `inputenc` solo compone los caracteres que tenga declarados: un `≠`
suelto aborta con «Unicode character not set up for use with LaTeX». Es el fallo
más habitual al pegar texto de una web o de una IA.

El bloque `SÍMBOLOS UNICODE` de `base.ltx` declara con `newunicodechar` los que
faltaban de verdad (los griegos, las relaciones matemáticas, conjuntos, cálculo
y los checkmarks). La lista **no se inventó**: se compiló un documento con todos
los candidatos y se leyó del log cuáles avisaban, así que `→ ± × ÷ ° … — – « »
¿ ¡ € • ½` no están porque inputenc ya los conoce.

Debajo, `\UTFviii@undefined@err` (de `utf8.def`) se redefine para que un símbolo
no declarado emita un **aviso** y un `[?]` visible en vez de abortar. Es la red
de seguridad: la lista nunca cubrirá todos los símbolos posibles, y el criterio
del proyecto es avisar sin impedir que el trabajo se genere.

`unsupported_character_warnings()` lee esos avisos del log y `compile_pdf()` los
pasa al callback `on_warning`, que el CLI imprime en stderr; sin eso el `[?]`
pasaría inadvertido. El generador sigue sin imprimir nada por su cuenta.

El log hay que leerlo con `decode_latex_log()`, no con `decode_process_output()`:
pdflatex mezcla mensajes en UTF-8 con líneas de división silábica escritas en la
codificación interna de la fuente (T1), así que el archivo entero no es UTF-8
válido y el respaldo a cp1252 convertía los avisos en «â””» en vez de «└». Se
decodifica línea por línea.

Dos casos no caen en el aviso porque la plantilla los resuelve sola:

- **Dibujo de caja** (`└ ─ ┼ │ ╭`). Son el arte ASCII que las IA meten dentro de
  un bloque de código; `pmboxdraw` los compone como reglas del ancho exacto de
  la letra monoespaciada, así que el diagrama no se desalinea. Las esquinas
  redondeadas y las figuras geométricas (`► ▲ ▼ ● ■`) no están en ese paquete y
  se declaran a mano en el bloque `SÍMBOLOS UNICODE`.
- **Emoji**. `twemojis` los trae como páginas de un PDF y los inserta con
  `\texttwemoji{<hex>}`. Declararlos uno por uno serían miles de líneas, así que
  en vez de una lista el propio `\UTFviii@undefined@err` calcula el punto de
  código (con `\decode@UTFviii`, el mismo de `utf8.def`), arma el nombre del
  comando de twemojis y lo usa **si existe**; si no, cae al aviso de siempre. Por
  eso también se declaran vacíos U+FE0F y U+200D, que son invisibles y si no
  dejarían un `[?]` al lado de cada emoji.

### Los logos: fuera del repositorio y configurables

`Latex/logos/` está en `.gitignore` (con una excepción para su `LEEME.md`):
los logos de una institución rara vez son redistribuibles, así que el proyecto
no trae ninguno. La plantilla busca dos nombres genéricos,
`logo-universidad.png` y `logo-facultad.png`, cada uno envuelto en
`\IfFileExists`, de modo que sin ellos la portada se compila igual.

`resolve_logos_directory()` decide de dónde salen: la ruta de `--logos` (o de la
variable `LOGOS` del `.env`, que resuelve el CLI) gana sobre la carpeta `logos/`
que esté junto a la plantilla. Una ruta inexistente es `GenerationError`; que no
haya carpeta por omisión, en cambio, no es error: devuelve `None`.

Los documentos de `ejemplo/` **no pueden depender de los logos** por esa misma
razón; usan `ejemplo/imagen-ejemplo.png`, que sí se versiona.

### Los logos y TEXINPUTS

`compile_pdf()` ejecuta pdflatex con `cwd=markdown_path.parent` (p. ej. `Crudo/`)
y el `.tex` en un temporal, y TeX resuelve las imágenes contra el **cwd**: un
`\includegraphics{Latex/logos/logo-uat.png}` no se encontraría al generar desde
otro directorio. Por eso `copy_template_assets()` copia
`<directorio de la plantilla>/logos/` al temporal y `latex_search_path()` pone
ese temporal al principio de `TEXINPUTS` (con separador final, que en TeX
significa «y además las rutas por omisión»; sin él pdflatex no encontraría ni sus
propios paquetes). La plantilla entonces escribe solo `{logo-uat}`.

Cada logo va envuelto en `\IfFileExists`, así que una plantilla sin `logos/` al
lado —caso de `--plantilla`— compila igual, solo que sin logos.

### Compilación de LaTeX

`compile_pdf()` repite pdflatex hasta que el `.toc` se estabiliza (máximo
`MAX_LATEX_RUNS`), porque un índice que cambia de longitud deja mal los números
de página. Al fallar, `summarize_latex_errors()` extrae del log solo los errores
reales (líneas `archivo:línea:` de `-file-line-error` y líneas `!`) y descarta
las rutas de paquetes; `keep_failure_artifacts()` guarda `ultimo-error.tex` y
`ultimo-error.log` en el directorio de salida, porque el temporal se borra.

### raw_tex desactivado por omisión

`pandoc_to_latex()` usa `markdown-raw_tex` salvo que se pase `--permitir-latex`.
Sin eso, una contrabarra suelta del texto corriente (una ruta `C:\Users\...`) se
enviaría a LaTeX como comando y abortaría la compilación. Las fórmulas `$...$`
no dependen de esa extensión y siguen funcionando en ambos modos.

### Validación de estructura

`validate_markdown()` solo advierte —nunca impide generar el PDF— si faltan
`Introducción`, `Desarrollo`, `Conclusión` o `Referencias`, o si están
desordenadas. `markdown_headings()` ignora los bloques delimitados por ``` o ~~~
para que un `# comentario` dentro de código no cuente como encabezado.

## Portabilidad

El proyecto se desarrolla en Linux pero tiene que funcionar igual en Windows y
macOS, así que **no debe aparecer nada específico de un sistema**: ni rutas
fijas, ni `os.name`, ni separadores escritos a mano. Lo que ya está resuelto y
conviene no deshacer:

- **`os.pathsep`** para `TEXINPUTS` y `--resource-path`: en Windows el separador
  es `;` y en el resto `:`.
- **`Path.as_posix()`** en las variables de entorno que reciben los filtros Lua.
  Esas rutas acaban dentro de un `\includegraphics`, y en LaTeX la contrabarra
  de una ruta de Windows empezaría un comando inexistente. TeX acepta la barra
  normal en todos los sistemas.
- **`ignore_cleanup_errors=True`** en el directorio temporal: en Windows es
  normal que pdflatex deje un archivo bloqueado un instante, y el PDF ya está
  copiado cuando eso ocurre.
- **`reconfigure(errors="replace")`** sobre stdout y stderr en `main()`: los
  avisos incluyen el carácter problemático y la consola de Windows no siempre
  puede representarlo.
- Toda lectura y escritura de texto lleva **codificación explícita**; en Windows
  la de por omisión no es UTF-8.
- `.gitattributes` normaliza los finales de línea a LF.

## Directorios

- `Latex/base.ltx` — plantilla APA con portada TikZ.
- `Latex/logos/` — carpeta donde cada quien deja sus logos; ignorada por git
  salvo su `LEEME.md`, que explica los nombres que busca la plantilla.
- `src/investigacion/filtros/` — filtros Lua para Pandoc. Se instalan como
  datos del paquete, ver `pyproject.toml`.
- `imagenes/` — caché de las imágenes descargadas de la web y de los diagramas
  dibujados con Graphviz. Se crea sola, se puede borrar y está en `.gitignore`.
- `ejemplo/investigacion.md` — esqueleto con la estructura recomendada.
- `ejemplo/sintaxis.md` — referencia breve del Markdown que se soporta.
- `ejemplo/arboles-binarios.md` — un trabajo completo de ejemplo, con diagramas.
- `ejemplo/catalogo.md` — muestrario completo: un ejemplo de cada tabla,
  diagrama, gráfica y caja. Es el **banco de pruebas visual** del proyecto:
  si se toca la plantilla o un filtro, generarlo y revisar las páginas es la
  forma más rápida de ver qué se rompió. `ejemplo/catalogo.pdf` es su salida,
  versionada para poder verla sin instalar nada.
- `docs/imagenes/` — capturas del PDF que usa el README.
- `REQUISITOS.md` y `PROMPT-IA.md` — instalación por sistema operativo y el
  prompt con el que se le pide el trabajo a una IA. Si cambia la sintaxis que
  acepta el generador, el prompt hay que actualizarlo también: es la
  especificación que lee el modelo.

`ejemplo/catalogo.pdf` y esas capturas son lo único generado que se versiona, y
**no deben llevar datos personales ni logos de nadie**. Se rehacen así:

```bash
mkdir -p /tmp/sinlogos
investigacion ejemplo/catalogo.md \
  --titulo "Catalogo de elementos" --materia "Nombre de la materia" \
  --docente "Nombre del docente" --grupo "7-A" \
  --env-file .env.example --logos /tmp/sinlogos --salida /tmp/pub
cp /tmp/pub/catalogo-de-elementos.pdf ejemplo/catalogo.pdf
pdftoppm -r 110 -png -f 1 -l 1 ejemplo/catalogo.pdf docs/imagenes/portada
```

El `--env-file .env.example` es la clave: la portada sale con los mismos
marcadores que ve quien clona el repositorio («Nombre de la universidad»,
«Nombre del alumno»), y `--logos` a una carpeta vacía evita publicar las marcas
de una institución.
- `output/` — PDFs generados, ignorado por git.
- `input/` — los trabajos propios en Markdown, con las subcarpetas que cada
  quien quiera (por materia, por semestre). Está en `.gitignore`: no forma parte
  del proyecto, y el comando busca ahí los archivos por su cuenta.

## Documentación

Tres archivos, con papeles distintos; al cambiar el comportamiento hay que ver
cuál toca:

- `README.md` — presentación del proyecto para quien llega de GitHub: qué hace,
  requisitos, instalación e inicio rápido.
- `GUIA.md` — manual de uso: la tubería explicada, todas las opciones del
  comando, la sintaxis completa que acepta y la tabla de síntomas cuando algo
  falla.
- `CLAUDE.md` — este archivo: los contratos internos y las trampas encontradas,
  para quien vaya a tocar el código.
