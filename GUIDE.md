# User guide

Everything the program accepts and how to ask for it. Installation is covered
in [INSTALL.md](INSTALL.md).

- [The menu](#the-menu)
- [The command line](#the-command-line)
- [Course profiles](#course-profiles)
- [Templates](#templates)
- [Where files go](#where-files-go)
- [The cover page](#the-cover-page)
- [Writing the paper](#writing-the-paper)
- [When something fails](#when-something-fails)
- [Customizing templates and logos](#customizing-templates-and-logos)

## The menu

Run `investigacion` with no arguments. The first time it asks for the language
(your system's language is preselected) and saves it as `IDIOMA` in `.env`.

| Key | Action |
|---|---|
| ↑ ↓ (or Tab) | Move between fields |
| Enter | Edit the field, or open its list if it is marked ▸ |
| Del / Backspace | Clear the field |
| `g` (or F5) | Generate the PDF; warnings and errors appear in the result panel |
| `v` | Open the last PDF |
| `c` / `f` | Open a project folder (`input`, `output`, `courses`, `templates`, `cache`) |
| `l` | Switch between Spanish and English |
| `s` / `q` / Esc | Quit |

Three fields are chosen from a list instead of typed:

- **Course profile.** Fills in the course, teacher, group, template and output
  folder.
- **Markdown file.** Browse with the arrows; Enter (or →) opens a folder or
  picks the file, and Backspace (or ←) goes up. Typing filters the list. The
  list starts in the folder of the chosen course.
- **Template.**

Choosing the Markdown proposes the PDF name. A field left empty behaves as if
the option had not been given, so the profile or `.env` value applies. Rarely
used options (`--env-file`, `--allow-latex`, a template given by path,
`--logos`) are only available on the command line.

## The command line

```bash
investigacion paper.md --title "Ecuaciones diferenciales" --course "Cálculo"
```

Only the Markdown file and `--title` are always required. The course can come
from `--course` or from a profile. The PDF goes to `output/` and is named after the Markdown file;
generating again replaces it. The delivery date is the day you run the command.

| Option | What it does |
|---|---|
| `markdown` | The `.md` file. A bare name is also searched inside `input/` |
| `--title` | Title of the paper (cover only) |
| `--course` | Course name |
| `-p`, `--profile` | Course profile from `courses/` |
| `--file-name` | Name of the PDF (default: the Markdown's name) |
| `--teacher` | Teacher (falls back to `DOCENTE` in `.env`) |
| `--members` | Team members in one argument: `"Ana Ruiz, Luis Paz"` |
| `--group` | Group (falls back to `GRUPO` in `.env`) |
| `--output` | Output folder (default `output/`, or `output/<folder>` of the profile) |
| `--copy` | Extra folder for a copy of the PDF; can be repeated |
| `--template` | Template name (`apa`, `apa-simple`) or path to a `.ltx` |
| `--allow-latex` | Interpret LaTeX commands written in the Markdown |
| `--env-file` | Another `.env` file |
| `--logos` | Folder with the logos |
| `--lang` | Interface language for this run: `es` or `en` |

- **Precedence:** command-line option > course profile > `.env`.
- The old Spanish option names (`--titulo`, `--materia`, `--docente`…) still
  work.
- Options can go in any order. **Values with spaces go in quotes**; otherwise
  the extra words are reported as unexpected arguments.

## Course profiles

A profile saves what repeats in every paper of a course. It is a file named
`courses/<key>.toml`:

```toml
name = "Inteligencia artificial"   # required: the course name on the cover
teacher = "Nombre del docente"
group = "M"
members = "Ana Ruiz, Luis Paz"     # optional
template = "apa-simple"            # a template from templates/
folder = "IA"                      # input/IA and output/IA
```

```bash
investigacion Tarea1.md -p ia --title "Búsqueda heurística"
```

With `folder`, the Markdown is searched first in `input/IA/`, so two courses
can each have a `Tarea1.md`, and the PDF goes to `output/IA/`. Profiles are not
uploaded to git because they contain teachers' names. `courses/example.toml` is
the one to copy.

## Templates

Each folder in `templates/` with a `template.ltx` file is a cover design:

| Template | Cover |
|---|---|
| `apa` (default) | Geometric design drawn with TikZ |
| `apa-simple` | Classic, centered, no decoration |

Both share the APA body from `templates/common/`, so only the cover changes.

## Where files go

```
input/       your papers in Markdown, with subfolders per course if you like
output/      generated PDFs
courses/     course profiles
templates/   templates and logos
cache/       downloaded images, diagrams and LaTeX state (safe to delete)
examples/    template, syntax reference, example paper and catalog
```

The paths are relative to the project, not to your current folder. The
Markdown is looked up in this order:

1. the path as written;
2. inside `input/`, for example `IA/Tarea1.md`;
3. by name in any subfolder of `input/`.

If two files share a name, the program lists them and asks you to include the
subfolder.

## The cover page

The order of the cover lines is fixed:

```
ALUMNO (or INTEGRANTES)
MATERIA
DOCENTE
SEMESTRE
GRUPO
```

- **Always shown:** course (materia) and semester. The other lines disappear
  when they are empty.
- **Team members replace the student.** With `--members` (or `INTEGRANTES`)
  the cover lists the team, one name per line, and leaves out `ALUMNO`.
  Without members it shows the student from `.env`.
- **PDF metadata:** the author follows the same rule.

## Writing the paper

Use the recommended structure `# Introducción`, `# Desarrollo`, `# Conclusión`,
`# Referencias`. The program only warns if a heading is missing or out of
order. Do **not** write the cover, the table of contents or your data: the
template adds them. Start directly with `# Introducción`.

| Markdown | APA level |
|---|---|
| `#` | 1: centered, bold |
| `##` | 2: left, bold |
| `###` | 3: left, bold italic |
| `####`, `#####` | 4 and 5: indented, run into the paragraph |

**The golden rule:** leave a blank line before and after every block (heading,
list, table, code, diagram). Most badly formatted PDFs come from a missing
blank line.

References are written by hand in APA 7, as a dash list or as separate
paragraphs; the program adds the hanging indent.

### Syntax summary

| Element | How to write it |
|---|---|
| Bold, italic, both | `**bold**`, `*italic*`, `***both***` |
| Strikethrough, highlight | `~~text~~`, `==text==` |
| Sub/superscript | `H~2~O`, `X^2^` |
| Inline code | `` `code` `` |
| Lists | `- item`, `1. item`, `- [x] done`, `- [ ] pending` |
| Definition list | the term, then `: definition` on the next line |
| Quote | `> text` |
| Link | `[text](https://...)`, or a bare URL |
| Footnote | `text[^1]` and `[^1]: the note` on another line |
| Table | `\| A \| B \|` with `\|---\|---\|` below; `:---:` centers |
| Code block | ` ```python ` … ` ``` `; the language enables syntax colors |
| Formula | `$E = mc^2$` inline, `$$ … $$` on its own line |
| Line break | `<br>` |
| Emoji | pasted (🚀) or by code (`:rocket:`) |
| Note box | `::: nota` … `:::`; also `aviso`, `importante`, `ejemplo`, `definicion` |

Inline HTML is translated too (`<mark>`, `<sub>`, `<u>`, `<kbd>`, `<img>`…);
block HTML (`<div>`, `<table>`) is dropped. For a version of each element
rendered in a real PDF, see [`examples/syntax.md`](examples/syntax.md) and
[`examples/catalog.md`](examples/catalog.md) ([PDF](examples/catalog.pdf)).

### Images

```markdown
![Escudo de la universidad](logo.png){width=40%}
![Modelo OSI](https://ejemplo.com/osi.png)
```

- **Where they are looked up:** next to the `.md` and in `cache/`.
- **Web images:** downloaded once into `cache/remote/`.
- **Placement:** the caption follows APA ("Figura 1" in bold above, the title
  in italics), and the figure stays where you wrote it.
- **Formats:** pdflatex only handles PNG, JPG and PDF; convert SVG first.
- **If it fails:** a missing image leaves its alt text and a warning.

### Diagrams (Graphviz)

````markdown
```{.dot caption="Árbol binario de búsqueda"}
digraph { 50 -> 30; 50 -> 70; 30 -> 20; 30 -> 40; }
```
````

Diagrams are drawn as vector graphics and cached in `cache/diagrams/`. Without
Graphviz, the block stays as code and you get a warning.

### Charts (pgfplots)

````markdown
```{.pgfplot caption="Horas por fase"}
\begin{axis}[ybar, ymin=0, symbolic x coords={Análisis,Diseño,Código}, xtick=data]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180)};
\end{axis}
```
````

- **Chart types:** bars, lines, scatter with regression, histograms, box plots,
  error bars, functions, log scales and pie charts, in a grayscale palette
  suitable for APA. [`examples/catalog.md`](examples/catalog.md) has one of each.
- **The exception to "always produces a PDF":** this is real LaTeX, so a syntax
  error **stops** the build and the message points to the line.

### Symbols and ASCII art

- **Symbols:** `≠ ≤ ≥ ≈ ∈ ∑ ∫ √ ∞ α β Δ π ✓ ★`, box-drawing characters and emoji
  work as-is. An unknown symbol prints `[?]` and a warning names it; to support
  it, add it to the `SÍMBOLOS UNICODE` block of
  `templates/common/investigacion.sty`.
- **ASCII diagrams** (`└──► ┌─┐`) must go inside a code block; outside one,
  they fall apart.

### LaTeX commands and pasted text

- **Backslashes print as typed.** A path like `C:\Users\alumno` comes out
  literally instead of breaking the build. To write LaTeX on purpose
  (`\newpage`), add `--allow-latex`.
- **Pasted text is cleaned up.** Text pasted from Word, Notion, the web or an
  AI often carries invisible non-breaking spaces. Without cleanup, the `###`
  marks appear in the PDF and tables come out as rows of bars. The program
  removes those spaces when it reads the file.
- **Encoding:** UTF-8 and Windows-1252 files are both accepted.

## When something fails

Only the real LaTeX error is shown. `last-error.tex` and `last-error.log` are
saved next to the PDF.

| Symptom | Likely cause |
|---|---|
| The error points to a line of your text | A stray backslash with `--allow-latex` |
| The error points to a chart | pgfplots syntax; the message gives the line |
| `###` printed in the PDF | Invisible spaces or a missing blank line |
| A table shows as rows of bars | The dashes row under the header is missing |
| A diagram is misaligned | ASCII art outside a code block |
| `[?]` in the PDF | A symbol the template does not know; the warning says which |
| Wrong page numbers in the table of contents | Delete `cache/latex/` and generate again |

## Customizing templates and logos

- **New design:** copy `templates/apa-simple/` to `templates/my-design/` and
  change only the cover. It appears in the menu by itself and works with
  `--template my-design`.
- **What a template contains:** only the cover and the data markers
  (`%%TITULO%%`, `%%MATERIA%%`…). The APA body and everything Pandoc needs live
  in `templates/common/`, which every template loads.
- **Logos:**
  - **File names:** `logo-universidad.png` (top left) and `logo-facultad.png`
    (top right). Use dark or colored PNGs on a transparent background.
  - **Where they are looked up:** `--logos`, then `LOGOS` in `.env`, then
    `logos/` next to the template, then `templates/logos/`. Both files are
    optional.
  - **Position and size:** set in the `LOGOS` block of
    `templates/apa/template.ltx`.
