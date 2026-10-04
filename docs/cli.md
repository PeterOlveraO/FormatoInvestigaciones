# Command-line reference

Every command and option of `investigacion`. For a friendly introduction see
[usage.md](usage.md); for formats, designs and the Markdown syntax see
[reference.md](reference.md).

- [Commands](#commands)
- [Options](#options)
- [Precedence](#precedence)
- [Cover data without a profile (`--set`)](#cover-data-without-a-profile---set)
- [Finding the Markdown file](#finding-the-markdown-file)
- [Where the output goes](#where-the-output-goes)
- [`--check-template`](#--check-template)
- [Exit codes](#exit-codes)
- [Environment variables](#environment-variables)
- [`settings.toml`](#settingstoml)
- [Examples](#examples)

## Commands

| Command | What it does |
|---|---|
| `investigacion` | No arguments: opens the full-screen menu (see [usage.md](usage.md)). |
| `investigacion <MARKDOWN> --title <TITLE> [options]` | Generates one PDF. |
| `investigacion --check-template <design>` | Checks a design and builds the catalog with it. |
| `investigacion --help` (`-h`) | Shows the help, in the interface language. |
| `investigacion --version` (`-V`) | Shows the version (`investigacion 2.0.0`). |

Only the Markdown file and `--title` are always required. The rest is
required only if the chosen design uses it (see
[Cover data](#cover-data-without-a-profile---set)).

Options may go in any order, before or after the Markdown file. Both
`--title "Tema"` and `--title="Tema"` work. A value with spaces needs quotes.

## Options

The long names are in English. Every old Spanish or earlier name still works
as a **hidden alias**: it is not shown in `--help`, and it is kept on purpose.

| Option | Short | Hidden aliases | Value | Default | Repeatable |
|---|---|---|---|---|---|
| `<MARKDOWN>` (positional) | | | path or file name | none, required | no |
| `--title` | | `--titulo` | text | none, required | no |
| `--file-name` | | `--nombre` | name | the Markdown's name | no |
| `--course` | | `--materia`, `--subject` | text | the profile's `name` | no |
| `--profile` | `-p` | `--perfil`, `--subject-profile` | profile key | none | no |
| `--teacher` | | `--docente` | text | the profile's `teacher` | no |
| `--members` | | `--integrantes` | names separated by commas | the profile's `members` | no |
| `--group` | | `--grupo` | text | the profile's `group` | no |
| `--output` | | `--salida` | folder | `output/` or `output/<folder>` | no |
| `--copy` | | `--copia` | folder | none | **yes** |
| `--settings` | | `--env-file` | file | `settings.toml` in the project | no |
| `--allow-latex` | | `--permitir-latex` | none (flag) | off | no |
| `--design` | | `--template`, `--plantilla`, `--diseno` | design name or `.ltx` path | the profile's, else `geometric-cover` | no |
| `--format` | | `--formato` | format name | the profile's, else the design's first | no |
| `--set` | | none | `NAME=value` | none | **yes** |
| `--doc-lang` | | `--idioma-documento` | `es` or `en` | the profile's, else the format's | no |
| `--logos` | | none | folder | `LOGOS` setting, else the design's | no |
| `--lang` | | `--idioma` | `es` or `en` | saved language, else the system's | no |
| `--help` | `-h` | none | none | | no |
| `--version` | `-V` | none | none | | no |
| `--check-template` | | `--revisar-plantilla` | design name | none | no |

`--lang` and `--doc-lang` also accept the full names (`español`, `spanish`,
`english`, `inglés`, `ingles`, `espanol`), in any case. Any other value is an
argument error.

### What each option does

**`<MARKDOWN>`**: the `.md` file with the paper. A bare name such as
`Tarea1.md` is also looked up inside `input/`
([details](#finding-the-markdown-file)). It must end in `.md`.

**`--title`**: the title shown on the cover. It does **not** name the file.

**`--file-name`**: name of the PDF, with or without `.pdf`. Accents, spaces
and symbols are simplified to lowercase words joined by hyphens
(`Entrega Álgebra.pdf` becomes `entrega-algebra.pdf`). Without it the PDF
takes the Markdown's name (`Tarea2-3.md` becomes `tarea2-3.pdf`).

**`--course`**: course name for the cover. Overrides the profile's `name`.

**`--profile`, `-p`**: course profile from `courses/` (for example `-p ia`
reads `courses/ia.toml`). The key is matched ignoring case. A path to a
`.toml` file also works. A profile gives the course, teacher, group, members,
university, faculty, student, semester, format, design, document language,
custom fields and folder. See [`courses/README.md`](../courses/README.md).

**`--teacher`, `--members`, `--group`**: cover data that overrides the
profile. If nothing gives a value, the line is left out of the cover.
`--members` takes names separated by commas (`"Ana Ruiz, Luis Paz"`); a
design that shows members replaces the student with them.

**`--output`**: folder for the PDF. Created if missing. `~` is expanded.

**`--copy`**: extra folder that receives a copy of the PDF. Repeat the option
for several folders. A folder equal to the output folder, or repeated, is
skipped. Folders are created if missing; `~` is expanded.

**`--settings`**: use another settings file instead of the project's
`settings.toml` ([format](#settingstoml)). A missing file is not an error; an
invalid one is.

**`--allow-latex`**: interpret LaTeX commands written in the Markdown
(`\newpage`, and so on). Without it they are shown as text.

**`--design`**: the cover and look. Either a name (looked up in
`my-templates/designs/` and `templates/designs/`), a folder, or the path to a
`.ltx` file. The old names `apa` (now `geometric-cover`) and `apa-simple` (now
`classic-cover`) still work. A design that is self-contained (it does not use
`%%FORMAT%%`) applies no format.

**`--format`**: the norm: `apa7` (default), `harvard`, `mla`, or one of your
own. Without it, the first format the design lists is used (APA 7 if it lists
none). A format the design does not accept is an error that names the
compatible ones.

**`--set NAME=value`**: a design's own field (for example
`--set SALON="B-204"`), or one of the standard cover data names when there is
no profile ([see below](#cover-data-without-a-profile---set)). The name is
converted to upper case; the value may be empty (`--set AULA=`). Without an
`=` and a name, the option is rejected with `use NAME=value`. Repeat it for
several fields. If the same name appears twice, the last one wins.

**`--doc-lang`**: language of the PDF (`es` or `en`): babel, the date and the
titles of boxes. It is independent of the interface language (`--lang`).

**`--logos`**: folder with `logo-universidad.png` and `logo-facultad.png`. The
folder must exist. Without the option, the `LOGOS` setting is used; if there
is none, the `logos/` folder next to the design, then `templates/logos/`. No
logos at all is not an error: the cover is built without them.

**`--lang`**: language of the messages, help and errors for this run. It does
not change the saved language and does not affect the PDF.

**`--check-template`**: see [below](#--check-template). It is not a normal
clap option: it does not appear in the option list of `--help`, only in its
last lines.

## Precedence

Each value is taken from the first place that has it:

| Value | Order |
|---|---|
| Course, teacher, members, group | option > profile |
| University, faculty, student, semester | `--set` > profile |
| Design | `--design` > profile `design` (or old `template`) > `geometric-cover` |
| Format | `--format` > profile `format` > the design's first format > `apa7` |
| Document language | `--doc-lang` > profile `language` > the format's > Spanish |
| Output folder | `--output` > `output/<folder>` of the profile > `output/` |
| Markdown folder to search | the profile's `input/<folder>` > `input/` |
| Custom fields | `--set` over the profile's `[fields]`, name by name |
| Logos | `--logos` > `LOGOS` setting > design's `logos/` > `templates/logos/` |
| Interface language | `--lang` > `IDIOMA` (environment or `settings.toml`) > the system's |

An empty option or an empty profile value counts as "not given". The cover
date is always the day you run the command, written in the document language.

## Cover data without a profile (`--set`)

With a profile, the cover data (university, faculty, student, semester, and
so on) comes from `courses/<key>.toml`. Without one, give it on the command
line. Besides `--course`, `--teacher`, `--members` and `--group`, the
following names work with `--set` and fill the standard data directly:

| `--set` name | Fills |
|---|---|
| `UNIVERSIDAD` | University |
| `FACULTAD` | Faculty |
| `ALUMNO` | Student |
| `SEMESTRE` | Semester |
| `DOCENTE` | Teacher (`--teacher` wins if both are given) |
| `GRUPO` | Group (`--group` wins if both are given) |
| `INTEGRANTES` | Team members (`--members` wins if both are given) |

Which of them are **required** depends on the design. A standard field is
required only if the design uses its marker and does not list it under
`optional` in its `template.toml`. The standard fields that can be required
are title, university, faculty, semester and course; student, teacher, group
and members are never required. A custom field is required only if its spec
says so. If something required is missing, the command fails with exit code 1
and a message such as:

```
Error: Missing data used by the design geometric-cover: UNIVERSIDAD, FACULTAD, SEMESTRE, MATERIA. Add them to the profile (Options → p in the menu) or with --set NAME=value.
```

The default design (`geometric-cover`) needs university, faculty, semester
and course. The `report` design needs no cover data beyond the title.

## Finding the Markdown file

The program looks for the file in this order, and the first hit wins:

1. The path as written (relative to the current folder). `~` is expanded.
   An existing path always wins.
2. With a profile that has a `folder`: that folder inside `input/`
   (`input/<folder>`), as a path and then, for a bare name, by name in any
   subfolder of it. This lets two courses have their own `Tarea1.md`.
3. Inside `input/`: the path under `input/`, then, for a bare name (no
   folder part), by name in every subfolder of `input/`.

If two files in different subfolders share the name, the command fails and
lists them; give the subfolder (`IA/Tarea1.md`). If nothing is found, you get
`The Markdown file does not exist: <absolute path>`. The file must end in
`.md` (any case), and an empty file is an error.

Relative image paths in the paper are resolved from the Markdown's own
folder.

## Where the output goes

- The PDF goes to `--output`, else `output/<folder>` of the profile, else
  `output/` in the project. The folder is created and tested for writing
  before the build starts.
- The name is described under `--file-name`. An existing PDF with that name is
  replaced.
- The command prints `PDF generated: <path>` on standard output and one
  `Copy saved: <path>` line per `--copy` folder.
- Warnings (a missing image, an unknown symbol, a missing recommended
  heading, a missing Graphviz) go to standard error as `Warning: …`. They
  never stop the PDF.
- When pdflatex fails, `last-error.tex` and `last-error.log` are saved next to
  the PDF, and the message shows the real LaTeX error.
- Each external tool (Pandoc, pdflatex, Graphviz) has a 180 second time limit.
- Temporary build files live in a temporary folder that is removed
  afterwards. Downloaded images, diagrams and the LaTeX state are cached in
  `cache/` and may be deleted at any time.

The project root (where `input/`, `output/`, `courses/`, `templates/`,
`cache/` and `settings.toml` live) is found like this:

1. `INVESTIGACION_HOME`, if set.
2. The first folder, starting at the current one and going up, that contains
   `templates/common/investigacion-base.sty`.
3. The folder where the program was compiled.
4. The current folder (nothing is written there if it has no templates).

## `--check-template`

```bash
investigacion --check-template <design>
```

Checks a design of your own and test-builds it. `<design>` is a name, a
folder or the path to a `.ltx` file, like `--design`.

- It is handled **before** the other arguments are parsed. It is found
  anywhere in the arguments, and the argument right after it is the design.
  Everything else on the line is ignored. `--revisar-plantilla` is its old
  name. The `--check-template=<design>` form is not supported.
- It first reads the design and reports:
  - errors: `\usepackage{investigacion-base}`, `\usepackage{investigacion-final}`
    or `%%CONTENIDO_MARKDOWN%%` missing, or a format in `template.toml` that
    does not exist;
  - warnings: no `%%FORMAT%%` (self-contained design), no `%%CLASS_OPTIONS%%`,
    a marker that looks like a typo of a standard one (for example
    `%%DOCENTES%%`), custom fields not described in `template.toml`, and
    `template.toml` entries the design does not use.
- If there are no errors, it builds `examples/catalog.md` with every format
  the design accepts (or once, if the design is self-contained), using sample
  data. If `examples/catalog.md` does not exist, the build is skipped with a
  warning.
- Output lines start with `✗` (error or build failure), `!` (warning) or `✓`
  (build ok). The last line is `Done: the design is ready to use.` when all is
  well. The text follows `--lang` and the saved language.

Exit codes: `0` passed, `1` errors or a failed build (or the design was not
found), `2` no design name after the option.

## Exit codes

| Code | When |
|---|---|
| `0` | The PDF was generated; `--help`/`--version` shown; `--check-template` passed; the menu closed normally. |
| `1` | A generation error (missing data, file not found, Pandoc/pdflatex failure, unwritable folder, bad profile or settings file); `--check-template` found errors or a build failed; the menu closed because of an internal error. |
| `2` | An argument error (missing `--title`, unknown option, invalid value such as `--lang fr`, no design after `--check-template`). The message is in the interface language and ends with `Use --help to see the options.` |

Errors go to standard error as `Error: <message>`.

## Environment variables

| Variable | Used for |
|---|---|
| `INVESTIGACION_HOME` | Project root (see above). |
| `INVESTIGACION_LOG` | Log level: `debug` records full commands and paths; `off` or `0` turns the log off; anything else (or unset) is `info`. |
| `INVESTIGACION_TIMING` | `1` prints `[timing] <tool>: <seconds> s` to standard error after each Pandoc/pdflatex/Graphviz run. |
| `IDIOMA`, `INTERFACE_LANGUAGE` | Interface language (`es` or `en`). `IDIOMA` is checked first. |
| `LOGOS` | Folder with the logos, when `--logos` is not given. |
| `TEXINPUTS` | Your own value is kept and appended after the project's folders when pdflatex runs. |
| `HOME`, `USERPROFILE` | Used to expand `~` in paths. |

`IDIOMA`, `INTERFACE_LANGUAGE` and `LOGOS` can also be set in
`settings.toml`; when both exist, the **environment variable wins**. The
system language comes from the operating system's locale (`LANG` and similar):
Spanish if it starts with `es`, otherwise English.

The log is written to `cache/logs/investigacion.log` (a log over 1 MB is
renamed to `investigacion.old.log` the next time the program starts). It is
written by both the CLI and the menu and never stops a run. It is only
created inside a real project folder.

## `settings.toml`

The file `settings.toml` in the project root remembers the interface language
and, if you want, the logos folder. Despite the name, it is **not** TOML:
it has one `KEY=value` per line.

```
# comments start with #
IDIOMA="en"
LOGOS="/home/me/logos"
```

- Accepted keys: `IDIOMA` (or `INTERFACE_LANGUAGE`) and `LOGOS`.
- An optional `export ` prefix and single or double quotes are accepted.
  A comment after an unquoted value needs a space before the `#`.
- A line without `=`, or a key that is not a plain name, is an error.
- The menu writes `IDIOMA` here when you choose a language (first run, or
  Options → l).
- The cover data is **not** stored here: it lives in the course profiles.
- Use another file with `--settings`.

## Examples

All of them use placeholder data. `Tarea1.md` can live in the current folder
or in `input/`.

**With a profile** (the profile gives course, university, format, design…):

```bash
investigacion Tarea1.md -p example --title "Búsqueda heurística"
```

**Without a profile**, giving the cover data with `--set`:

```bash
investigacion Tarea1.md --title "Ecuaciones diferenciales" \
  --course "Cálculo" \
  --set UNIVERSIDAD="Nombre de la universidad" \
  --set FACULTAD="Nombre de la facultad" \
  --set ALUMNO="Nombre del alumno" \
  --set SEMESTRE="2026-2" \
  --teacher "Nombre del docente" --group "7-A"
```

**A team paper** (members replace the student on the cover):

```bash
investigacion Tarea1.md -p example --title "Tema" --members "Ana Ruiz, Luis Paz"
```

**Choosing the format and the design** (the design must accept the format):

```bash
investigacion Tarea1.md -p example --title "Tema" --format mla --design classic-cover
investigacion Tarea1.md -p example --title "Tema" --format harvard --design report
```

**A design from a file of your own:**

```bash
investigacion Tarea1.md -p example --title "Tema" --design my-templates/designs/mine/template.ltx
```

**An English document** (the PDF language), with English messages:

```bash
investigacion Essay.md -p example --title "On Graph Theory" --doc-lang en --format mla --lang en
```

**Name and folder of the PDF, plus extra copies:**

```bash
investigacion Tarea1.md -p example --title "Tema" \
  --file-name entrega-1 \
  --output ~/Documentos/Entregas \
  --copy ~/Drive/Entregas --copy /media/usb
```

**A custom design field** (`mine` is your design; it uses `%%SALON%%` in its `.ltx`):

```bash
investigacion Tarea1.md -p example --title "Tema" --design mine --set SALON="B-204"
```

**LaTeX commands in the Markdown, and your own logos:**

```bash
investigacion Tarea1.md -p example --title "Tema" --allow-latex --logos ~/logos
```

**Checking a design of your own:**

```bash
investigacion --check-template mine
investigacion --check-template my-templates/designs/mine
investigacion --check-template starter --lang en
```

**Diagnosing a problem** (full log in `cache/logs/investigacion.log`, and the
time of each tool):

```bash
INVESTIGACION_LOG=debug INVESTIGACION_TIMING=1 investigacion Tarea1.md -p example --title "Tema"
```
