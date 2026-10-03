# Requisitos e instalación

El programa es un binario de Rust que se compila una vez con `cargo`, y
necesita tres herramientas externas para hacer su trabajo. Funciona en **Linux, Windows y
macOS**: no usa nada específico de un sistema operativo.

## Qué hace falta

| Herramienta | ¿Obligatoria? | Para qué | Si falta |
|---|---|---|---|
| **Rust** (`cargo`, vía [rustup](https://rustup.rs)) | Sí, para compilarlo | Compilar el programa una vez | No se puede instalar |
| **Pandoc** | Sí | Convertir el Markdown a LaTeX | Error claro al generar |
| **Una distribución de TeX** con `pdflatex` | Sí | Componer el PDF | Error claro al generar |
| **Graphviz** | No | Dibujar los bloques ```` ```dot ```` | El diagrama se queda como código y el comando avisa |

Los paquetes de LaTeX que usa la plantilla también son opcionales uno por uno:
si falta alguno se pierde ese detalle concreto (los emoji, el fondo del código,
los caracteres de dibujo…) pero el PDF se genera igual.

## Linux

### Rust (todas las distribuciones)

El programa necesita Rust 1.88 o posterior. Los paquetes `cargo` de Debian y
Ubuntu suelen ser más viejos, así que conviene instalarlo con **rustup**:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Arch, Manjaro, EndeavourOS

```bash
sudo pacman -S pandoc graphviz \
  texlive-basic texlive-latexextra texlive-fontsextra \
  texlive-langspanish texlive-pictures texlive-plaingeneric
```

### Debian, Ubuntu, Linux Mint

```bash
sudo apt update
sudo apt install pandoc graphviz \
  texlive-latex-recommended texlive-latex-extra texlive-fonts-extra \
  texlive-lang-spanish texlive-pictures texlive-plain-generic
```

### Fedora

```bash
sudo dnf install pandoc graphviz \
  texlive-scheme-medium texlive-collection-latexextra \
  texlive-collection-fontsextra texlive-collection-langspanish
```

### openSUSE

```bash
sudo zypper install pandoc graphviz \
  texlive-latex texlive-latexextra texlive-fontsextra texlive-babel-spanish
```

## Windows

La forma más corta es con `winget`, que ya viene en Windows 10 y 11:

```powershell
winget install Rustlang.Rustup
winget install JohnMacFarlane.Pandoc
winget install Graphviz.Graphviz
winget install MiKTeX.MiKTeX
```

Con **MiKTeX** no hay que elegir paquetes de LaTeX: la primera vez que generes
un PDF te irá pidiendo permiso para descargar los que falten, o los instalará
solo si marcas «Install missing packages on-the-fly: Yes» en el MiKTeX Console.
La alternativa es [TeX Live](https://tug.org/texlive/windows.html), que instala
todo de una vez y ocupa varios gigabytes.

Después de instalar, **cierra y vuelve a abrir la terminal** para que se
actualice el `PATH`.

La instalación del proyecto es la misma que en Linux:

```powershell
cargo install --path .
copy .env.example .env
```

`rustup` puede pedir las «Build Tools» de Visual Studio la primera vez; acepta
la instalación que propone.

## macOS

Con [Homebrew](https://brew.sh):

```bash
brew install rustup pandoc graphviz
rustup-init -y
brew install --cask mactex-no-gui
```

`mactex-no-gui` ocupa unos 5 GB y trae todo lo necesario. Si prefieres algo más
ligero, instala `basictex` y añade los paquetes a mano:

```bash
brew install --cask basictex
sudo tlmgr update --self
sudo tlmgr install newtx pgfplots pgf-pie twemojis pmboxdraw floatrow \
  newunicodechar ulem framed footnotehyper xurl titlesec ragged2e \
  babel-spanish
```

## Comprobar que todo está

```bash
cargo --version
pandoc --version
pdflatex --version
dot -V               # opcional
```

Y la prueba de fuego, que usa absolutamente todo:

```bash
investigacion ejemplo/catalogo.md --title "Catalogo" --subject "Prueba"
```

Si termina sin advertencias, no falta nada. Si avisa de algo —un símbolo, una
imagen, un diagrama— el mensaje dice exactamente qué es y el PDF se genera de
todas formas.

## Los paquetes de LaTeX, uno por uno

Solo hace falta mirar esta tabla si tienes una instalación de TeX mínima y
quieres añadir lo justo. Todos están en CTAN y se instalan con
`tlmgr install <nombre>` (TeX Live) o desde el MiKTeX Console.

| Paquete | Para qué |
|---|---|
| `babel` + `babel-spanish` | Idioma español, títulos y partición de palabras |
| `newtx` | Tipografía tipo Times, que es la que pide APA |
| `geometry`, `setspace`, `titlesec`, `ragged2e`, `fancyhdr`, `indentfirst` | Márgenes, interlineado y encabezados APA |
| `pgf` / `tikz` | La portada |
| `pgfplots`, `pgfplotstable`, `pgf-pie` | Las gráficas de datos |
| `graphicx`, `caption`, `floatrow` | Imágenes y pies de figura |
| `longtable`, `booktabs`, `array`, `calc` | Tablas |
| `fancyvrb`, `framed`, `upquote` | Bloques de código y cajas de nota |
| `soul`, `ulem` | Tachado, subrayado y resaltado |
| `amsmath`, `amssymb` | Fórmulas |
| `newunicodechar`, `pmboxdraw`, `twemojis` | Símbolos, arte ASCII y emoji |
| `hyperref`, `bookmark`, `xurl`, `footnotehyper` | Enlaces, marcadores y notas al pie en tablas |
| `etoolbox` | Parches internos de la plantilla |

## Notas

- **El PDF sale igual en los tres sistemas.** Lo que compone el documento es
  pdflatex con la misma plantilla, así que no depende del sistema operativo.
- **Las rutas con espacios funcionan** (`C:\Users\Juan Pérez\...`), igual que
  los acentos en los nombres de archivo.
- El proyecto se desarrolló y se prueba en Linux. El código no contiene nada
  específico de un sistema —las rutas, los separadores y las codificaciones se
  resuelven con la biblioteca estándar de Rust, y el menú usa crossterm, que
  funciona igual en las tres consolas— pero si encuentras algo que falle en
  Windows o macOS, es un fallo que merece la pena reportar.
