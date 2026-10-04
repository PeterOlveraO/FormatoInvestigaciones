# User guide

Everything the program accepts and how to ask for it. Installation is covered
in [INSTALL.md](INSTALL.md).

- [Formats and designs](#formats-and-designs)
- [Profiles and the wizard](#profiles-and-the-wizard)
- [The menu](#the-menu)
- [The command line](#the-command-line)
- [Where files go](#where-files-go)
- [The cover page](#the-cover-page)
- [Writing the paper](#writing-the-paper)
- [When something fails](#when-something-fails)
- [Making your own design](#making-your-own-design)

## Formats and designs

A PDF combines two things you choose separately:

- **The format** is the norm: font, size, line spacing, margins, how headings,
  captions and references look, and the page number. It lives in
  `templates/formats/<name>/`.
- **The design** is the cover (or title block) and the look. It lives in
  `templates/designs/<name>/`, and it lists the formats it works with.

| Format | Main rules | Structure check |
|---|---|---|
| `apa7` (default) | 12 pt Times, double spacing, APA headings, "Figura 1" above, hanging references, page number top right | Introducción, Desarrollo, Conclusión, Referencias |
| `harvard` | 12 pt Times, 1.5 spacing, no indent, numbered headings, page number bottom center | none |
| `mla` | 12 pt Times, double spacing, 1 in margins, half-inch indent, page number top right, Works Cited | none |

| Design | Look | Formats |
|---|---|---|
| `geometric-cover` (default) | Cover with geometric lines and arcs (TikZ) | apa7, harvard, mla |
| `classic-cover` | Classic centered cover | apa7, harvard, mla |
| `report` | Compact title block and table of contents, no cover | apa7, harvard, mla |
| `starter` | Minimal, heavily commented design to copy | apa7, harvard, mla |

Harvard and MLA are generic versions written from their published guides. If
your school asks for something different, copy the format folder into
`my-templates/formats/` and adjust it.

**Document language.** Each format has a default (all three: Spanish). A
profile (`language = "en"`) or `--doc-lang en` changes it. That switches the
date ("October 3, 2026"), "Contents", "Figure", and the box titles. The
fixed labels a design writes on its cover (like "FECHA DE ENTREGA") belong
to the design.

## Profiles and the wizard

A profile (`courses/<name>.toml`) keeps everything for one course: format,
design and all the cover data. Each profile has its own data, so two courses
can use different formats, teachers or even universities.

The **wizard** creates or edits a profile. It opens by itself the first time
(when there are no profiles). In the menu, use **New profile** on the home
view, or the options (`o`): `p` edits the chosen profile, `n` creates one.

1. **Profile name:** a short name such as `ia`.
2. **Format.**
3. **Design:** only the ones that work with that format are listed.
4. **The data that design shows:** university, faculty, student or team,
   course, teacher, group, semester and the design's own fields.
   - Required data is marked with `*`.
   - Shift+Tab goes back a step.
5. **Folder:** the subfolder of `input/` with that course's papers. PDFs go to
   the same subfolder of `output/`.

A profile file looks like this; you can also edit it by hand:

```toml
name = "Inteligencia artificial"     # the course; required
university = "Universidad Norte"
faculty = "Facultad de Ingeniería"
student = "Ana Ruiz"                 # or: members = "Ana Ruiz, Luis Paz"
teacher = "Nombre del docente"
group = "7-A"
semester = "2026-2"
format = "apa7"
design = "geometric-cover"
language = "es"                      # optional; default: the format's
folder = "IA"

[fields]                             # the design's own fields
SALON = "B-204"
```

- **Precedence:** command-line option > profile > `.env`. The `.env` is only a
  fallback for data no profile gives.
- **Old profiles:** a profile with the old `template = "apa"` still works.
- **Privacy:** profiles are not uploaded to git.

## The menu

Run `investigacion` with no arguments. It opens on the **home view**:

- a short description of the program;
- a menu: **Generate a PDF** (the form), **New profile** (the wizard),
  **Options** and **Quit**;
- a status panel: your profiles, the last PDF generated in this session and
  whether Pandoc and pdflatex (required) and Graphviz (optional, for diagrams)
  are installed. A missing required tool shows in red with a pointer to
  [INSTALL.md](INSTALL.md).

Home keys: ↑ ↓ to move, Enter to choose, `s` / `q` to quit.

**The form** (Generate a PDF):

| Key | Action |
|---|---|
| ↑ ↓ (or Tab) | Move between fields |
| Enter | Edit the field, or open its list if it is marked ▸ |
| Del / Backspace | Clear the field |
| `g` (or F5) | Generate the PDF; warnings and errors appear in the result panel |
| `v` | Open the last PDF |
| `o` | Open the options |
| Esc | Back to the home view |
| `s` / `q` | Quit |

**Options** (`o` in the form, or from the home menu). Enter runs the
highlighted row, or press its letter; Esc closes the list.

| Key | Action |
|---|---|
| `p` | Edit the chosen profile, or create one if none is chosen (wizard) |
| `n` | New profile (the wizard, empty) |
| `c` / `f` | Open a project folder (`input`, `output`, `courses`, `templates`, `cache`) |
| `l` | Switch between Spanish and English (saved in `.env`) |
| `i` / `h` | Back to the home view |

The option letters also work straight from the form, so `p`, `c`/`f` and `l`
still do what they used to. Both languages' letters work in either language.

- **Fields chosen from a list:**
  - the **profile**, which fills in the rest;
  - the **Markdown file**, browsed from the course's folder: Enter or → opens a
    folder, Backspace or ← goes up, typing filters;
  - the **format**;
  - the **design**, listing only the ones that accept the chosen format.
- **The design's own fields** (like *Salón*) appear at the end of the form
  when the design uses them.
- **Required fields** depend on the design (for example, the course only if the
  design shows it), and they are marked with `*`.
- **Empty fields** behave as if the option had not been given: the profile or
  `.env` value applies.

## The command line

```bash
investigacion paper.md --title "Ecuaciones diferenciales" --course "Cálculo"
investigacion Tarea1.md -p ia --title "Búsqueda heurística"
investigacion paper.md --title "Tema" --format mla --design report --set SALON="B-204"
```

Only the Markdown file and `--title` are always required. Other data is
required only if the chosen design uses it. The PDF goes to `output/` (or
`output/<folder>` of the profile) and is named after the Markdown file;
generating again replaces it.

| Option | What it does |
|---|---|
| `markdown` | The `.md` file. A bare name is also searched inside `input/` |
| `--title` | Title of the paper |
| `-p`, `--profile` | Course profile from `courses/` |
| `--format` | Format: `apa7`, `harvard`, `mla`, or one of yours |
| `--design` | Design name (or path to a `.ltx`); the old `--template` still works |
| `--course`, `--teacher`, `--members`, `--group` | Cover data, overriding the profile |
| `--set NAME=value` | A design's own field (`%%NAME%%`); can be repeated |
| `--doc-lang es\|en` | Document language |
| `--file-name` | Name of the PDF (default: the Markdown's name) |
| `--output`, `--copy` | Output folder; extra folders for copies (repeatable) |
| `--allow-latex` | Interpret LaTeX commands written in the Markdown |
| `--env-file`, `--logos` | Another `.env`; folder with the logos |
| `--lang es\|en` | Interface language for this run |
| `--check-template <design>` | Check a design and build the catalog with it (see below) |

- The old Spanish option names (`--titulo`, `--materia`, `--plantilla`…) still
  work.
- Values with spaces go in quotes.

## Where files go

```
input/                  your papers in Markdown, with subfolders per course if you like
output/                 generated PDFs
courses/                course profiles
templates/formats/      built-in formats (apa7, harvard, mla)
templates/designs/      built-in designs
templates/logos/        your logos (logo-universidad.png, logo-facultad.png)
my-templates/formats/   your own formats  (not uploaded to git)
my-templates/designs/   your own designs  (not uploaded to git)
cache/                  downloaded images, diagrams, LaTeX state and logs (safe to delete)
```

The Markdown is looked up as written, then inside `input/`, then by name in any
subfolder of `input/` (the profile's folder first). If two files share a name,
the program asks you to include the subfolder.

## The cover page

What the cover shows depends on the design. The built-in covers follow these
rules:

- **Lines with no data disappear:** student, team, teacher and group are
  omitted when empty.
- **Team members replace the student,** one name per line. The PDF metadata
  follow the same rule.
- **The order of the cover lines is fixed in the design:** student (or team),
  course, teacher, semester, group.
- **Logos:** `geometric-cover` and `classic-cover` show your logos if they are
  in `templates/logos/`.

## Writing the paper

Do **not** write the cover, the table of contents or your data: the design
adds them. Start directly with your first heading.

APA 7 recommends `# Introducción`, `# Desarrollo`, `# Conclusión`,
`# Referencias`, and the program warns if one is missing or out of order.
Harvard and MLA have no fixed structure, so they do not warn. How each
heading level looks is up to the format:

| Markdown | APA 7 | Harvard | MLA 9 |
|---|---|---|---|
| `#` | centered, bold | `1` numbered, large, bold | bold |
| `##` | left, bold | `1.1` numbered, bold | italic |
| `###` | left, bold italic | `1.1.1` numbered, bold italic | bold italic |

**The golden rule:** leave a blank line before and after every block (heading,
list, table, code, diagram). Most badly formatted PDFs come from a missing
blank line.

References are written by hand, in the style of your format, as a dash list
or as separate paragraphs under a `# Referencias` (or `# References`, `#
Bibliography`, `# Works Cited`) heading. The format adds the hanging indent.

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
- **Placement:** the figure stays where you wrote it; the caption follows the
  format (APA: "Figura 1" in bold above; Harvard and MLA: below).
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
  in APA. [`examples/catalog.md`](examples/catalog.md) has one of each.
- **The exception to "always produces a PDF":** this is real LaTeX, so a syntax
  error **stops** the build and the message points to the line.

### Symbols and ASCII art

- **Symbols:** `≠ ≤ ≥ ≈ ∈ ∑ ∫ √ ∞ α β Δ π ✓ ★`, box-drawing characters and emoji
  work as-is. An unknown symbol prints `[?]` and a warning names it; to support
  it, add it to the `SÍMBOLOS UNICODE` block of
  `templates/common/investigacion-base.sty`.
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
| "Missing data used by the design…" | Fill it in the profile (`p` in the menu's options), in `.env` or with `--set` |
| "The design … does not work with the format …" | Choose a format the design lists, or another design |
| Your own design fails | Run `investigacion --check-template <design>` |

### Logs

Every run, from the menu or the command line, appends to
`cache/logs/investigacion.log`. It records what the program did, step by step:
the options it received, the design and format it chose, each Pandoc and
pdflatex run with its time, every warning and error you saw, and crashes with
their backtrace. When something fails without a clear message, look at the end
of this file, or attach it when you ask for help.

- **Size:** if the log is over 1 MB when a run starts, it is renamed to
  `investigacion.old.log` (replacing the previous one) and a new one starts.
- **More detail:** set `INVESTIGACION_LOG=debug` to also record the full
  commands, their folders and Pandoc's own messages.
- **No log:** set `INVESTIGACION_LOG=off` (or `0`).
- **Privacy:** the log stays on your computer, inside `cache/`, which git
  ignores. It contains paths, titles and the messages you saw, so review it
  before sharing it.
- If the log cannot be written (for example, a read-only folder), the program
  keeps working without it.

## Making your own design

1. **Copy the starter:** copy `templates/designs/starter/` to
   `my-templates/designs/<your-design>/`. It is heavily commented.
2. **Change the cover** between the `TU DISEÑO` marks. Keep the contract:
   - `\documentclass[%%CLASS_OPTIONS%%]{article}`, so the format can set the
     font size and paper;
   - `\usepackage{investigacion-base}` first, then `%%FORMAT%%` (the program
     puts the language and the format there);
   - `\usepackage{investigacion-final}` at the end of the preamble;
   - `%%CONTENIDO_MARKDOWN%%` where the paper goes, with `\FormatBodySetup`
     before it.
3. **Use the data you need:**
   - **Standard data:** `%%TITULO%%`, `%%UNIVERSIDAD%%`, `%%FACULTAD%%`,
     `%%ALUMNO%%`, `%%INTEGRANTES%%`, `%%MATERIA%%`, `%%DOCENTE%%`,
     `%%GRUPO%%`, `%%SEMESTRE%%`, `%%FECHA_ENTREGA%%`.
   - **Your own fields:** any other uppercase name (`%%SALON%%`) is a field of
     your own. The menu and the wizard ask for it, and it is saved in the
     profile.
4. **Describe it in `template.toml`:**

   ```toml
   description = { es = "Mi portada", en = "My cover" }
   formats = ["apa7", "harvard"]         # formats it works with; empty = all
   optional = ["UNIVERSIDAD"]            # standard data it shows but does not require
   [fields.SALON]
   label = { es = "Salón", en = "Room" }
   help = { es = "Aula de entrega", en = "Delivery room" }
   required = true
   ```

5. **Check it:**

   ```bash
   investigacion --check-template <your-design>
   ```

   It reports missing contract parts, unknown formats and fields not
   described. It suggests the standard name when a field looks like a typo
   (`%%DOCENTES%%` → "did you mean `%%DOCENTE%%`?"). Then it builds the
   example catalog with every format the design accepts.

**Self-contained designs.** A design without `%%FORMAT%%` carries all its own
rules, for a school format that follows no known norm.

**Your own formats.** Copy `templates/formats/<format>/` to
`my-templates/formats/<name>/`. Change `format.sty` (style) and `format.toml`
(name, default language, class options, recommended headings).

**Logos.**
- **Names:** `logo-universidad.png` (top left) and `logo-facultad.png` (top
  right), in `templates/logos/`, in the folder given by `--logos`/`LOGOS`, or
  in `logos/` next to a design.
- **Image:** use dark or colored PNGs with a transparent background.
- **Position:** set in the `LOGOS` block of
  `templates/designs/geometric-cover/template.ltx`.
