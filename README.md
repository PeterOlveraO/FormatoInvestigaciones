# Formato de Investigaciones

Escribe tu trabajo escolar en Markdown y obtén un PDF con formato APA 7:
portada, índice, encabezados, tablas, diagramas y gráficas, sin tocar Word ni
pelearte con LaTeX.

```
mi-trabajo.md  →  Pandoc  →  plantilla LaTeX  →  pdflatex  →  output/mi-trabajo.pdf
```

Tú escribes solo el contenido. La portada, la numeración, el índice, el
interlineado doble, la sangría y los pies de figura los pone el programa.

| | |
|---|---|
| ![Portada generada](docs/imagenes/portada.png) | ![Página con un diagrama](docs/imagenes/diagrama.png) |

## Qué sabe hacer

- **Markdown completo.** Toda la sintaxis básica y extendida: tablas, notas al
  pie, listas de tareas, listas de definición, tachado, resaltado, subíndices,
  emoji a color y el HTML en línea que Markdown permite intercalar.
- **Portada APA con tus datos**, individual o de equipo, leídos de un archivo de
  configuración para no repetirlos en cada trabajo.
- **Diagramas de verdad.** Un bloque ```` ```dot ```` se dibuja con Graphviz:
  árboles, flujos, autómatas, modelos entidad-relación.
- **Gráficas de datos.** Un bloque ```` ```pgfplot ```` se calcula y se dibuja
  dentro del propio PDF: barras, líneas, dispersión con regresión, histogramas,
  caja y bigotes, pastel.
- **Imágenes locales y de internet.** Las de la web se descargan solas la
  primera vez y quedan guardadas para trabajar sin conexión.
- **Arte ASCII alineado.** Los diagramas hechos con caracteres de dibujo
  (`└ ─ ┼ │`) se componen como líneas reales.
- **Nunca te deja sin PDF.** Si falta un símbolo, una imagen no se descarga o
  Graphviz no está instalado, el trabajo se genera igual y el comando te avisa
  en la terminal de qué pasó.

![Página con gráficas](docs/imagenes/graficas.png)

## Requisitos

Python 3.10 o superior, **Pandoc** y una distribución de **TeX** con `pdflatex`.
Graphviz es opcional y solo hace falta para los diagramas. Funciona en Linux,
Windows y macOS.

```bash
# Arch o Manjaro
sudo pacman -S python pandoc graphviz texlive-basic texlive-latexextra \
  texlive-fontsextra texlive-langspanish texlive-pictures texlive-plaingeneric

# Debian o Ubuntu
sudo apt install python3 python3-venv pandoc graphviz \
  texlive-latex-recommended texlive-latex-extra texlive-fonts-extra \
  texlive-lang-spanish texlive-pictures texlive-plain-generic

# Windows
winget install Python.Python.3.12 JohnMacFarlane.Pandoc Graphviz.Graphviz MiKTeX.MiKTeX

# macOS
brew install python pandoc graphviz && brew install --cask mactex-no-gui
```

Las instrucciones completas, con Fedora, openSUSE, la lista de paquetes de LaTeX
y cómo comprobar que no falta nada, están en **[REQUISITOS.md](REQUISITOS.md)**.

El paquete de Python **no tiene dependencias**: solo biblioteca estándar.

## Instalación

```bash
git clone <url-del-repositorio>
cd FormatoInvestigaciones
python -m venv .venv
source .venv/bin/activate        # En Windows: .venv\Scripts\activate
python -m pip install -e .
cp .env.example .env             # En Windows: copy .env.example .env
```

Edita `.env` con los datos que no cambian entre trabajos (universidad, facultad,
semestre y tu nombre).

### Los logos de tu institución

El repositorio **no incluye ningún logo**: los de una universidad rara vez son
redistribuibles. Deja los tuyos en `Latex/logos/` con estos nombres:

| Archivo | Dónde sale |
|---|---|
| `logo-universidad.png` | Arriba a la izquierda de la portada |
| `logo-facultad.png` | Arriba a la derecha |

Si prefieres tenerlos fuera del proyecto, indica la carpeta con `--logos` o deja
la ruta fija en la variable `LOGOS` del `.env`. Los dos archivos son opcionales:
sin ellos la portada se genera igual, solo que sin logos. Más detalles en
[`Latex/logos/LEEME.md`](Latex/logos/LEEME.md).

## Inicio rápido

```bash
# 1. Parte del esqueleto incluido
cp ejemplo/investigacion.md input/mi-trabajo.md

# 2. Escribe el contenido en Markdown

# 3. Genera el PDF
investigacion mi-trabajo.md --titulo "Ecuaciones diferenciales" --materia "Cálculo"
```

El resultado queda en `output/ecuaciones-diferenciales.pdf`.

Solo **el título y la materia** son obligatorios; el resto de los datos son
opcionales o salen del `.env`. Y no hace falta escribir la carpeta: los trabajos
viven en `input/` y el comando los busca ahí solo, incluso dentro de subcarpetas
por materia:

```
input/
  IS/Actividad1.md   →  investigacion Actividad1.md
  Economia.md        →  investigacion Economia.md
```

## El catálogo

[`ejemplo/catalogo.md`](ejemplo/catalogo.md) reúne un ejemplo de **cada cosa que
el programa sabe componer**: los cinco niveles de encabezado APA, las cuatro
clases de lista, cinco variantes de tabla, siete diagramas de Graphviz, doce
gráficas y las fórmulas, cada uno con el código que lo produce.

El PDF que genera está en el repositorio para verlo sin instalar nada:
[`ejemplo/catalogo.pdf`](ejemplo/catalogo.pdf) (23 páginas).

```bash
investigacion ejemplo/catalogo.md --titulo "Catalogo" --materia "Ejemplo" --docente "Ejemplo"
```

## Pedirle el trabajo a una IA

**[PROMPT-IA.md](PROMPT-IA.md)** trae un prompt listo para copiar que le explica
al modelo todo lo que el generador sabe componer: la estructura que debe seguir,
la sintaxis completa, cómo escribir un diagrama de Graphviz, los doce tipos de
gráfica y cómo dejar las referencias en APA.

Pegas el prompt, añades tu tema, guardas la respuesta en `input/` y generas el
PDF. El resultado aprovecha tablas, diagramas y gráficas en vez de quedarse en
párrafos sueltos.

## Documentación

- **[REQUISITOS.md](REQUISITOS.md)** — qué instalar en cada sistema operativo y
  cómo comprobar que no falta nada.
- **[GUIA.md](GUIA.md)** — cómo funciona por dentro, todas las opciones del
  comando, la sintaxis completa que acepta y qué hacer cuando algo falla.
- **[PROMPT-IA.md](PROMPT-IA.md)** — el prompt para que una IA te redacte el
  trabajo en este formato.
- **[CLAUDE.md](CLAUDE.md)** — notas de arquitectura para quien vaya a tocar el
  código: los contratos entre las piezas y las trampas encontradas.

## Ejemplos incluidos

| Archivo | Qué es |
|---|---|
| `ejemplo/investigacion.md` | Esqueleto vacío para empezar un trabajo |
| `ejemplo/sintaxis.md` | Referencia breve de la sintaxis de Markdown |
| `ejemplo/catalogo.md` | Muestrario completo de todos los elementos |
| `ejemplo/arboles-binarios.md` | Un trabajo real de ejemplo, con diagramas |

Tus propios trabajos van en `input/`, que está en `.gitignore`: no se suben al
repositorio.

## Pruebas

```bash
source .venv/bin/activate        # En Windows: .venv\Scripts\activate
python -m unittest discover -s tests
```

Las que generan un PDF real se omiten solas si Pandoc, pdflatex o Graphviz no
están instalados.

## Licencia

[GPL-3.0-or-later](LICENSE). Puedes usarlo, estudiarlo, modificarlo y
distribuirlo; si distribuyes una versión modificada, tiene que seguir siendo
libre bajo la misma licencia.
