# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Idioma

El proyecto es de un estudiante hispanohablante, pero **el código va en
inglés**: identificadores, módulos, archivos, carpetas, opciones del CLI y
mensajes del CLI y de la TUI. Los **comentarios van en español**, breves: uno
por bloque y por línea solo en casos especiales (trampas de LaTeX, portabilidad).
La documentación para el usuario (README, GUIA, REQUISITOS, MEJORAS) y lo que
sale en el PDF (plantillas) siguen en español. Las opciones antiguas en español
(`--titulo`, `--materia`…) se conservan como alias ocultos de clap.

## Comandos

```bash
cargo build --release                 # binario en target/release/investigacion
cargo install --path .                # lo instala en ~/.cargo/bin
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test                            # todas las pruebas
cargo test --lib markdown             # las de un módulo
cargo test --test pdf                 # las que compilan PDF reales

# Ejecutar el CLI (sin argumentos abre la TUI)
cargo run --release -- Economia.md --title "Tema" --course "Materia" --teacher "Docente"
cargo run --release -- Tarea1.md -p ia --title "Tema"

# Medir cuánto tarda cada herramienta
INVESTIGACION_TIMING=1 target/release/investigacion examples/catalog.md --title T --course M
```

Dependencias del crate: `clap`, `regex`, `unicode-normalization`, `tempfile`,
`thiserror`, `chrono`, `serde` + `toml` (perfiles), `ratatui` + `crossterm`
(TUI) y `opener` (abrir carpetas y PDF). Pandoc, pdflatex y Graphviz son
dependencias **del sistema**. Rust mínimo: 1.88 (`rust-version` en Cargo.toml;
el código usa let-chains de la edición 2024).

## Arquitectura

Tubería de una sola dirección, sin estado intermedio persistente:

```
archivo.md
  → read_markdown()      decodifica UTF-8 con respaldo CP1252
  → normalize_markdown() quita el espacio invisible (lo llama read_markdown)
  → validate_markdown()  solo advertencias, nunca bloquea
  → pandoc_to_latex()    Pandoc emite un FRAGMENTO de LaTeX (sin preámbulo);
                         con las extensiones de MARKDOWN_EXTENSIONS y los
                         filtros Lua de resources/filters/
  → render_template()    sustituye los marcadores %%NOMBRE%% de la plantilla
  → compile_pdf()        pdflatex en un directorio temporal, varias pasadas
  → copy_pdf_to()        copias opcionales en otros directorios
```

Módulos de `src/` (biblioteca `investigacion` + binarios):

| Módulo | Qué hace |
|---|---|
| `encoding.rs` | UTF-8 con respaldo Windows-1252; el log de pdflatex línea a línea |
| `markdown.rs` | `read_markdown`, `normalize_markdown`, búsqueda en `input/`, validación |
| `latex.rs` | `latex_escape`, `render_template`, lectura del log |
| `pandoc.rs` | `pandoc_to_latex`; los filtros Lua van **embebidos** con `include_str!` |
| `compile.rs` | pasadas de pdflatex, TEXINPUTS, caché del `.aux`/`.toc` |
| `generate.rs` | `generate_pdf` (la tubería), `copy_pdf_to`, nombre del PDF |
| `project.rs` | raíz del proyecto, carpetas, plantillas y logos |
| `settings.rs` | `.env` + entorno, sin modificar el entorno del proceso |
| `courses.rs` | perfiles de materia (`courses/*.toml`) |
| `cli.rs` | `Args` (clap) y `execute()`, que usan el CLI y la TUI |
| `tui/` | menú a pantalla completa: `app.rs` (estado), `picker.rs`, `ui.rs` |
| `process.rs` | ejecutar herramientas con timeout |

Todo fallo esperado es `GenerationError` (un mensaje para mostrar tal cual);
`cli::main_with_args` lo imprime y devuelve 1. `execute()` informa por un
`Reporter` (consola en el CLI, canal hacia la TUI), así que la lógica no
imprime nada por su cuenta.

### El contrato crítico: plantilla ↔ salida de Pandoc

Pandoc genera un **fragmento**, no un documento completo, así que la plantilla
tiene que aportar por su cuenta todo el preámbulo que la salida
de Pandoc da por supuesto. Es la fuente de fallos más habitual del proyecto: un
documento con tablas, imágenes o bloques de código falla mientras uno de solo
texto compila bien.

Las plantillas están en dos niveles. `templates/common/investigacion.sty` es el
preámbulo compartido (todo lo de abajo) y `templates/common/investigacion-final.sty`
el cierre (hyperref, footnotehyper, `\AutorPDF`); cada `templates/<nombre>/template.ltx`
solo pone los datos y la portada y carga los dos con `\usepackage`.
`copy_template_assets()` copia los `.sty` de `common/` (y los que traiga la
propia plantilla) al temporal, que está en `TEXINPUTS`. Dentro de un `.sty` no
hay `\makeatletter` (la arroba ya es letra) y se usa `\RequirePackage`.

El bloque `PAQUETES QUE NECESITA LA SALIDA DE PANDOC` de `investigacion.sty` cubre
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
nuevo hay que tocar `DocumentData`, la lista de `render_template()` y **todas**
las plantillas a la vez (la prueba `every_template_compiles_…` lo vigila).

`%%INTEGRANTES%%` es el único que no sale de un solo `latex_escape()`:
`DocumentData.members` es un `Vec<String>` y la lista se arma escapando **cada
nombre por separado**
y uniéndolos después con un `\\` literal. Al revés no funciona: `latex_escape()`
convertiría esas contrabarras en `\textbackslash{}` y los nombres saldrían en un
solo renglón.

### Datos opcionales de la portada: alumno, integrantes y grupo

Del comando solo son obligatorios `--title` y `--course` (y la materia puede
venir de un perfil). `--teacher` sigue el mismo patrón que `--group` (`Option`
en `Args`; la opción gana, luego el perfil y luego la variable del `.env`, ver
`pick()` en `cli.rs`) y su línea se omite cuando queda vacío, así que en la
portada solo `MATERIA` y `SEMESTRE` aparecen siempre.

`ALUMNO` ya no está en `REQUIRED_ENV` (solo quedan `UNIVERSIDAD`, `FACULTAD` y
`SEMESTRE`); `--members` acepta los nombres separados por comas o punto y
coma y `parse_members()` los normaliza. Como respaldo se lee la variable
`INTEGRANTES` del `.env`, pero `--members` manda. Cada variable acepta también
su nombre en inglés (`Settings::get` recibe la lista de nombres).

`--group` sigue exactamente el mismo patrón (opción, perfil, variable `GRUPO`) y su línea cierra el bloque
con el mismo `\ifdefempty`. Es el molde a copiar para el próximo dato opcional
de la portada; `--logos` lo repite con una ruta en vez de un texto.

El orden del bloque de datos es **fijo** —alumno (o integrantes), materia,
docente, semestre, grupo— y vive en la plantilla, no en el CLI: el orden en que
se escriban las opciones del comando no lo altera. La prueba `the_cover_order_is_fixed`
(`tests/templates.rs`) lo comprueba en la plantilla `apa`; `apa-simple` respeta el
mismo orden.

Las dos líneas son **excluyentes** en la portada, no acumulativas: si hay
integrantes se lista al equipo y `ALUMNO:` no aparece aunque `ALUMNO` siga
puesto en el `.env` (un trabajo de equipo no repite la firma individual); sin
integrantes se muestra el alumno; y si faltan los dos no aparece ninguna. Eso es
un `\ifdefempty{\ListaIntegrantes}` externo con el caso del alumno anidado en su
rama vacía: la decisión vive en la plantilla, no en Rust, así que
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

1. `MARKDOWN_EXTENSIONS` en `pandoc.rs` — extensiones que Pandoc no activa
   por omisión: `mark` (`==resaltado==`), `emoji` (`:joy:`) y
   `autolink_bare_uris` (una URL suelta se vuelve enlace).
2. Los filtros Lua de `resources/filters/`, que se aplican en el orden de
   `LUA_FILTERS`. Van embebidos en el binario y no en `templates/`, porque no
   dependen de la plantilla que se elija con `--template`, y trabajan sobre el árbol ya
   analizado, así que nunca tocan lo que hay dentro de un bloque de código:
   - `inline_html` — el HTML en línea que Markdown permite escribir a mano
     (`<br>`, `<mark>`, `<sub>`, `<kbd>`, `<img>`…). El escritor de LaTeX
     **descarta el HTML sin avisar**, así que sin el filtro ese contenido
     desaparece del PDF en silencio.
   - `images` — descarga las imágenes de la web y avisa de las locales que
     faltan (ver más abajo).
   - `diagrams` — dibuja con Graphviz los bloques ```` ```dot ````.
   - `charts` — deja pasar a LaTeX los bloques ```` ```pgfplot ```` y
     ```` ```tikz ````, envueltos en `tikzpicture` y, si llevan `caption`, en
     una figura. Es la única pieza donde un error del documento **detiene** la
     compilación: son comandos de LaTeX de verdad, no un lenguaje aparte como
     el de Graphviz.
   - `blocks` — las cajas `::: nota` y la sangría francesa de las referencias.
     Emiten `\begin{CajaMarcada}` y `\begin{ReferenciasAPA}`, dos entornos que
     define `investigacion.sty`: si se toca uno hay que tocar el otro.
3. `templates/common/investigacion.sty` — los comandos que Pandoc da por
   definidos (ver más abajo).

Los avisos de los filtros salen por stderr con el prefijo `[investigacion]`;
`filter_warnings()` los separa del ruido de Pandoc y `pandoc_to_latex()` los
pasa al mismo callback `on_warning` que ya usaba `compile_pdf()`.

Para ver qué produce hoy Pandoc con las extensiones activas:

```bash
pandoc archivo.md --from=markdown-raw_tex+mark+emoji+autolink_bare_uris \
  --to=latex --wrap=none --lua-filter=resources/filters/inline_html.lua
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

Por eso `investigacion.sty` redefine `\st` y `\ul` sobre `ulem` (`\sout` y `\uline`, que
no hacen ese análisis) y los declara con `\DeclareRobustCommand` para que no se
expandan al escribir el índice. El resaltado no tiene sustituto que corte
renglón, así que se conserva el de soul en el texto corriente y
`\AtBeginEnvironment{longtable}` lo cambia por un `\colorbox` dentro de las
tablas. Si algún día hay que tocar esto, la prueba que lo cubre es
`the_extended_syntax_compiles_without_lost_symbols` (`tests/pdf.rs`).

Como red de seguridad —soul no es el único paquete capaz de colgarse—
`compile_pdf()` y `pandoc_to_latex()` corren con `timeout=TOOL_TIMEOUT_SECONDS`:
más vale un error legible que una terminal congelada.

### Gráficas con pgfplots

El bloque `GRÁFICAS DE DATOS` de `investigacion.sty` carga `pgfplots` y fija un estilo
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
que no explica la causa. `filters/images.lua` la baja antes con
`pandoc.mediabag.fetch` y reescribe la ruta a la copia local; a partir de la
segunda vez el trabajo se genera sin internet.

`Project::media_directories()` crea y devuelve `cache/remote/` y
`cache/diagrams/` en la raíz del proyecto (`Project::discover()`). Las dos son **caché**: se pueden borrar enteras. El
nombre de cada archivo es el `sha1` de la URL o del código del diagrama, que es
lo que permite saber si ya está hecho sin volver a pedirlo.

Los filtros reciben esas rutas por variables de entorno
(`INVESTIGACION_REMOTE_IMAGES`, `INVESTIGACION_DIAGRAMS`, `INVESTIGACION_RESOURCES`),
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

`src/tui/` es un menú a pantalla completa con ratatui + crossterm (funciona en
la consola de Windows; `curses` no). No duplica lógica: `App::build_args()` arma
los mismos `Args` que el CLI y un hilo llama a `cli::execute()`, que informa por
un canal (`ChannelReporter`) para no congelar la pantalla. Los campos vacíos no
se pasan, para que sigan valiendo el perfil y el `.env`.

- `app.rs` es el estado y la respuesta a cada tecla, **sin dibujar**: se prueba
  con teclas simuladas (`src/tui/tests.rs`). `ui.rs` solo dibuja.
- `picker.rs` es el mismo selector para el Markdown (navega carpetas), los
  perfiles, las plantillas y las carpetas del proyecto (`f`). El filtro ordena
  por nombre exacto > prefijo > contiene > descripción: sin eso, `ia` elegía el
  perfil `example` porque su materia dice «mater**ia**».
- En Windows llegan también los eventos de soltar tecla: el bucle filtra
  `KeyEventKind::Press`.
- El menú no ofrece `.env`, «permitir LaTeX», plantilla por ruta ni logos (las
  antiguas opciones 9–12); siguen en el CLI. Al añadir una opción al CLI que sí
  deba estar en el menú, va en `FIELDS` (con ayuda, ejemplo y qué pasa si se
  deja vacío) y en `build_args()`.

### Perfiles de materia y nombre del PDF

`courses/<clave>.toml` (`SubjectProfile`, con `deny_unknown_fields` para que
un campo mal escrito sea error y no se ignore) guarda materia, docente, grupo,
integrantes, plantilla y carpeta. Con `folder`, `locate_markdown()` busca
primero en `input/<folder>` y la salida por omisión es `output/<folder>`.
`courses/` está en `.gitignore` salvo `example.toml` y `LEEME.md`.

El nombre del PDF **no sale del título**: `output_file_name()` usa el nombre del
Markdown, o `--file-name` si se da. En la TUI se propone al elegir el Markdown
y deja de proponerse en cuanto la persona escribe uno.

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

El bloque `SÍMBOLOS UNICODE` de `investigacion.sty` declara con `newunicodechar` los que
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

`templates/logos/` está en `.gitignore` (con una excepción para su `LEEME.md`):
los logos de una institución rara vez son redistribuibles, así que el proyecto
no trae ninguno. La plantilla busca dos nombres genéricos,
`logo-universidad.png` y `logo-facultad.png`, cada uno envuelto en
`\IfFileExists`, de modo que sin ellos la portada se compila igual.

`Project::resolve_logos_directory()` decide de dónde salen: la ruta de
`--logos` (o de la variable `LOGOS` del `.env`, que resuelve el CLI) gana sobre
la carpeta `logos/` que esté junto a la plantilla, y esa sobre la compartida
`templates/logos/`. Una ruta inexistente es `GenerationError`; que no
haya carpeta por omisión, en cambio, no es error: devuelve `None`.

Los documentos de `examples/` **no pueden depender de los logos** por esa misma
razón; usan `examples/sample-image.png`, que sí se versiona.

### Los logos y TEXINPUTS

`compile_pdf()` ejecuta pdflatex con `cwd=markdown_path.parent` (p. ej. `Crudo/`)
y el `.tex` en un temporal, y TeX resuelve las imágenes contra el **cwd**: un
`\includegraphics{templates/logos/logo-uat.png}` no se encontraría al generar desde
otro directorio. Por eso `copy_template_assets()` copia
la carpeta de logos (y los `.sty` de `common/`) al temporal y `latex_search_path()` pone
ese temporal al principio de `TEXINPUTS` (con separador final, que en TeX
significa «y además las rutas por omisión»; sin él pdflatex no encontraría ni sus
propios paquetes). La plantilla entonces escribe solo `{logo-uat}`.

Cada logo va envuelto en `\IfFileExists`, así que una plantilla sin `logos/` al
lado —caso de `--template ruta.ltx`— compila igual, solo que sin logos.

### Compilación de LaTeX

`compile_pdf()` repite pdflatex hasta que el `.toc` se estabiliza (máximo
`MAX_LATEX_RUNS`), porque un índice que cambia de longitud deja mal los números
de página.

**El tiempo se va en pdflatex** (~1.7 s por pasada en el catálogo; Pandoc
~0.3 s), así que el lenguaje del programa casi no influye. Dos optimizaciones:

- Sin estado previo, la primera pasada va con `-draftmode` (no escribe el PDF;
  nunca puede ser la definitiva porque lee un índice vacío).
- El `.aux` y el `.toc` de la última generación se guardan en
  `cache/latex/<pdf>-<hash>/` (una carpeta por PDF de salida y plantilla). Con
  ellos la primera pasada ya lee el índice correcto y, si no cambió, es la
  única: regenerar el catálogo baja de 3 pasadas (5.5 s) a 1 (2.0 s). Si ese
  estado viejo hace fallar la compilación, `compile_pdf()` lo borra y repite
  desde cero **sin** dejar `last-error.*` del primer intento. Al fallar, `summarize_latex_errors()` extrae del log solo los errores
reales (líneas `archivo:línea:` de `-file-line-error` y líneas `!`) y descarta
las rutas de paquetes; `keep_failure_artifacts()` guarda `last-error.tex` y
`last-error.log` en el directorio de salida, porque el temporal se borra.

### raw_tex desactivado por omisión

`pandoc_to_latex()` usa `markdown-raw_tex` salvo que se pase `--allow-latex`.
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
fijas, ni `cfg!(windows)`, ni separadores escritos a mano. Lo que ya está resuelto y
conviene no deshacer:

- **`std::env::join_paths` / `split_paths`** para `TEXINPUTS` y
  `--resource-path`: en Windows el separador es `;` y en el resto `:`.
- **`project::absolute()`** (`std::path::absolute`) en vez de `canonicalize()`:
  en Windows `canonicalize` devuelve rutas `\\?\C:\...` que pdflatex y Pandoc
  no entienden.
- **`pandoc::posix()`** en las variables de entorno que reciben los filtros Lua.
  Esas rutas acaban dentro de un `\includegraphics`, y en LaTeX la contrabarra
  de una ruta de Windows empezaría un comando inexistente. TeX acepta la barra
  normal en todos los sistemas.
- **`temporary.close()` sin propagar el error**: en Windows es normal que
  pdflatex deje un archivo bloqueado un instante, y el PDF ya está copiado
  cuando eso ocurre.
- Rust escribe en la consola de Windows con la API Unicode, así que un aviso con
  un símbolo raro no rompe la salida (lo que en Python obligaba a
  `reconfigure(errors="replace")`).
- La TUI usa crossterm y filtra `KeyEventKind::Press`; `opener` abre carpetas y
  PDF con el programa de cada sistema.
- Toda lectura de texto pasa por `decode_text()`: UTF-8 o Windows-1252, nunca la
  codificación por omisión del sistema.
- `.gitattributes` normaliza los finales de línea a LF.

## Directorios

- `src/` — el código Rust (ver la tabla de módulos); `tests/` — pruebas de
  integración (`templates.rs`, `pandoc.rs`, `pdf.rs`, `cli.rs`).
- `templates/` — `common/` (preámbulo compartido), una carpeta por plantilla
  (`apa/`, `apa-simple/`, cada una con su `template.ltx`) y `logos/`, ignorada
  por git salvo su `LEEME.md`.
- `resources/filters/` — filtros Lua para Pandoc, embebidos en el binario.
- `courses/` — perfiles de materia; ignorada salvo `example.toml` y `LEEME.md`.
- `cache/` — imágenes descargadas (`remote/`), diagramas (`diagrams/`) y estado
  de LaTeX (`latex/`). Se crea sola, se puede borrar y está en `.gitignore`.
- `examples/paper-template.md` — esqueleto con la estructura recomendada.
- `examples/syntax.md` — referencia breve del Markdown que se soporta.
- `examples/binary-trees.md` — un trabajo completo de ejemplo, con diagramas.
- `examples/catalog.md` — muestrario completo: un ejemplo de cada tabla,
  diagrama, gráfica y caja. Es el **banco de pruebas visual** del proyecto:
  si se toca la plantilla o un filtro, generarlo y revisar las páginas es la
  forma más rápida de ver qué se rompió. `examples/catalog.pdf` es su salida,
  versionada para poder verla sin instalar nada.
- `docs/images/` — capturas del PDF que usa el README.
- `REQUISITOS.md` y `PROMPT-IA.md` — instalación por sistema operativo y el
  prompt con el que se le pide el trabajo a una IA. Si cambia la sintaxis que
  acepta el generador, el prompt hay que actualizarlo también: es la
  especificación que lee el modelo.

`examples/catalog.pdf` y esas capturas son lo único generado que se versiona, y
**no deben llevar datos personales ni logos de nadie**. Se rehacen así:

```bash
mkdir -p /tmp/sinlogos
investigacion examples/catalog.md \
  --title "Catalogo de elementos" --course "Nombre de la materia" \
  --teacher "Nombre del docente" --group "7-A" \
  --env-file .env.example --logos /tmp/sinlogos --output /tmp/pub
cp /tmp/pub/catalogo.pdf examples/catalog.pdf
pdftoppm -r 110 -png -f 1 -l 1 examples/catalog.pdf docs/images/cover
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
