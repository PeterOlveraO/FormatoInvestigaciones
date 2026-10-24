# Command-line reference

Every command and option of `investigacion`. For a friendly introduction see
[usage.md](usage.md); for the format, designs and the Markdown syntax see
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
| `investigacion --convert-image <SOURCE> <DESTINATION>` | Internal, hidden from `--help`: the image filter calls it to turn an SVG or WEBP into a PNG. It must be the first argument; it does not read `settings.toml` or write the log. It stops itself after 30 s. Exit code `0` converted, `1` failed (short reason on standard error), `2` wrong number of arguments, `124` past the 30 s limit. |

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
| `--set` | | none | `NAME=value` | none | **yes** |
| `--doc-lang` | | `--idioma-documento` | `es` or `en` | the profile's, else Spanish | no |
| `--logos` | | none | folder | `LOGOS` setting, else the design's | no |
| `--lang` | | `--idioma` | `es` or `en` | saved language, else the system's | no |
| `--help` | `-h` | none | none | | no |
| `--version` | `-V` | none | none | | no |
| `--check-template` | | `--revisar-plantilla` | design name | none | no |
| `--format` (hidden) | | `--formato` | any text | ignored | no |

`--lang` and `--doc-lang` also accept the full names (`español`, `spanish`,
`english`, `inglés`, `ingles`, `espanol`), in any case, with or without a
region (`en-US`, `es_MX`). Any other value is an argument error.

### What each option does

**`<MARKDOWN>`**: the `.md` file with the paper. A bare name such as
`Tarea1.md` is also looked up inside `input/`
([details](#finding-the-markdown-file)). It must end in `.md`.

**`--title`**: the title shown on the cover. It does **not** name the file.

**`--file-name`**: name of the PDF, with or without `.pdf`. Accents, spaces
and symbols are simplified to lowercase words joined by hyphens
(`Entrega Álgebra.pdf` becomes `entrega-algebra.pdf`). Without it the PDF
takes the Markdown's name (`Tarea2-3.md` becomes `tarea2-3.pdf`). The name is
cut to 100 characters, and Windows reserved names get a prefix (`con.md`
becomes `investigacion-con.pdf`).

**`--course`**: course name for the cover. Overrides the profile's `name`.

**`--profile`, `-p`**: course profile from `courses/` (for example `-p ia`
reads `courses/ia.toml`). The key is matched ignoring case. A path to a
`.toml` file also works. A profile gives the course, teacher, group, members,
university, faculty, student, design, document language, custom fields and
folders. An old profile with `semester` or `format` still loads: each one is
ignored with a one-line warning. See [`courses/README.md`](../courses/README.md).

**`--teacher`, `--members`, `--group`**: cover data that overrides the
profile. If nothing gives a value, the line is left out of the cover.
`--members` takes names separated by commas (`"Ana Ruiz, Luis Paz"`); a
design that shows members replaces the student with them.

**`--output`**: folder for the PDF. Created if missing. `~` is expanded.

**`--copy`**: extra folder that receives a copy of the PDF. Repeat the option
for several folders. Without `--copy`, the profile's `copies` are used. A folder equal to the output folder, or repeated, is
skipped. Folders are created if missing; `~` is expanded. A folder that fails
(it is a file, cannot be created, or the copy fails) is reported as a warning
and skipped: the PDF is still generated and the exit code stays 0.

**`--settings`**: use another settings file instead of the project's
`settings.toml` ([format](#settingstoml)). A missing file is not an error; an
invalid one is.

**`--allow-latex`**: interpret LaTeX commands written in the Markdown
(`\newpage`, and so on). Without it they are shown as text.

**`--design`**: the cover and look. Either a name (looked up in
`templates/designs/` and `my-templates/designs/`), a folder, or the path to a
`.ltx` file. The retired names `apa`, `apa-simple`, `classic-cover`, `report`
and `starter` use `geometric-cover` with a one-line warning (a design of your
own with one of those names wins, without a warning). A design that is
self-contained (it does not use `%%FORMAT%%`) applies no format.

**`--format`** (hidden): APA 7 is the only format, so the option is accepted
only so old commands keep working. `--format apa7` does nothing; any other
value prints `The format "<name>" no longer exists; APA 7 is used.` and the
PDF is made in APA 7.

**`--set NAME=value`**: a design's own field (for example
`--set SALON="B-204"`), or one of the standard cover data names when there is
no profile ([see below](#cover-data-without-a-profile---set)). The name is
converted to upper case; the value may be empty (`--set AULA=`). Without an
`=` and a name, the option is rejected with `use NAME=value`. Repeat it for
several fields. If the same name appears twice, the last one wins.

**`--doc-lang`**: language of the PDF (`es` or `en`): babel, the date and the
titles of boxes. Without it, the profile's `language`, else Spanish (the APA 7
default). It is independent of the interface language (`--lang`).

**`--logos`**: folder with `logo-universidad` and `logo-facultad`, each as SVG,
PDF, PNG, JPG or WEBP (looked up in that order; SVG and WEBP are converted to
PNG). Without the option, the `LOGOS` setting is used; if there
is none, the `logos/` folder next to the design, then `logos/` in your work
folder, then `templates/logos/`. No
logos at all is not an error: the cover is built without them. A `--logos` or
`LOGOS` folder that does not exist (a removed USB drive, say) is a warning that
names where the path came from, and the cover is built without logos; it does
not fall back to the other folders. A logo with the right name but another
extension or upper case (`logo-facultad.GIF`, `logo-facultad.PNG`) is not
used, and a warning says how to rename it.

**`--lang`**: language of the messages, help and errors for this run. It does
not change the saved language and does not affect the PDF.

**`--check-template`**: see [below](#--check-template). It is not a normal
clap option: it does not appear in the option list of `--help`, only in its
last lines.

**`--convert-image`**: internal, see [Commands](#commands). Do not use it by
hand.

## Precedence

Each value is taken from the first place that has it:

| Value | Order |
|---|---|
| Course, teacher, members, group | option > `--set MATERIA=…` (`DOCENTE`, `INTEGRANTES`, `GRUPO`) > profile |
| University, faculty, student | `--set` > profile |
| Design | `--design` > profile `design` (or old `template`) > `geometric-cover` |
| Document language | `--doc-lang` > profile `language` > the format's (`es`) |
| Output folder | `--output` > profile `output` > `output/<folder>` of the profile > `output/` |
| Extra copies | `--copy` (all of them) > profile `copies` > none |
| Markdown folder to search | the profile's `input/<folder>` > `input/` |
| Custom fields | `--set` over the profile's `[fields]`, name by name |
| Logos | `--logos` > `LOGOS` setting > design's `logos/` > work folder `logos/` > `templates/logos/` |
| Interface language | `--lang` > `IDIOMA` (environment or `settings.toml`) > the system's |

An empty option or an empty profile value counts as "not given". The cover
date is always the day you run the command, written in the document language.

## Cover data without a profile (`--set`)

With a profile, the cover data (university, faculty, student, and so on) comes from `courses/<key>.toml`. Without one, give it on the command
line. Besides `--course`, `--teacher`, `--members` and `--group`, the
following names work with `--set` and fill the standard data directly:

| `--set` name | Fills |
|---|---|
| `UNIVERSIDAD` | University |
| `FACULTAD` | Faculty |
| `ALUMNO` | Student |
| `MATERIA` | Course (`--course` wins if both are given) |
| `DOCENTE` | Teacher (`--teacher` wins if both are given) |
| `GRUPO` | Group (`--group` wins if both are given) |
| `INTEGRANTES` | Team members (`--members` wins if both are given) |

Which of them are **required** depends on the design. A standard field is
required only if the design uses its marker and does not list it under
`optional` in its `template.toml`. The standard fields that can be required
are title, university, faculty and course; student, teacher, group
and members are never required. A custom field is required only if its spec
says so. If something required is missing, the command fails with exit code 1
and a message such as:

```
Error: Missing data used by the design geometric-cover: UNIVERSIDAD, FACULTAD, MATERIA. Add them to the profile (Options → p in the menu) or with --set NAME=value.
```

The default design (`geometric-cover`) needs university, faculty and course.

`--set SEMESTRE=…` is ignored with a warning (`use --group`), unless the
design is one of yours that uses `%%SEMESTRE%%`: then it is a custom field.

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

- The PDF goes to `--output`, else the profile's `output`, else
  `output/<folder>` of the profile, else `output/` in the project. The folder is created and tested for writing
  before the build starts.
- The name is described under `--file-name`. An existing PDF with that name is
  replaced.
- The command prints `PDF generated: <path>` on standard output and one
  `Copy saved: <path>` line per copy folder (`--copy` or the profile's `copies`).
- Warnings (a missing image, an unknown symbol, a missing recommended
  heading, a missing Graphviz) go to standard error as `Warning: …`. They
  never stop the PDF.
- When pdflatex fails, `last-error.tex` and `last-error.log` are saved next to
  the PDF, and the message shows the real LaTeX error.
- Each external tool (Pandoc, pdflatex, Graphviz) has a 180 second time limit;
  the conversion of one SVG or WEBP image, 30 seconds.
- Temporary build files live in a temporary folder that is removed
  afterwards. Downloaded images, SVG and WEBP images converted to PNG,
  diagrams and the LaTeX state are cached in `cache/` and may be deleted at
  any time.

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
  anywhere in the arguments, and the argument right after it is the design;
  `--check-template=<design>` works too. A value that starts with `-` is not
  taken as the design. Everything else on the line is ignored.
  `--revisar-plantilla` is its old name.
- It first reads the design and reports:
  - errors: `\usepackage{investigacion-base}`, `\usepackage{investigacion-final}`
    or `%%CONTENIDO_MARKDOWN%%` missing;
  - warnings: no `%%FORMAT%%` (self-contained design), no `%%CLASS_OPTIONS%%`,
    a marker that looks like a typo of a standard one (for example
    `%%DOCENTES%%`), `%%SEMESTRE%%` (no longer a standard field: use
    `%%GRUPO%%`), custom fields not described in `template.toml`,
    `template.toml` entries the design does not use, a leftover `formats`
    list in `template.toml`, and a retired design name (`starter`…) that led
    to `geometric-cover`.
- If there are no errors, it builds `examples/catalog.md` once with APA 7 (or
  with no format, if the design is self-contained), using sample data. If
  `examples/catalog.md` does not exist, the build is skipped with a warning.
  The build leaves no LaTeX state in `cache/latex/`.
- Output lines start with `✗` (error or build failure), `!` (warning) or `✓`
  (build ok). The last line is `Done: the design is ready to use.` when all is
  well. The text follows `--lang` and the saved language.

Exit codes: `0` passed, `1` errors or a failed build (or the design was not
found), `2` no design name after the option.

## Exit codes

| Code | When |
|---|---|
| `0` | The PDF was generated; `--help`/`--version` shown; `--check-template` passed; the menu closed normally. Recovered problems only print a `Warning:` line and keep `0`: a missing or unreadable image, a broken SVG/WEBP, a remote image that cannot be downloaded, a missing or misnamed logo or logos folder, Graphviz not installed, a skipped extra copy, a retired format or design (`mla`, `classic-cover`…), `semester` in a profile. |
| `1` | A generation error: missing cover data, Markdown not found or empty, unknown profile or design, invalid profile or settings file, unwritable output folder, Pandoc or pdflatex missing or past the 180 s limit, and LaTeX that does not compile. A broken ` ```pgfplot `/` ```tikz ` block or broken `$…$` math is real LaTeX, so it ends here: the message starts with `The LaTeX build failed:`, shows the line, and ends with `Full log: <output folder>/last-error.log` (`last-error.tex` is saved next to it). Also: `--check-template` found errors, a build failed or the design was not found; the menu closed because of an internal error or could not find `templates/`. |
| `2` | An argument error (missing `--title`, unknown option, invalid value such as `--lang fr`, no design after `--check-template`). The message is in the interface language and ends with `Use --help to see the options.` |
| `124` | Only the internal `--convert-image`: the conversion passed its own 30 s limit and was stopped. The image filter keeps the alt text and warns that the image took too long, suggesting to export it to PNG; the PDF is still made. |

Errors go to standard error as `Error: <message>`; warnings as
`Warning: <message>`. Every one is also written to the log
([`INVESTIGACION_LOG`](#environment-variables)).

## Environment variables

| Variable | Used for |
|---|---|
| `INVESTIGACION_HOME` | Project root (see above). |
| `INVESTIGACION_LOG` | Log level: `debug` records full commands and paths; `off` or `0` turns the log off; anything else (or unset) is `info`. |
| `INVESTIGACION_TIMING` | `1` prints `[timing] <tool>: <seconds> s` to standard error after each Pandoc/pdflatex/Graphviz run. |
| `IDIOMA`, `INTERFACE_LANGUAGE` | Interface language (`es` or `en`). `IDIOMA` is checked first. |
| `CARPETA_TRABAJO`, `INVESTIGACION_WORKSPACE` | Work folder with `input/`, `output/`, `courses/`, `my-templates/` and `logos/`. |
| `LOGOS` | Folder with the logos, when `--logos` is not given. |
| `TEXINPUTS` | Your own value is kept and appended after the project's folders when pdflatex runs. |
| `HOME`, `USERPROFILE` | Used to expand `~` in paths. |
| `INVESTIGACION_CONVERTER_TIMEOUT_MS` | Only for the tests: a shorter limit (in ms) for `--convert-image` instead of 30 s. |

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
and color theme and, if you want, the logos folder. Despite the name, it is **not** TOML:
it has one `KEY=value` per line.

```
# comments start with #
IDIOMA="en"
TEMA="nord"
LOGOS="~/logos"
```

- Accepted keys: `IDIOMA` (or `INTERFACE_LANGUAGE`), `TEMA` (or `THEME`),
  `CARPETA_TRABAJO` (or `INVESTIGACION_WORKSPACE`) and `LOGOS`.
- `CARPETA_TRABAJO` is the work folder the menu asks for on the first run.
  `input/`, `output/`, `courses/`, `my-templates/` and `logos/` are read there
  (and relative paths in profiles start there). Without it, the program folder
  is used.
- An optional `export ` prefix and single or double quotes are accepted.
  A comment after an unquoted value needs a space before the `#`.
- A line without `=`, or a key that is not a plain name, is an error.
- The menu writes `IDIOMA` here when you choose a language (first run, or
  Options → l), and `TEMA` when you choose a color theme (Options → t):
  `classic`, `dracula`, `nord`, `gruvbox`, `catppuccin` or `tokyo-night`.
- The cover data is **not** stored here: it lives in the course profiles.
- Use another file with `--settings`.

## Examples

All of them use placeholder data. `Tarea1.md` can live in the current folder
or in `input/`.

**With a profile** (the profile gives course, university, design…):

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
  --teacher "Nombre del docente" --group "7-A"
```

**A team paper** (members replace the student on the cover):

```bash
investigacion Tarea1.md -p example --title "Tema" --members "Ana Ruiz, Luis Paz"
```

**A design from a file of your own:**

```bash
investigacion Tarea1.md -p example --title "Tema" --design my-templates/designs/mine/template.ltx
```

**An English document** (the PDF language), with English messages:

```bash
investigacion Essay.md -p example --title "On Graph Theory" --doc-lang en --lang en
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
investigacion --check-template=mine --lang en
```

**Diagnosing a problem** (full log in `cache/logs/investigacion.log`, and the
time of each tool):

```bash
INVESTIGACION_LOG=debug INVESTIGACION_TIMING=1 investigacion Tarea1.md -p example --title "Tema"
```
