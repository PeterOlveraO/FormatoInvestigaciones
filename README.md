# Formato de Investigaciones

Write your school paper in Markdown and get a PDF in APA 7 format (cover page,
table of contents, headings, tables, diagrams and charts) without Word or
hand-written LaTeX.

```
paper.md  →  Pandoc  →  LaTeX template  →  pdflatex  →  output/paper.pdf
```

| | |
|---|---|
| ![Generated cover](docs/images/cover.png) | ![Page with a diagram](docs/images/diagram.png) |

The program's menu, messages and help are in **Spanish or English**, and you
choose the language the first time you open it. The paper itself is always
produced in Spanish APA format.

## What it does

- **Full Markdown**, basic and extended: tables, footnotes, task lists,
  definition lists, highlight, sub/superscript, emoji and inline HTML.
- **APA cover page** with your data, for individual or team papers, with
  optional logos and two designs (`apa`, `apa-simple`).
- **Diagrams and charts** drawn into the PDF: Graphviz (` ```dot `) and pgfplots
  (` ```pgfplot `).
- **Images** from disk or from the web. Web images are downloaded once and
  cached.
- **Always produces a PDF.** A missing symbol, image or Graphviz gives a
  warning instead of a failure.
- **Fast re-runs.** Generating the same paper again usually takes a single
  LaTeX pass.

![Page with charts](docs/images/charts.png)

## Requirements

[Rust](https://rustup.rs) 1.88+, [Pandoc](https://pandoc.org) and a TeX
distribution with `pdflatex`. Graphviz is optional and only needed for diagrams.
It works on Linux, Windows and macOS. The commands for each system are in
**[INSTALL.md](INSTALL.md)**.

## Install

```bash
git clone -b mejoras-rust https://github.com/PeterOlveraO/FormatoInvestigaciones.git
cd FormatoInvestigaciones
cargo install --path .        # puts `investigacion` in ~/.cargo/bin
cp .env.example .env          # Windows: copy .env.example .env
```

Keep the folder where you cloned it: the program uses it to find templates,
profiles and your papers.

## Configure (once)

1. **`.env`**: your university, faculty, semester and name. The comments in the
   file explain each line.
2. **Logos (optional)**: put `logo-universidad.png` and `logo-facultad.png` in
   `templates/logos/`. They are not in the repository because they are rarely
   redistributable.
3. **Course profiles (optional)**: copy `courses/example.toml` to, for example,
   `courses/ia.toml` and fill in the course name, teacher, group, template and
   folder. You enter them once and reuse them in every paper.

## Use

Put your papers in `input/`, one subfolder per course if you like (`input/IA/`,
`input/PM/`…).

**Interactive menu:** run `investigacion` with no arguments.

| Key | Action |
|---|---|
| ↑ ↓, Enter | Move and edit; fields marked ▸ open a list (profile, Markdown, template) |
| `g` | Generate the PDF |
| `v` | View the last PDF |
| `c` / `f` | Open a project folder |
| `l` | Switch language (Español / English) |
| `s` / `q` | Quit |

**Command line:**

```bash
investigacion Tarea1.md -p ia --title "Búsqueda heurística"
investigacion paper.md --title "Ecuaciones diferenciales" --course "Cálculo"
```

The PDF is named after the Markdown file (`Tarea1.md` → `tarea1.pdf`), not
after the title. Use `--file-name` to choose another name.

## Documentation

- **[INSTALL.md](INSTALL.md)**: installation per operating system, updating and
  installation problems.
- **[GUIDE.md](GUIDE.md)**: every option, course profiles, templates, the
  Markdown syntax and what to do when something fails.
- **[AI-PROMPT.md](AI-PROMPT.md)**: a ready-to-paste prompt so an AI writes the
  paper in this format.
- **[CHANGELOG.md](CHANGELOG.md)**: what changed between versions.
- `examples/`: a paper template, a syntax reference, a full example paper and
  the catalog of every element, also as
  [`examples/catalog.pdf`](examples/catalog.pdf).

## License

[GPL-3.0-or-later](LICENSE).
