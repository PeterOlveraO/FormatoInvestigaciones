# Formato de Investigaciones

[![Version](https://img.shields.io/badge/version-2.0.0-blue)](CHANGELOG.md)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-green)](LICENSE)
[![Rust 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange)](https://rustup.rs)
![Linux | Windows](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-lightgrey)
![Español | English](https://img.shields.io/badge/interface-Espa%C3%B1ol%20%7C%20English-informational)

Write your school paper in Markdown and get a finished PDF: cover page, table
of contents, headings, tables, diagrams and references in **APA 7, Harvard or
MLA 9**. No Word, no hand-written LaTeX.

```
paper.md  →  investigacion  →  output/paper.pdf
```

| | |
|---|---|
| ![Generated cover](docs/images/cover.png) | ![Page with a diagram](docs/images/diagram.png) |

The menu and messages are in **Spanish or English**. Papers are Spanish by
default and can be English.

## How it works

- **Format** = the norm (fonts, spacing, headings, references): `apa7`,
  `harvard`, `mla`.
- **Design** = the cover and the look: `geometric-cover`, `classic-cover`,
  `report`, `starter`, or one you make yourself.
- **Profile** = one per course, with your data (university, name, teacher,
  group…). The program asks for it once and fills every cover with it.

You only write the content. The program adds the cover, the table of contents
and the formatting.

## Install

You need [Rust](https://rustup.rs), [Pandoc](https://pandoc.org) and a TeX
distribution with `pdflatex` ([Graphviz](https://graphviz.org) is optional, for
diagrams). Works on **Linux and Windows**; macOS will come in a later update.
Step by step: **[INSTALL.md](INSTALL.md)**.

```bash
git clone https://github.com/PeterOlveraO/FormatoInvestigaciones.git
cd FormatoInvestigaciones
cargo install --path .
```

Keep the cloned folder: it holds the formats, designs, profiles and papers.

## Use it

1. Run `investigacion`. The first time it asks for the language and creates
   your first profile.
2. Put your paper in `input/` (for example `input/IA/Tarea1.md`).
3. On the home screen choose **Generate a PDF**, pick the Markdown, write the
   title and press `g`. The PDF lands in `output/`.

Or from the command line:

```bash
investigacion Tarea1.md -p ia --title "Búsqueda heurística"
```

Don't want to write the Markdown yourself? Give an AI the prompt in
[docs/prompts/paper.md](docs/prompts/paper.md) along with your topic.

## Documentation

| For | Read |
|---|---|
| Installing on each system | [INSTALL.md](INSTALL.md) |
| Using the menu and the basic commands | [docs/usage.md](docs/usage.md) |
| Every command-line option | [docs/cli.md](docs/cli.md) |
| How everything works (formats, designs, profiles, syntax, troubleshooting) | [docs/reference.md](docs/reference.md) |
| Prompt for an AI to write the paper | [docs/prompts/paper.md](docs/prompts/paper.md) |
| Prompt for an AI to turn a cover idea into a design | [docs/prompts/design.md](docs/prompts/design.md) |
| AI assistants reading this repository | [llms.txt](llms.txt), [AGENTS.md](AGENTS.md) |
| What changed | [CHANGELOG.md](CHANGELOG.md) |

`examples/` has sample papers and [`catalog.pdf`](examples/catalog.pdf), which
shows every supported element.

## Privacy

Your data never goes into git: profiles (`courses/*.toml`), papers (`input/`),
PDFs (`output/`), logos, `settings.toml` and the log (`cache/`) are ignored.

## License

[GPL-3.0-or-later](LICENSE).
