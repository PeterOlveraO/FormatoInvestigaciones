# Guía de uso

Todo lo que el programa acepta, cómo se le pide y qué hace por dentro. Para una
presentación general del proyecto, ver [README.md](README.md); para lo que hay
que instalar en cada sistema operativo, [REQUISITOS.md](REQUISITOS.md).

## Índice

- [Cómo funciona](#cómo-funciona)
- [Configuración](#configuración)
- [Dónde van los archivos](#dónde-van-los-archivos)
- [El comando](#el-comando)
- [Perfiles de materia](#perfiles-de-materia)
- [Plantillas](#plantillas)
- [Menú interactivo (TUI)](#menú-interactivo-tui)
- [La portada: alumno, integrantes y grupo](#la-portada-alumno-integrantes-y-grupo)
- [Estructura del trabajo](#estructura-del-trabajo)
- [Qué puedes escribir en el Markdown](#qué-puedes-escribir-en-el-markdown)
- [Texto pegado de otras herramientas](#texto-pegado-de-otras-herramientas)
- [Copias en otros directorios](#copias-en-otros-directorios)
- [Cuando algo falla](#cuando-algo-falla)
- [Personalizar la plantilla](#personalizar-la-plantilla)
- [Pruebas](#pruebas)

## Cómo funciona

El programa es una tubería de una sola dirección. Cada paso recibe lo que
produjo el anterior:

```
mi-trabajo.md
   │
   ├─ 1. Se lee el archivo (UTF-8, con respaldo Windows-1252) y se limpia
   │     el espacio invisible que traen los documentos pegados de otras
   │     herramientas
   │
   ├─ 2. Se avisa si faltan los encabezados recomendados (nunca bloquea)
   │
   ├─ 3. Pandoc convierte el Markdown en un FRAGMENTO de LaTeX, pasando por
   │     cinco filtros que añaden lo que Pandoc no hace solo:
   │        · el HTML en línea que el escritor de LaTeX descartaría
   │        · las imágenes de la web, que se descargan a la cache
   │        · los diagramas ```dot, que dibuja Graphviz
   │        · las gráficas ```pgfplot, que se dibujan en el propio PDF
   │        · las cajas ::: nota y la sangría francesa de las referencias
   │
   ├─ 4. El fragmento se inserta en la plantilla elegida
   │     (templates/<nombre>/template.ltx), que aporta la portada; el
   │     formato APA y el preámbulo vienen de templates/common/
   │
   ├─ 5. pdflatex compila en un directorio temporal, repitiendo las pasadas
   │     necesarias hasta que el índice se estabiliza (con el índice de la
   │     generación anterior, casi siempre basta una)
   │
   └─ 6. El PDF se copia a output/ y, si se pidió, a otros directorios
```

Lo importante de este diseño: **ningún paso intermedio deja archivos** en el
proyecto salvo la caché de `cache/`. Si algo falla en la compilación, se
guardan `last-error.tex` y `last-error.log` en el directorio de salida para
poder revisarlos.

El programa está escrito en Rust (`src/`): `generate.rs` arma la tubería,
`cli.rs` solo traduce argumentos y `tui/` es el menú interactivo. Los filtros de
Pandoc están en `resources/filters/` y viajan dentro del binario.

**¿Dónde se va el tiempo?** Casi todo en pdflatex (alrededor de 1.7 s por pasada
en el catálogo) y unas décimas en Pandoc. Por eso la primera generación de un
trabajo hace 2 o 3 pasadas y las siguientes, si no cambió la estructura, una
sola: el `.aux` y el `.toc` de la vez anterior se guardan en `cache/latex/`.
Con `INVESTIGACION_TIMING=1` el comando imprime cuánto tardó cada paso.

## Configuración

Los datos que no cambian entre trabajos van en `.env`, copiado de
`.env.example`:

```dotenv
UNIVERSIDAD="Nombre de la universidad"
FACULTAD="Nombre de la facultad"
SEMESTRE="2026-2"

# Opcionales
ALUMNO="Nombre del alumno"
INTEGRANTES="Ana Ruiz, Luis Paz, Sofia Vela"
DOCENTE="Nombre del docente"
GRUPO="7-A"
LOGOS="/home/usuario/Documentos/logos-de-mi-universidad"
```

`UNIVERSIDAD`, `FACULTAD` y `SEMESTRE` son obligatorias. Las demás son
opcionales: `ALUMNO` e `INTEGRANTES` pueden faltar las dos, y `DOCENTE` y
`GRUPO` son el respaldo de las opciones del mismo nombre, pensados para no
repetirlos en cada trabajo del semestre.

Cada variable acepta también su nombre en inglés (`UNIVERSITY`, `FACULTY`,
`SEMESTER`, `STUDENT`, `MEMBERS`, `TEACHER`, `GROUP`).

También puedes definirlas en el entorno del shell: si ya existen ahí, tienen
prioridad sobre `.env`. El archivo `.env` está en `.gitignore`, así que tus
datos no se suben a ningún lado.

## Dónde van los archivos

```
input/       tus trabajos en Markdown, en subcarpetas por materia (ignorado por git)
output/      los PDF generados (ignorado por git)
courses/    perfiles de materia *.toml (ignorado por git salvo example.toml)
templates/   plantillas: apa/, apa-simple/, common/ (preámbulo común) y logos/
cache/       imágenes descargadas, diagramas y estado de LaTeX (ignorado por git)
```

Todas son relativas a la raíz del proyecto, no a la carpeta desde la que
ejecutes el comando: `investigacion` encuentra el proyecto solo (o lo lee de la
variable `INVESTIGACION_HOME`).

**No hace falta escribir la carpeta `input/` en el comando.** El archivo se busca
en tres sitios, en este orden:

1. La ruta tal como la escribiste, desde donde estés.
2. Dentro de `input/`, respetando las subcarpetas: `investigacion IS/Actividad1.md`.
3. Por nombre, en cualquier subcarpeta de `input/`: `investigacion Actividad1.md`
   encuentra `input/IS/Actividad1.md`.

Si hay dos archivos con el mismo nombre en subcarpetas distintas, el comando lo
dice y te pide que indiques cuál, con su subcarpeta.

## El comando

```bash
investigacion mi-trabajo.md --title "Ecuaciones diferenciales" --course "Cálculo"
```

Solo el título y la materia son obligatorios (la materia puede venir de un
perfil). El resultado es `output/mi-trabajo.pdf`: **el nombre del PDF sale del
nombre del Markdown, no del título**, en minúsculas, sin acentos y con guiones.
Así el título de la portada puede ser largo sin que el archivo se llame igual;
para otro nombre usa `--file-name`. Si vuelves a generar, el PDF anterior se
reemplaza. La fecha de entrega es la del día en que ejecutas el comando.

Sin argumentos, `investigacion` abre el menú interactivo (ver más abajo).

### Opciones

| Opción | Alias anterior | Obligatoria | Descripción |
|---|---|---|---|
| `markdown` | | Sí | Archivo `.md` con el contenido del trabajo |
| `--title` | `--titulo` | Sí | Título del trabajo (solo la portada) |
| `--course` | `--materia` | Sí, salvo con perfil | Nombre de la materia |
| `-p`, `--profile` | `--perfil` | No | Perfil de `courses/` con los datos de la materia |
| `--file-name` | `--nombre` | No | Nombre del PDF; por omisión, el del Markdown |
| `--teacher` | `--docente` | No | Nombre del docente; respaldo en la variable `DOCENTE` |
| `--members` | `--integrantes` | No | Nombres del equipo en un solo argumento, separados por comas o punto y coma |
| `--group` | `--grupo` | No | Grupo de la materia; cierra el bloque de datos de la portada |
| `--output` | `--salida` | No | Directorio del PDF (por omisión `output/`, u `output/<folder>` del perfil) |
| `--copy` | `--copia` | No | Directorio adicional para una copia; puede repetirse |
| `--template` | `--plantilla` | No | Plantilla: un nombre de `templates/` (`apa`, `apa-simple`) o una ruta |
| `--allow-latex` | `--permitir-latex` | No | Interpreta los comandos LaTeX escritos en el Markdown |
| `--env-file` | | No | Archivo dotenv alternativo (por omisión el `.env` del proyecto) |
| `--logos` | | No | Carpeta con los logos de la portada |

`investigacion --help` muestra lo mismo desde la terminal. Las opciones de la
columna «Alias anterior» siguen funcionando, para no romper comandos que ya
tengas escritos.

Prioridad de los datos: **opción del comando > perfil de materia > `.env`**.

### El orden de los argumentos no importa

El archivo `.md` puede ir en cualquier posición. Estas tres líneas hacen lo
mismo:

```bash
investigacion trabajo.md --title "Tema" --course "Materia" --teacher "Docente"
investigacion --title "Tema" --course "Materia" --teacher "Docente" trabajo.md
investigacion --teacher "Docente" trabajo.md --course "Materia" --title "Tema"
```

Lo único que importa es que cada valor con espacios vaya **entre comillas**. Sin
ellas el comando falla:

```bash
# Mal: "de" y "costos" se toman como argumentos sueltos
investigacion trabajo.md --title Conceptos de costos --course "M"
```

## Perfiles de materia

Un perfil guarda los datos que se repiten en todos los trabajos de una materia.
Es un archivo `courses/<clave>.toml`:

```toml
name = "Inteligencia artificial"   # obligatorio
teacher = "Nombre del docente"
group = "M"
members = "Ana Ruiz, Luis Paz"        # opcional
template = "apa-simple"               # plantilla de templates/
folder = "IA"                         # input/IA y output/IA
```

```bash
investigacion Tarea1.md -p ia --title "Búsqueda heurística"
```

Con `folder`, el Markdown se busca primero en `input/IA/` (así dos materias
pueden tener cada una su `Tarea1.md`) y el PDF se guarda en `output/IA/`.
Cualquier opción del comando sigue ganando sobre el perfil. Los perfiles están
en `.gitignore` (llevan nombres de docentes); `courses/example.toml` es la
plantilla para copiar.

## Plantillas

Cada carpeta de `templates/` con un `template.ltx` es un diseño de portada:

| Plantilla | Portada |
|---|---|
| `apa` | La geométrica con TikZ (la de siempre, por omisión) |
| `apa-simple` | Clásica, centrada y sin adornos |

Las dos comparten `templates/common/investigacion.sty` (el formato APA y todo lo
que necesita la salida de Pandoc) e `investigacion-final.sty` (hyperref y los
metadatos), así que el cuerpo del trabajo sale idéntico y solo cambia la
portada. Ver «Personalizar la plantilla» para crear una nueva.

## Menú interactivo (TUI)

`investigacion` sin argumentos (o `investigacion-tui`) abre un menú a pantalla
completa que funciona igual en Linux, macOS y Windows:

- **↑ ↓** recorren los campos; **Enter** edita el campo o abre su lista.
- Los campos con **▸** se eligen de una lista en vez de escribirse: el
  **perfil**, el **Markdown** y la **plantilla**. En la lista, las flechas
  mueven, Enter (o →) entra en una carpeta o elige, Retroceso (o ←) sube una
  carpeta y escribir filtra. El Markdown se busca empezando en la carpeta de la
  materia del perfil.
- Elegir un perfil rellena materia, docente, grupo, plantilla y carpeta de
  salida; elegir el Markdown propone el nombre del PDF.
- **g** genera el PDF; los avisos y errores salen en el panel «Result».
- **o** abre el último PDF y **f** abre una carpeta del proyecto (`input`,
  `output`, `courses`, `templates`, `cache`) en el explorador de archivos.
- **Supr** vacía un campo y **q** sale.

Los campos vacíos se comportan como si no hubieras escrito la opción (se usa el
perfil o el `.env`). El menú ya no pregunta por el archivo `.env`, «permitir
LaTeX», la plantilla por ruta ni la carpeta de logos: siguen disponibles como
opciones del comando.

## La portada: alumno, integrantes y grupo

El orden de la portada es **fijo** y no depende de cómo hayas escrito las
opciones del comando:

```
ALUMNO (o INTEGRANTES, si es trabajo de equipo)
MATERIA
DOCENTE
SEMESTRE
GRUPO
```

Solo **materia** y **semestre** aparecen siempre. Las líneas de alumno,
integrantes, docente y grupo son opcionales: si faltan, no dejan hueco.

La portada firma el trabajo de una sola forma: **los integrantes sustituyen al
alumno**. No hay que vaciar nada ni editar el `.env` para cambiar de un trabajo
individual a uno de equipo, basta con usar o no usar `--members`.

| Qué usas | Qué sale en la portada |
|---|---|
| Nada | Solo `ALUMNO:`, con el nombre del `.env` |
| `--members` | Solo `INTEGRANTES:`; `ALUMNO:` no aparece aunque esté en el `.env` |
| Ninguno de los dos | Ninguna de las dos líneas, sin dejar hueco |

**Trabajo individual.** El nombre sale de `ALUMNO` en el `.env`:

```
ALUMNO: Ana Ruiz Medina
MATERIA: Economía
```

**Trabajo en equipo.** Los nombres van en un solo argumento, separados por comas
(también valen los punto y coma):

```bash
investigacion trabajo.md --title "Tema" --course "Materia" --teacher "Docente" \
  --members "Ana Ruiz, Luis Paz, Sofia Vela"
```

Aparecen uno por renglón, alineados bajo el primero, y el `ALUMNO` del `.env` se
queda fuera de la portada:

```
INTEGRANTES: Ana Ruiz
             Luis Paz
             Sofia Vela
MATERIA: Economía
```

Si el equipo es siempre el mismo, puedes dejarlo fijo en el `.env` con la
variable `INTEGRANTES`; `--members` tiene prioridad cuando se usa.

El autor que se guarda en los metadatos del PDF sigue el mismo criterio: los
integrantes si los hay y, si no, el alumno.

**Grupo.** `--group` añade la última línea del bloque:

```
MATERIA: Economía
DOCENTE: Nombre del docente
SEMESTRE: 2026-2
GRUPO: 7-A
```

Si no lo indicas se lee `GRUPO` del `.env`, y si tampoco está ahí la línea no
aparece.

**Docente.** Funciona igual: `--teacher` gana sobre la variable `DOCENTE` del
`.env`, y sin ninguno de los dos la línea desaparece de la portada.

## Estructura del trabajo

El programa recomienda los encabezados `Introducción`, `Desarrollo`,
`Conclusión` y `Referencias`, y avisa si faltan o están desordenados. Son solo
advertencias: el PDF se genera igual y puedes añadir los encabezados y
subniveles que quieras.

En el Markdown **no** van la portada, el índice, el título del trabajo ni tus
datos: todo eso lo pone la plantilla. El documento empieza directamente en
`# Introducción`.

Los niveles de encabezado se traducen al formato APA:

| Markdown | Cómo sale |
|---|---|
| `#` | Centrado y en negrita (nivel 1 APA) |
| `##` | A la izquierda, en negrita |
| `###` | A la izquierda, en negrita cursiva |
| `####` | Sangrado, en negrita, seguido del texto con punto |
| `#####` | Sangrado, en negrita cursiva |

Las referencias se escriben a mano en formato APA 7, como lista con guiones o
como párrafos sueltos; el programa les da la sangría francesa que pide APA.

## Qué puedes escribir en el Markdown

Funciona la sintaxis completa de Markdown, básica y extendida, tal como la
describe la [Markdown Guide](https://www.markdownguide.org).

> Para ver cada elemento compuesto en un PDF real, genera
> [`examples/catalog.md`](examples/catalog.md) o abre
> [`examples/catalog.pdf`](examples/catalog.pdf).

**La regla de oro:** cada bloque va separado por una línea en blanco. Antes y
después de un encabezado, una lista, una tabla o un bloque de código. De ahí
sale casi cualquier PDF mal formado.

### Sintaxis básica

| Elemento | Cómo se escribe |
|---|---|
| Encabezados | `# Título`, `## Subtítulo`… hasta `######` |
| Negrita, cursiva | `**negrita**`, `*cursiva*`, `***ambas***` |
| Cita en bloque | `> cita`, se anida con `>>` y admite listas dentro |
| Lista con viñetas | `- item`; se anida con dos espacios |
| Lista numerada | `1. item`; se anida igual |
| Código en línea | `` `código` `` |
| Regla horizontal | `---`, `***` o `___` en su propia línea |
| Enlace | `[texto](https://...)`, con título `[texto](https://... "Título")` |
| Enlace por referencia | `[texto][ref]` y después `[ref]: https://...` |
| Imagen | `![texto alternativo](imagen.png)` |
| Salto de línea | `<br>` o una `\` al final del renglón |
| Escapar un símbolo | `\*` `\_` `\#` `\[` … |

### Sintaxis extendida

| Elemento | Cómo se escribe |
|---|---|
| Tabla | `\| Col \| Col \|` con `\|---\|---\|` debajo; `:---`, `:---:` y `---:` alinean |
| Bloque de código | ` ```python ` … ` ``` `; el lenguaje activa el resaltado |
| Nota al pie | `texto[^1]` y en otra línea `[^1]: la nota` |
| Identificador de encabezado | `### Título {#mi-id}` |
| Lista de definición | el término en una línea y `: definición` en la siguiente |
| Tachado | `~~texto~~` |
| Lista de tareas | `- [x] hecha` y `- [ ] pendiente` |
| Emoji | pegado (`🚀`) o por código (`:rocket:`) |
| Resaltado | `==texto==` |
| Subíndice, superíndice | `H~2~O`, `X^2^` |
| URL automática | `https://ejemplo.com` suelta se convierte en enlace |
| Caja de nota | `::: nota`, el texto, y `:::` para cerrar |
| Diagrama | un bloque de código con `{.dot}` |
| Gráfica | un bloque de código con `{.pgfplot}` |

### HTML en línea

Markdown permite intercalar HTML para lo que su sintaxis no cubre. Se traducen
`<br>`, `<em>`, `<strong>`, `<mark>`, `<sub>`, `<sup>`, `<del>`, `<s>`, `<u>`,
`<ins>`, `<kbd>`, `<samp>`, `<code>`, `<img src="...">` y `<hr>`. El HTML de
bloque (`<div>`, `<table>`, `<p>`) no tiene equivalente en el PDF y se descarta.

### Imágenes

La ruta se busca desde la carpeta del `.md` y desde `cache/` del proyecto,
así que una imagen que uses en varios trabajos basta con dejarla ahí una vez:

```markdown
![Escudo de la universidad](logo.png)
```

**También funcionan las imágenes de la web.** Se descargan la primera vez a
`cache/remote/` y a partir de ahí el trabajo se genera sin internet:

```markdown
![Diagrama del modelo OSI](https://ejemplo.com/osi.png)
```

El tamaño se controla con un atributo detrás del enlace; el porcentaje es
respecto al ancho del texto:

```markdown
![Escudo](logo.png){width=40%}
![Diagrama](osi.png){width=12cm}
```

El pie sale como pide APA 7 —«Figura 1» en negrita y debajo el título en
cursiva, los dos encima de la imagen— y la figura se queda **donde la
escribiste**, sin flotar a otra página.

Si una imagen no se encuentra o no se puede descargar, el PDF se genera igual:
en su lugar queda el texto alternativo y el comando avisa. pdflatex compone PNG,
JPG y PDF; un SVG hay que convertirlo antes.

`cache/` es una caché: se puede borrar entera y lo único que pasa es que la
siguiente vez se vuelva a descargar.

### Diagramas con Graphviz

Con Graphviz instalado, un bloque marcado como `dot` se dibuja en vectorial:

````markdown
```{.dot caption="Árbol binario de búsqueda"}
digraph {
  50 -> 30; 50 -> 70;
  30 -> 20; 30 -> 40;
}
```
````

El `caption` es opcional y pone el pie de figura. El resultado se guarda en
`cache/diagrams/`, así que un diagrama que no cambió no se vuelve a dibujar.
Si Graphviz no está instalado, el bloque se queda como código y el comando
avisa: el trabajo se genera igual.

### Gráficas con pgfplots

Un bloque marcado como `pgfplot` se dibuja dentro del propio PDF, vectorial y
con la misma tipografía que el texto:

````markdown
```{.pgfplot caption="Horas por fase del proyecto"}
\begin{axis}[ybar, ymin=0, ylabel={Horas},
             symbolic x coords={Análisis,Diseño,Código}, xtick=data]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180)};
\end{axis}
```
````

Salen barras (simples, agrupadas, apiladas, horizontales), líneas, dispersión
con recta de regresión, histogramas, caja y bigotes, barras de error, funciones
matemáticas, escalas logarítmicas y pastel. La paleta por omisión es en escala
de grises, para no salirse de APA. Un bloque `tikz` funciona igual y sirve para
dibujar cualquier otra cosa.

A diferencia de los diagramas `dot`, aquí **la sintaxis tiene que ser correcta**:
un error detiene la compilación y el comando indica la línea.
[`examples/catalog.md`](examples/catalog.md) trae una gráfica de cada tipo,
lista para copiar.

### Cajas destacadas

```markdown
::: nota
Lo que va aquí sale dentro de un recuadro con el título «Nota».
:::
```

Funcionan `nota`, `aviso`, `importante`, `ejemplo` y `definicion`. El título se
puede cambiar: `::: {.aviso title="Antes de entregar"}`.

### Diagramas de arte ASCII

Un esquema hecho con caracteres de dibujo de caja **dentro de un bloque de
código** sale compuesto con líneas de verdad, sin desalinearse:

```
  Requisitos
      └──► Diseño
              └──► Codificación
```

Fuera de un bloque de código, Markdown colapsa los espacios y el dibujo se
deshace.

### Símbolos y emoji

Puedes escribir `≠ ≤ ≥ ≈ ∈ ∉ ⊆ ∪ ∩ ∑ ∫ ∂ √ ∞ ∀ ∃ ⇒ ⇔ α β γ Δ Ω π μ σ ✓ ✗ ★`
directamente: la plantilla los tiene declarados. Los caracteres de dibujo de
caja y los emoji tampoco hay que declararlos.

Si usas un símbolo que la plantilla no conoce, **el PDF se genera igual**: en su
lugar aparece `[?]` y el comando te avisa:

```
Warning: The symbol ∮ (U+222E) could not be typeset and shows as [?] in the PDF.
Replace it in the Markdown or declare it in templates/common/investigacion.sty.
```

Para añadirlo de forma permanente, agrégalo al bloque `SÍMBOLOS UNICODE` de
`templates/common/investigacion.sty` siguiendo el patrón de los que ya están:

```latex
\newunicodechar{∮}{\ensuremath{\oint}}
```

### Comandos LaTeX

Por omisión el Markdown se lee **sin** `raw_tex`: una contrabarra suelta en el
texto corriente (por ejemplo la ruta `C:\Users\alumno`) se imprime tal cual en
lugar de enviarse a LaTeX como un comando inexistente. Esa es una causa habitual
de que la compilación falle sin motivo aparente.

Las fórmulas con `$...$` y `$$...$$` funcionan siempre, en los dos modos.

Si necesitas escribir LaTeX a propósito (`\newpage`, `\vspace`, un entorno
propio), agrega `--allow-latex`. Con esa opción, un error de sintaxis en tu
LaTeX sí detiene la compilación.

## Texto pegado de otras herramientas

Si copias el trabajo de Word, Notion, una página web o una IA, es probable que
traiga **espacios duros** invisibles: líneas que parecen vacías pero no lo son y
dos espacios al final de cada renglón. Con eso Pandoc no distingue los párrafos
y el PDF sale con los `###` a la vista, las tablas como filas de barras y el
índice con una sola entrada, todo ello **sin dar ningún error**.

El programa lo limpia solo al leer el archivo. Se recorta el final de cada línea
y se repara un marcador separado del texto por un espacio duro; el espacio duro
dentro de una frase se respeta y los bloques de código no se tocan.

En un documento limpio, en cambio, los dos espacios finales se conservan, porque
ahí sí son el salto de línea que define Markdown. Si el salto te importa, la
forma que nunca se pierde es `<br>`.

El archivo puede estar guardado en UTF-8 o en Windows-1252, para que sirvan
también documentos antiguos con acentos.

## Copias en otros directorios

`--output` define dónde se genera el PDF. `--copy` deja además una copia en
otro directorio, que se crea si no existe:

```bash
investigacion trabajo.md --title "Tema" --course "Materia" --teacher "Docente" \
  --copy ~/Documentos/Escuela/Economia
```

Puede repetirse para dejar varias copias, por ejemplo en una memoria USB y en
una carpeta sincronizada:

```bash
investigacion trabajo.md --title "Tema" --course "Materia" --teacher "Docente" \
  --copy /media/usb \
  --copy ~/Nextcloud/Tareas
```

Si un directorio de copia coincide con el de `--output`, se omite.

## Cuando algo falla

El comando muestra únicamente el error real de LaTeX, no todo el registro de
carga de paquetes, y guarda `last-error.tex` y `last-error.log` en el
directorio de salida.

| Síntoma | Causa más probable |
|---|---|
| El error apunta a una línea de tu texto | Una contrabarra suelta; ver [Comandos LaTeX](#comandos-latex) |
| El error apunta a una gráfica | Sintaxis de pgfplots; el mensaje indica la línea |
| Los `###` salen impresos en el PDF | Espacios invisibles; ver [Texto pegado](#texto-pegado-de-otras-herramientas) |
| Una tabla sale como filas de barras | Falta la fila de guiones debajo del encabezado |
| Un diagrama sale desalineado | Está fuera de un bloque de código |
| Aparece `[?]` en el PDF | Un símbolo que la plantilla no conoce; el comando dice cuál |

## Personalizar la plantilla

Las plantillas están repartidas en dos niveles:

- `templates/common/investigacion.sty` — el formato APA y el bloque con los
  paquetes y macros que **necesita la salida de Pandoc**. Lo comparten todas.
- `templates/common/investigacion-final.sty` — hyperref, las notas en tablas y
  los metadatos del PDF; va justo antes de `\begin{document}`.
- `templates/<nombre>/template.ltx` — solo los datos (`%%TITULO%%`…) y la
  portada.

**Para crear un diseño nuevo**, copia `templates/apa-simple/` a
`templates/mi-diseno/` y cambia solo la portada: aparece sola en el menú y se
usa con `--template mi-diseno`. Los `.sty` que pongas junto a tu
`template.ltx` también se copian al compilar.

El preámbulo común trae lo que la salida de Pandoc da por supuesto: `calc` y
`\newcounter{none}` para las tablas, `\pandocbounded` para las imágenes, los
comandos de resaltado de sintaxis para el código, `soul` y `ulem` para el
tachado y el resaltado, `amsmath` para las fórmulas. Si quitas alguno, los
documentos que usen ese elemento dejarán de compilar aunque el resto funcione.

Para comprobar qué espera tu versión de Pandoc:

```bash
pandoc trabajo.md -s --to=latex | sed -n '/documentclass/,/begin{document}/p'
```

Los datos de la portada llegan como marcadores `%%TITULO%%`, `%%ALUMNO%%`,
`%%INTEGRANTES%%`, `%%GRUPO%%`, `%%FECHA_ENTREGA%%`, etc., y el trabajo
convertido entra en `%%CONTENIDO_MARKDOWN%%`. Para probar cambios sin tocar el
original, usa `--template ruta/a/mi-copia.ltx`.

La regla de que los integrantes sustituyan al alumno vive en la plantilla, no en
el programa: es un `\ifdefempty{\ListaIntegrantes}` que en su rama vacía
comprueba `\NombreAlumno`. Si prefieres que salgan los dos, basta con separar
esos dos `\ifdefempty` en la portada.

### Los logos de la portada

El repositorio **no trae logos**: los de una institución rara vez son
redistribuibles, así que `templates/logos/` está en `.gitignore` y cada quien pone
los suyos. La plantilla busca dos nombres fijos:

| Archivo | Dónde sale | Altura a la que se escala |
|---|---|---|
| `logo-universidad.png` | Arriba a la izquierda | 1.6 cm |
| `logo-facultad.png` | Arriba a la derecha | 0.88 cm |

Los dos son opcionales: cada uno va envuelto en `\IfFileExists`, así que si
faltan la portada se compila igual, solo que sin ellos.

Deben ser PNG, preferiblemente con fondo transparente, y **oscuros o de color**:
la portada es blanca, así que un logo blanco sería invisible.

**Si prefieres tenerlos fuera del proyecto** —lo habitual, para no mezclarlos con
el código— indica la carpeta al generar:

```bash
investigacion trabajo.md --title "Tema" --course "M" --teacher "D" \
  --logos ~/Documentos/logos-de-mi-universidad
```

O déjalo fijo en el `.env` con `LOGOS="/ruta/a/la/carpeta"`, que es lo cómodo
cuando son siempre los mismos. Sin nada de eso se usa la carpeta `logos/` de la
plantilla, si existe, y si no la compartida `templates/logos/`.

Si quedan descolocados, las posiciones y alturas están en el bloque `LOGOS` de
`templates/apa/template.ltx`: son dos nodos de TikZ con su `xshift`, `yshift` y `height`.

Por dentro esto funciona así: pdflatex se ejecuta con el directorio del Markdown
como directorio de trabajo, no con el de la plantilla, de modo que una ruta
relativa nunca encontraría los archivos. En vez de eso, el generador copia la
carpeta de logos junto al `.tex` temporal y añade ese directorio al inicio de
`TEXINPUTS`; por eso la plantilla puede escribir solo `{logo-universidad}`, sin
ruta ni extensión.

## Pruebas

```bash
cargo test                       # todas
cargo test --lib markdown        # solo las de un módulo
cargo test --test pdf            # solo las que generan PDF reales
```

Las que generan un PDF real o llaman a Pandoc se omiten solas si Pandoc,
pdflatex o Graphviz no están instalados.
