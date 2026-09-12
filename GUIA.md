# Guía de uso

Todo lo que el programa acepta, cómo se le pide y qué hace por dentro. Para una
presentación general del proyecto, ver [README.md](README.md); para lo que hay
que instalar en cada sistema operativo, [REQUISITOS.md](REQUISITOS.md).

## Índice

- [Cómo funciona](#cómo-funciona)
- [Configuración](#configuración)
- [Dónde van los archivos](#dónde-van-los-archivos)
- [El comando](#el-comando)
- [La portada: alumno, integrantes y grupo](#la-portada-alumno-integrantes-y-grupo)
- [Estructura del trabajo](#estructura-del-trabajo)
- [Qué puedes escribir en el Markdown](#qué-puedes-escribir-en-el-markdown)
- [Texto pegado de otras herramientas](#texto-pegado-de-otras-herramientas)
- [Copias en otros directorios](#copias-en-otros-directorios)
- [Cuando algo falla](#cuando-algo-falla)
- [Personalizar la plantilla](#personalizar-la-plantilla)

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
   ├─ 4. El fragmento se inserta en Latex/base.ltx, que aporta la portada,
   │     el formato APA y todo el preámbulo
   │
   ├─ 5. pdflatex compila en un directorio temporal, repitiendo las pasadas
   │     necesarias hasta que el índice se estabiliza
   │
   └─ 6. El PDF se copia a output/ y, si se pidió, a otros directorios
```

Lo importante de este diseño: **ningún paso intermedio deja archivos** en el
proyecto salvo la caché de `imagenes/`. Si algo falla en la compilación, se
guardan `ultimo-error.tex` y `ultimo-error.log` en el directorio de salida para
poder revisarlos.

El código vive en dos archivos: `src/investigacion/generator.py` tiene toda la
lógica y `src/investigacion/cli.py` solo traduce argumentos. Los filtros de
Pandoc están en `src/investigacion/filtros/`.

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

También puedes definirlas en el entorno del shell: si ya existen ahí, tienen
prioridad sobre `.env`. El archivo `.env` está en `.gitignore`, así que tus
datos no se suben a ningún lado.

## Dónde van los archivos

```
input/     tus trabajos en Markdown (ignorado por git)
output/    los PDF generados (ignorado por git)
imagenes/  cache de imagenes descargadas y diagramas (ignorado por git)
```

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
investigacion mi-trabajo.md --titulo "Ecuaciones diferenciales" --materia "Cálculo"
```

Solo el título y la materia son obligatorios. El resultado es
`output/ecuaciones-diferenciales.pdf`: el nombre sale del título, en minúsculas,
sin acentos y con guiones. Si vuelves a generar el mismo título, el PDF anterior
se reemplaza. La fecha de entrega es la del día en que ejecutas el comando.

### Opciones

| Opción | Obligatoria | Descripción |
|---|---|---|
| `markdown` | Sí | Archivo `.md` con el contenido del trabajo |
| `--titulo` | Sí | Título del trabajo; define el nombre del PDF |
| `--materia` | Sí | Nombre de la materia |
| `--docente` | No | Nombre del docente; respaldo en la variable `DOCENTE` |
| `--integrantes` | No | Nombres del equipo en un solo argumento, separados por comas o punto y coma |
| `--grupo` | No | Grupo de la materia; cierra el bloque de datos de la portada |
| `--salida` | No | Directorio donde se genera el PDF (por omisión `output`) |
| `--copia` | No | Directorio adicional para una copia; puede repetirse |
| `--permitir-latex` | No | Interpreta los comandos LaTeX escritos en el Markdown |
| `--env-file` | No | Archivo dotenv alternativo (por omisión `.env`) |
| `--plantilla` | No | Plantilla LaTeX alternativa a `Latex/base.ltx` |
| `--logos` | No | Carpeta con los logos de la portada |

`investigacion --help` muestra lo mismo desde la terminal.

### El orden de los argumentos no importa

El archivo `.md` puede ir en cualquier posición. Estas tres líneas hacen lo
mismo:

```bash
investigacion trabajo.md --titulo "Tema" --materia "Materia" --docente "Docente"
investigacion --titulo "Tema" --materia "Materia" --docente "Docente" trabajo.md
investigacion --docente "Docente" trabajo.md --materia "Materia" --titulo "Tema"
```

Lo único que importa es que cada valor con espacios vaya **entre comillas**. Sin
ellas el comando falla:

```bash
# Mal: "de" y "costos" se toman como argumentos sueltos
investigacion trabajo.md --titulo Conceptos de costos --materia "M" --docente "D"
```

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
individual a uno de equipo, basta con usar o no usar `--integrantes`.

| Qué usas | Qué sale en la portada |
|---|---|
| Nada | Solo `ALUMNO:`, con el nombre del `.env` |
| `--integrantes` | Solo `INTEGRANTES:`; `ALUMNO:` no aparece aunque esté en el `.env` |
| Ninguno de los dos | Ninguna de las dos líneas, sin dejar hueco |

**Trabajo individual.** El nombre sale de `ALUMNO` en el `.env`:

```
ALUMNO: Ana Ruiz Medina
MATERIA: Economía
```

**Trabajo en equipo.** Los nombres van en un solo argumento, separados por comas
(también valen los punto y coma):

```bash
investigacion trabajo.md --titulo "Tema" --materia "Materia" --docente "Docente" \
  --integrantes "Ana Ruiz, Luis Paz, Sofia Vela"
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
variable `INTEGRANTES`; `--integrantes` tiene prioridad cuando se usa.

El autor que se guarda en los metadatos del PDF sigue el mismo criterio: los
integrantes si los hay y, si no, el alumno.

**Grupo.** `--grupo` añade la última línea del bloque:

```
MATERIA: Economía
DOCENTE: Nombre del docente
SEMESTRE: 2026-2
GRUPO: 7-A
```

Si no lo indicas se lee `GRUPO` del `.env`, y si tampoco está ahí la línea no
aparece.

**Docente.** Funciona igual: `--docente` gana sobre la variable `DOCENTE` del
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
> [`ejemplo/catalogo.md`](ejemplo/catalogo.md) o abre
> [`ejemplo/catalogo.pdf`](ejemplo/catalogo.pdf).

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

La ruta se busca desde la carpeta del `.md` y desde `imagenes/` del proyecto,
así que una imagen que uses en varios trabajos basta con dejarla ahí una vez:

```markdown
![Escudo de la universidad](logo.png)
```

**También funcionan las imágenes de la web.** Se descargan la primera vez a
`imagenes/remotas/` y a partir de ahí el trabajo se genera sin internet:

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

`imagenes/` es una caché: se puede borrar entera y lo único que pasa es que la
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
`imagenes/diagramas/`, así que un diagrama que no cambió no se vuelve a dibujar.
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
[`ejemplo/catalogo.md`](ejemplo/catalogo.md) trae una gráfica de cada tipo,
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
Advertencia: El simbolo ∮ (U+222E) no se pudo componer y salio como [?] en el PDF.
Reemplazalo en el Markdown o declaralo en Latex/base.ltx.
```

Para añadirlo de forma permanente, agrégalo al bloque `SÍMBOLOS UNICODE` de
`Latex/base.ltx` siguiendo el patrón de los que ya están:

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
propio), agrega `--permitir-latex`. Con esa opción, un error de sintaxis en tu
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

`--salida` define dónde se genera el PDF. `--copia` deja además una copia en
otro directorio, que se crea si no existe:

```bash
investigacion trabajo.md --titulo "Tema" --materia "Materia" --docente "Docente" \
  --copia ~/Documentos/Escuela/Economia
```

Puede repetirse para dejar varias copias, por ejemplo en una memoria USB y en
una carpeta sincronizada:

```bash
investigacion trabajo.md --titulo "Tema" --materia "Materia" --docente "Docente" \
  --copia /media/usb \
  --copia ~/Nextcloud/Tareas
```

Si un directorio de copia coincide con el de `--salida`, se omite.

## Cuando algo falla

El comando muestra únicamente el error real de LaTeX, no todo el registro de
carga de paquetes, y guarda `ultimo-error.tex` y `ultimo-error.log` en el
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

`Latex/base.ltx` contiene la portada con TikZ, el formato APA y un bloque con
los paquetes y macros que **necesita la salida de Pandoc**: `calc` y
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
original, usa `--plantilla mi-copia.ltx`.

La regla de que los integrantes sustituyan al alumno vive en la plantilla, no en
el programa: es un `\ifdefempty{\ListaIntegrantes}` que en su rama vacía
comprueba `\NombreAlumno`. Si prefieres que salgan los dos, basta con separar
esos dos `\ifdefempty` en la portada.

### Los logos de la portada

El repositorio **no trae logos**: los de una institución rara vez son
redistribuibles, así que `Latex/logos/` está en `.gitignore` y cada quien pone
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
investigacion trabajo.md --titulo "Tema" --materia "M" --docente "D" \
  --logos ~/Documentos/logos-de-mi-universidad
```

O déjalo fijo en el `.env` con `LOGOS="/ruta/a/la/carpeta"`, que es lo cómodo
cuando son siempre los mismos.

Si quedan descolocados, las posiciones y alturas están en el bloque `LOGOS` de
`Latex/base.ltx`: son dos nodos de TikZ con su `xshift`, `yshift` y `height`.

Por dentro esto funciona así: pdflatex se ejecuta con el directorio del Markdown
como directorio de trabajo, no con el de la plantilla, de modo que una ruta
relativa nunca encontraría los archivos. En vez de eso, el generador copia la
carpeta de logos junto al `.tex` temporal y añade ese directorio al inicio de
`TEXINPUTS`; por eso la plantilla puede escribir solo `{logo-universidad}`, sin
ruta ni extensión.

## Pruebas

```bash
source .venv/bin/activate        # En Windows: .venv\Scripts\activate
python -m unittest discover -s tests
```

Hay que usar el Python del entorno virtual: las pruebas importan el paquete
instalado en modo editable. Para una sola clase:

```bash
python -m unittest tests.test_generator.CopyTests
```

Las que generan un PDF real se omiten solas si Pandoc, pdflatex o Graphviz no
están instalados.
