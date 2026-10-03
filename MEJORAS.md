# Mejoras de la rama `mejoras-rust`

Bitácora de lo que se hizo mientras no estabas: qué cambió, en qué orden, qué
decidí por mi cuenta y qué conviene que revises. Todo está en la rama
`mejoras-rust` (subida a GitHub); `main` no se tocó.

## Resumen

| # | Pedido | Resultado |
|---|---|---|
| 1 | Pasarlo a Rust para reducir tiempos | Port completo a Rust. Regenerar un trabajo tarda **hasta 64 % menos** |
| 2 | Mejor organización de carpetas y acceso fácil | `templates/`, `subjects/`, `resources/`, `cache/`; la tecla `f` del menú abre cualquier carpeta |
| 3 | Elegir archivos de una lista, sin escribir rutas | Selector navegable con flechas y filtro al escribir |
| 4 | Varias plantillas y datos de cada materia, con el mismo selector | Plantillas `apa` y `apa-simple` + perfiles de materia en `subjects/*.toml` |
| 5 | Nombre del archivo distinto del título | El PDF se llama como el Markdown (o `--file-name`); el título solo va en la portada |
| 6 | Quitar las opciones 9 a 12 de la TUI | Fuera `.env`, «permitir LaTeX», plantilla por ruta y logos (siguen en el CLI) |

Además, por tu indicación: **el proyecto está en inglés** (código, carpetas,
opciones y mensajes) y **los comentarios en español**, uno breve por bloque.

## Orden en que se hizo y por qué

1. **Medir antes de tocar nada** (línea base con la versión de Python).
2. **Reorganizar carpetas primero**, todavía en Python: así el port a Rust se
   escribió una sola vez contra la estructura final. Comprobé que el PDF del
   catálogo salía con el texto idéntico antes de seguir.
3. **Port a Rust** con las pruebas de Python traducidas una a una; comparé de
   nuevo el texto de los PDF contra la línea base.
4. **Nombre del PDF ≠ título** (cambio pequeño, sobre el código ya portado).
5. **Plantillas y perfiles de materia**: la TUI los necesitaba para sus listas.
6. **TUI nueva**, al final, porque usa todo lo anterior.
7. **Documentación**.

Cada paso terminó con `cargo test`, `cargo clippy` sin avisos y una generación
real; cada uno es un commit propio:

```
7ffe200 Add text menu TUI (tui.py) as baseline          ← tu tui.py, que no estaba en git
12e22bd Reorganize folders: templates/, resources/filters/, cache/
b01183d Port the generator and CLI to Rust
3e18856 Remove the Python implementation
32945a2 Name the PDF independently of the title
444799c Add selectable templates and subject profiles
7dbc2d6 Replace the text menu with a full-screen TUI
(último) Update documentation for the Rust version
```

## 1. Rust y los tiempos

**Lo importante primero:** casi todo el tiempo lo gasta **pdflatex**, no el
programa. Medido en el catálogo (23 páginas): Pandoc 0.27 s y cada pasada de
pdflatex ~1.7 s. Por eso el port a Rust, por sí solo, no cambió nada (5.5 s →
5.5 s). Lo que sí funcionó fue atacar las pasadas de LaTeX:

- **Se recuerda el índice de la vez anterior.** El `.aux` y el `.toc` se guardan
  en `cache/latex/`. Al regenerar un trabajo, pdflatex ya parte del índice
  correcto y, si no cambió la estructura, basta **una pasada en vez de 2 o 3**.
  Si ese estado viejo diera problemas, se descarta solo y se compila desde cero.
- **La primera pasada usa `-draftmode`** cuando no hay estado previo (no escribe
  el PDF, que nunca podría ser el definitivo).

| Documento | Python | Rust, 1.ª vez | Rust, al regenerar |
|---|---|---|---|
| `ejemplo/catalogo.md` (23 págs.) | 5.5 s | 5.5 s (3 pasadas) | **2.0 s** (1 pasada) |
| `ejemplo/arboles-binarios.md` (12 págs.) | 1.85 s | 1.9 s (2 pasadas) | **1.1 s** (1 pasada) |

Regenerar es justo el caso de todos los días (corriges algo y vuelves a
generar), así que ahí está la ganancia real. Comprobé que un trabajo editado
(con una sección nueva que mueve todas las páginas) da exactamente el mismo
PDF por la vía rápida que desde cero.

Para ver dónde se va el tiempo: `INVESTIGACION_TIMING=1 investigacion ...`.

Otras ventajas del port: un solo binario sin entorno virtual, los filtros Lua
van dentro del binario, y el programa funciona desde cualquier carpeta (ya no
hace falta estar dentro del proyecto).

**Por qué todo el proyecto y no solo una parte:** mantener la lógica en dos
lenguajes habría duplicado cada arreglo. Las ~96 pruebas de Python se tradujeron
a Rust (ahora son 79) y el código de Python se borró en su propio commit; sigue
disponible en `main`.

## 2. Carpetas

```
input/       tus trabajos (sin cambios; tus subcarpetas IA/, IS/... siguen igual)
output/      PDFs (sin cambios; con un perfil van a output/<carpeta>/)
subjects/    NUEVO: perfiles de materia
templates/   antes Latex/: apa/, apa-simple/, common/ (preámbulo común) y logos/
resources/   los filtros Lua (antes src/investigacion/filtros/)
cache/       antes imagenes/: remote/, diagrams/ y latex/ (todo borrable)
src/, tests/ el código Rust y sus pruebas
```

- **Moví tus archivos locales** que git no ve: los logos de `Latex/logos/` a
  `templates/logos/` y la caché de `imagenes/` a `cache/`. Tus logos siguen
  saliendo en la portada (lo verifiqué).
- `base.ltx` se partió en `templates/common/investigacion.sty` (todo lo que
  comparten las plantillas: formato APA, compatibilidad con Pandoc, símbolos,
  emoji, gráficas) e `investigacion-final.sty` (hyperref). Cada plantilla queda
  en ~200 líneas y solo diseña su portada.
- **Acceso fácil:** en el menú, `f` abre `input`, `output`, `subjects`,
  `templates`, `cache` o la raíz del proyecto en el explorador de archivos, y
  `o` abre el último PDF generado.

## 3. Selección de archivos sin escribir rutas

En el menú, el campo «Markdown file» abre una lista: flechas para moverte,
Enter (o →) para entrar en una carpeta o elegir, Retroceso (o ←) para subir, y
escribir filtra la lista. Si elegiste un perfil, la lista empieza directamente
en la carpeta de esa materia (`input/IA/`, por ejemplo).

## 4. Plantillas y datos de cada materia

**Plantillas:** `apa` (tu portada geométrica de siempre) y `apa-simple`
(clásica, centrada, sin adornos). Se eligen con la misma lista en el menú o con
`--template apa-simple`. Una prueba compila **todas** las plantillas con y sin
los datos opcionales. Para crear otra: copia `templates/apa-simple/` con otro
nombre y cambia solo la portada; aparece sola en el menú.

**Perfiles de materia** (`subjects/<clave>.toml`):

```toml
subject = "Inteligencia artificial"
teacher = "..."
group = "M"
template = "apa"
folder = "IA"     # busca en input/IA y guarda en output/IA
```

En el menú se eligen de una lista y rellenan materia, docente, grupo, plantilla
y carpeta de salida. En el CLI: `investigacion Tarea1.md -p ia --title "..."`.
Prioridad: lo que escribas > perfil > `.env`.

**Ya te dejé creados cinco perfiles** con los datos que encontré en las
portadas de tus PDF de `output/`:

| Perfil | Materia | Docente | Grupo | Carpeta |
|---|---|---|---|---|
| `ia` | Inteligencia artificial | Martinez Ponce Mario Alberto | M | IA |
| `mi` | Metodos de investigacion | Rolon Aguilar Elvira | 7M | MI |
| `ie` | Ingeniería Económica | Hernandez Marin Juan Carlos | — | IE |
| `is` | Ingeniería de Software | Garcia Eliseo Luis Armando | — | IS |
| `pm` | Programación de Microprocesadores | Martinez Ponce Mario Alberto | M | PM |

Revísalos: en PM aparecían dos grupos (`7K` y `M`) y tomé el del PDF más
reciente; en IE e IS no había grupo. **Estos archivos no se suben a GitHub**
(`subjects/` está en `.gitignore` salvo `example.toml`), igual que tu `.env`.

## 5. Nombre del archivo ≠ título

Antes el PDF se llamaba como el título (`p-vs-np-el-problema-del-milenio.pdf`).
Ahora se llama como el Markdown (`Tarea2-3.md` → `tarea2-3.pdf`) y el título
solo va en la portada. Para otro nombre: `--file-name entrega-final` o el campo
«PDF file name» del menú, que se rellena solo al elegir el Markdown.

## 6. La TUI nueva

Se abre con `investigacion` sin argumentos (o `investigacion-tui`, como antes).
Es a pantalla completa, con ayuda del campo seleccionado y un panel de
resultados. Teclas: ↑↓ moverse, Enter editar/elegir, Supr vaciar, `g` generar,
`o` abrir PDF, `f` carpetas, `q` salir.

Campos: perfil, Markdown, título, nombre del PDF, materia, docente, integrantes,
grupo, plantilla, carpeta de salida y copias. **Quitadas** las antiguas 9–12
(`.env`, permitir LaTeX, plantilla por ruta, logos); siguen como opciones del
CLI (`--env-file`, `--allow-latex`, `--template ruta.ltx`, `--logos`).

La probé de verdad en una terminal simulada (130 y 80 columnas): elegir perfil,
navegar a `input/IA/Tarea2-3.md`, escribir el título y generar. Eso destapó dos
fallos que corregí: escribir `ia` en el filtro elegía el perfil `example`
(porque su materia dice «mater**ia**»), y en 80 columnas el panel de resultados
quedaba sin espacio.

## Decisiones que tomé sin poder preguntarte

- **Opciones del CLI en inglés** (`--title`, `--subject`, `--teacher`…), pero
  las de antes en español **siguen funcionando** como alias ocultos: tus
  comandos guardados no se rompen.
- **Mensajes del programa en inglés**, por la indicación de «proyecto en
  inglés». La portada y todo lo que sale en el PDF sigue en español, igual que
  la documentación (README, GUIA, este archivo).
- `ultimo-error.log` ahora se llama `last-error.log`.
- `output/` por omisión es el del proyecto, no el de la carpeta donde estés.
- Los nombres de las variables del `.env` no cambiaron (`UNIVERSIDAD`,
  `DOCENTE`…); se aceptan además en inglés (`UNIVERSITY`, `TEACHER`…).
- Los nombres de entornos de LaTeX (`CajaMarcada`, `ReferenciasAPA`) quedaron en
  español: renombrarlos solo arriesgaba romper la plantilla sin ganar nada.

## Qué conviene que revises

1. **Instalar:** `cargo install --path .` (deja `investigacion` en
   `~/.cargo/bin`). No lo ejecuté para no tocar tu `PATH`. La carpeta `.venv/`
   de Python ya no hace falta y se puede borrar.
2. **Los perfiles** de `subjects/` (sobre todo el grupo de PM).
3. **Windows y macOS:** el código no tiene nada específico de un sistema y la
   TUI usa crossterm, pero solo lo probé en Linux.
4. En Debian/Ubuntu el `cargo` de `apt` es demasiado viejo (hace falta Rust
   1.88+): usa rustup, como explica REQUISITOS.md.
5. Si te convence, abre el PR de `mejoras-rust` hacia `main`.

## Ideas que dejé fuera

- Precompilar el preámbulo de LaTeX como formato (`mylatexformat`) podría
  ahorrar otro ~40 % por pasada, pero es frágil con `babel` e `hyperref`.
- Que el selector de carpeta de salida también fuera una lista.
- Crear un perfil de materia desde el propio menú.
