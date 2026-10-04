# Installation

Version 2.0.0 supports **Linux**. Windows and macOS are planned for a later
update.

## What you need

| Tool | Required | What for |
|---|---|---|
| **Rust 1.88+** (installed with [rustup](https://rustup.rs)) | Yes | Building the program once |
| **Pandoc** | Yes | Converting the Markdown |
| **TeX** with `pdflatex` (TeX Live) | Yes | Typesetting the PDF |
| **Graphviz** | No | Drawing ` ```dot ` diagrams. Without it, diagrams stay as code and you get a warning |

## 1. Install the tools

Pick your system. After installing, **close and reopen the terminal** so the
new commands are found.

### Arch, Manjaro, EndeavourOS

```bash
sudo pacman -S rustup pandoc graphviz texlive-basic texlive-latexextra \
  texlive-fontsextra texlive-langspanish texlive-pictures texlive-plaingeneric
rustup default stable
```

### Debian, Ubuntu, Linux Mint

```bash
sudo apt install pandoc graphviz texlive-latex-recommended texlive-latex-extra \
  texlive-fonts-extra texlive-lang-spanish texlive-pictures texlive-plain-generic
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Do not use the `cargo` package from `apt`: it is too old for this program.

### Fedora

```bash
sudo dnf install pandoc graphviz texlive-scheme-medium \
  texlive-collection-latexextra texlive-collection-fontsextra texlive-collection-langspanish
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### openSUSE

```bash
sudo zypper install pandoc graphviz texlive-latex texlive-latexextra \
  texlive-fontsextra texlive-babel-spanish
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 2. Install the program

```bash
git clone https://github.com/PeterOlveraO/FormatoInvestigaciones.git
cd FormatoInvestigaciones
cargo install --path .
```

`cargo install` downloads the dependencies (internet is needed only this time),
builds the program and leaves `investigacion` in `~/.cargo/bin`, which rustup
adds to your `PATH`. There is no virtual environment to activate.

The program remembers where the project folder is. If you move the folder, run
`cargo install --path .` again, or set the `INVESTIGACION_HOME` variable to the
new location.

## 3. Configure

Run `investigacion`. The first time it asks for the language and opens the
**profile wizard**. The wizard asks for the format (APA 7, Harvard, MLA), the
design and all the cover data, and saves them in `courses/<name>.toml`. There
is nothing else to configure.

Optional:

- **Logos:** add them to `templates/logos/` (`logo-universidad.png`,
  `logo-facultad.png`).
- **`settings.toml`** is written by the menu: it remembers the language
  (`IDIOMA="es"`). You can add `LOGOS="<folder>"` for a logos folder outside
  the project. The cover data always goes in the profiles.

## 4. Check that it works

```bash
cargo --version && pandoc --version && pdflatex --version && dot -V
investigacion examples/catalog.md --title "Catalogo" -p example
```

Run the second command from the project folder. If it ends with `PDF
generated` and no warnings, nothing is missing. A warning names exactly what is
missing (a symbol, an image, Graphviz), and the PDF is still produced.

## Update

```bash
git pull
cargo install --path .
```

## Uninstall

```bash
cargo uninstall investigacion
```

Then delete the project folder.

## Installation problems

| Symptom | Fix |
|---|---|
| "The project folder (the one with templates/) was not found" | You moved or deleted the cloned folder after installing. Run `cargo install --path .` again from its new place, or set `INVESTIGACION_HOME=<folder>` |
| `investigacion: command not found` | Open a new terminal, or run `source ~/.cargo/env`. |
| A `ModuleNotFoundError` from Python appears | An old Python environment is still active: run `deactivate` and delete the `.venv/` folder |
| `cargo install` says the Rust version is too old | `rustup update` |
| LaTeX reports a missing `.sty` | Install the package with your TeX distribution (`tlmgr install <name>` or your distribution's package manager) |
| Diagrams come out as code | Install Graphviz and check `dot -V` |
