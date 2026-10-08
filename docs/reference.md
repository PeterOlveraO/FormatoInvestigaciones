# Technical reference

Complete reference of `investigacion` (version 2.0.0): a Rust CLI and
full-screen menu that turns a paper written in Markdown into a PDF through
Pandoc, a LaTeX template (format + design) and `pdflatex`.

**Supported platforms:** Linux and Windows. macOS is planned for a later
update; the code follows the cross-platform rules of section 21 so that port
stays small.

This page is written for developers and AI assistants that need to use,
analyze or extend the program. It is checked against the code under `src/`,
`templates/` and `resources/filters/`. When this page and the code disagree,
the code wins. Related files:

| File | Content |
|---|---|
| [`cli.md`](cli.md) | Every command-line option, in detail |
| [`../INSTALL.md`](../INSTALL.md) | Installing the tools on each operating system |
| [`../AGENTS.md`](../AGENTS.md) | Project rules for contributors and coding agents |
| [`prompts/paper.md`](prompts/paper.md) | Prompt that tells an AI how to write a paper for this program |
| [`prompts/design.md`](prompts/design.md) | Prompt that tells an AI how to make a compatible design |
| [`../CHANGELOG.md`](../CHANGELOG.md) | User-visible changes |
| [`../courses/README.md`](../courses/README.md), [`../courses/example.toml`](../courses/example.toml) | Profile file reference and an example |
| [`../examples/catalog.md`](../examples/catalog.md), [`../examples/syntax.md`](../examples/syntax.md) | Papers that exercise every supported element |

## Table of contents

1. [Overview](#1-overview)
2. [System requirements and dependencies](#2-system-requirements-and-dependencies)
3. [Project layout and privacy](#3-project-layout-and-privacy)
4. [Pipeline](#4-pipeline)
5. [Data model and precedence](#5-data-model-and-precedence)
6. [Settings and environment variables](#6-settings-and-environment-variables)
7. [Course profiles and the wizard](#7-course-profiles-and-the-wizard)
8. [Formats and designs](#8-formats-and-designs)
9. [The design contract, markers and custom fields](#9-the-design-contract-markers-and-custom-fields)
10. [Checking a design: `--check-template`](#10-checking-a-design---check-template)
11. [Making your own design or format](#11-making-your-own-design-or-format)
12. [Writing the paper: Markdown syntax](#12-writing-the-paper-markdown-syntax)
13. [Lua filters](#13-lua-filters)
14. [Reading, cleaning and finding the Markdown](#14-reading-cleaning-and-finding-the-markdown)
15. [The menu (TUI)](#15-the-menu-tui)
16. [The command line](#16-the-command-line)
17. [Compilation](#17-compilation)
18. [Logging](#18-logging)
19. [Errors, reporting and robustness](#19-errors-reporting-and-robustness)
20. [Internationalization](#20-internationalization)
21. [Cross-platform rules](#21-cross-platform-rules)
22. [Source map and how to extend](#22-source-map-and-how-to-extend)
23. [Testing and regression rules](#23-testing-and-regression-rules)
24. [Troubleshooting](#24-troubleshooting)
25. [Glossary](#25-glossary)

---

## 1. Overview

`investigacion` reads one Markdown file and produces one PDF:

```
paper.md  ->  investigacion  ->  output/paper.pdf
```

- **Binary:** `investigacion` (`src/main.rs`). With no arguments it opens the
  full-screen menu; with arguments it is a normal CLI.
- **Built-in formats** (the norm): `apa7` (default), `harvard`, `mla`.
- **Built-in designs** (cover and look): `geometric-cover` (default),
  `classic-cover`, `report`, `starter`.
- **Languages:** the interface (menu, messages, help) is Spanish or English.
  The PDF language is separate (the "document language"); papers are Spanish
  by default.
- **What the author writes:** only the body. The program adds the cover or
  title block, the table of contents and all formatting.
- **What the program never needs:** hand-written LaTeX (raw LaTeX is off by
  default) or a network connection (except to download web images the first
  time).
- **Always produce a PDF.** A missing image, an unknown symbol or a missing
  Graphviz becomes a warning. The intended hard failures are a broken
  `pgfplot`/`tikz` block ([section 12](#charts-pgfplot-tikz)) and broken math
  (`$\frac{1}{$`, see [Math](#math)), because both are real LaTeX.
- **License:** GPL-3.0-or-later. Minimum Rust 1.88 (edition 2024, let-chains).

## 2. System requirements and dependencies

### External programs

| Tool | Required | Used for | If missing |
|---|---|---|---|
| Pandoc 3.0+ (tested with 3.10.2) | yes | Markdown to LaTeX fragment | Error: "Pandoc was not found. Install it and make sure it is on the PATH." If Pandoc fails and is older than 3.0 (the filters use `pandoc.Figure`): "Pandoc 3.0 or newer is needed and 2.9.2.1 is installed. INSTALL.md has the steps." |
| `pdflatex` (TeX Live or MiKTeX) | yes | Typesetting | Error: "pdflatex was not found. Install TeX Live (or MiKTeX) and make sure it is on the PATH." |
| Graphviz (`dot`) | no | ` ```dot ` diagrams | Warning; the diagram stays as a code block |
| Rust 1.88+ | build time only | `cargo install --path .` | n/a |

LaTeX packages the templates load (installers are in
[`../INSTALL.md`](../INSTALL.md)): `newtx`, `pgfplots`, `pgf-pie`,
`twemojis`, `pmboxdraw`, `floatrow`, `newunicodechar`, `ulem`, `framed`,
`footnotehyper`, `xurl`, `titlesec`, `ragged2e`, `babel-spanish`, plus the
usual `fancyhdr`, `booktabs`, `longtable`, `caption`, `etoolbox`, `soul`,
`upquote`, `hyperref`, `bookmark`, `tikz`. The base loads optional packages
with `\IfFileExists`, so a missing optional package degrades a feature
instead of breaking the build.

### Rust crates (`Cargo.toml`)

| Crate | Why it is needed |
|---|---|
| `clap` (derive) | Argument parsing; help and errors are re-localized in `src/cli.rs` |
| `ratatui`, `crossterm` | The menu; same behavior on Linux, macOS and Windows |
| `serde`, `toml` | Manifests (`format.toml`, `template.toml`) and course profiles |
| `regex` | Marker extraction, Markdown cleanup, log parsing, settings keys |
| `chrono` | Delivery date and log timestamps |
| `tempfile` | Build folder, filter folder, check output |
| `thiserror` | `GenerationError` |
| `opener` | Open the PDF and project folders from the menu |
| `sys-locale` | Detect the operating-system language |
| `unicode-normalization` | Strip accents (`slugify`, heading comparison) |

New crates need a justification (project rule: minimal dependencies).

Release profile: `lto = "thin"`, `codegen-units = 1`,
`strip = "debuginfo"` (function names are kept so a panic backtrace in the log
is readable, about +0.6 MB).

## 3. Project layout and privacy

The **project root** is found by `Project::discover()` (`src/project.rs`):

1. the `INVESTIGACION_HOME` environment variable, if set and non-empty;
2. the first ancestor of the current directory that contains
   `templates/common/investigacion-base.sty`;
3. the folder the binary was compiled from (`CARGO_MANIFEST_DIR`), which is why
   `cargo install` works from any directory (moving the folder requires
   reinstalling or setting `INVESTIGACION_HOME`);
4. otherwise the current directory (nothing is written there: the log stays off
   when the root has no templates).

```
<root>/
  input/                    your Markdown papers (subfolder per course is common)
  output/                   generated PDFs (and last-error.tex/.log after a failed build)
  courses/                  course profiles: <key>.toml, example.toml, README.md
  settings.toml             remembered interface language (IDIOMA) and optional LOGOS
  templates/
    common/                 investigacion-base.sty, investigacion-final.sty (the base layer)
    formats/<f>/            format.sty + format.toml (the norm)
    designs/<d>/            template.ltx + template.toml (cover and look); optional logos/ and *.sty
    logos/                  logo-universidad.png, logo-facultad.png (+ README.md)
  my-templates/
    formats/<f>/            your own formats (same structure)
    designs/<d>/            your own designs (same structure)
  cache/
    latex/<pdf>-<hash>/     cached .aux/.toc of the last build of each PDF
    remote/                 downloaded web images (<sha1 of URL>.png|jpg|pdf)
    diagrams/               compiled Graphviz diagrams (<sha1 of code>.pdf)
    logs/                   investigacion.log (and investigacion.old.log)
  examples/                 Spanish sample papers; catalog.md is the visual test bench
  docs/                     documentation and README screenshots (docs/images/)
  resources/filters/        Lua filters (embedded into the binary at build time)
  src/, tests/              Rust code and integration tests
```

`cache/` is created on demand and is safe to delete entirely. Folders under
`cache/` are created by `Project::media_directories()` (images, diagrams), the
LaTeX state saver and the logger.

### What git ignores, and why

Personal data never goes into the repository (`.gitignore`):

| Ignored path | Reason |
|---|---|
| `.env` | Legacy file from earlier versions; ignored so it never leaks |
| `settings.toml` | Per-user settings |
| `/input/`, `/output/` | Your papers and PDFs |
| `/courses/*` except `example.toml` and `README.md` | Profiles hold teacher and student names |
| `/templates/logos/*` except `README.md`, and `/templates/*/logos/` | Institutional logos are rarely redistributable |
| `/my-templates/` | Your own formats and designs |
| `/cache/` | Regenerable; the log contains paths and titles |
| `last-error.tex`, `last-error.log` | Leftovers of failed builds |
| `/target/`, editor and OS files | Build output and noise |

Generated files tracked in git (`examples/catalog.pdf`, `docs/images/*.png`)
use the placeholders of `courses/example.toml` and no logos (see the
`/regenerate-catalog` skill). Never commit personal data or institutional
logos.

### Where output goes

- **PDF folder:** `--output` > `output/<folder>` of the profile (when it has a
  `folder`) > `output/`. A leading `~` is expanded (`HOME` or `USERPROFILE`).
- **PDF name:** `--file-name` (with or without `.pdf`, case-insensitive) or, by
  default, the Markdown file's name. Either is passed through `slugify`
  (accents removed, lowercase, non-alphanumerics become `-`; an empty result
  becomes `investigacion`). `Tarea2-3.md` produces `tarea2-3.pdf`;
  `--file-name "Entrega Final.pdf"` produces `entrega-final.pdf`. The title never
  affects the file name. Generating again replaces the PDF.
- **Copies:** `--copy <folder>` (repeatable) copies the PDF to each folder,
  skipping duplicates and the PDF's own folder. A destination that exists as a
  file is an error.
- **Write probe:** before compiling, the program creates and removes
  `.investigacion-write-test` in the output folder, so a permissions problem
  fails early and names the folder.

## 4. Pipeline

`cli::execute()` (shared by the CLI and the menu) calls `run_generation()`,
which calls `generate::generate_pdf()`. Steps in the order they actually run:

| # | Step | Function | Module | What it does |
|---|---|---|---|---|
| 1 | Settings | `Settings::load` | `src/settings.rs` | Reads `settings.toml` (or `--settings`) |
| 2 | Profile | `find_course` | `src/courses.rs` | Loads `courses/<key>.toml` when `-p` is given |
| 3 | Layout | `resolve_layout` | `src/template.rs` | Picks design and format; checks they are compatible |
| 4 | Data | `pick()` and `DocumentData` | `src/cli.rs`, `src/document.rs` | Merges CLI > profile; date in the document language |
| 5 | Required data | `missing_data` | `src/generate.rs` | Errors before reading any file if the design needs data that is empty |
| 6 | Locate and read | `locate_markdown`, `read_markdown` | `src/cli.rs`, `src/markdown.rs` | Finds the file (profile folder first), decodes UTF-8/Windows-1252 |
| 7 | Normalize | `normalize_markdown` | `src/markdown.rs` | Removes invisible spaces (part of `read_markdown`) |
| 8 | Validate | `validate_markdown` | `src/markdown.rs` | Warns about missing or out-of-order recommended headings |
| 9 | Re-resolve | `generate_pdf` | `src/generate.rs` | Checks the file is `.md` and non-empty; resolves layout and `missing_data` again; picks the logos folder |
| 10 | Template markers | `Layout::format_block`, `class_options` | `src/template.rs` | Replaces `%%FORMAT%%` and `%%CLASS_OPTIONS%%` in `template.ltx` |
| 11 | Pandoc | `pandoc_to_latex` | `src/pandoc.rs` | Markdown to a LaTeX **fragment** through the Lua filters |
| 12 | Render | `render_template` | `src/latex.rs` | Replaces the data markers (escaped), then inserts the Pandoc content last |
| 13 | Assets | `copy_template_assets` | `src/compile.rs` | Copies `.sty` files, the format and the logos to a temp folder |
| 14 | Compile | `compile_pdf` | `src/compile.rs` | `pdflatex` passes with the `.aux/.toc` cache |
| 15 | Copies | `copy_pdf_to` | `src/generate.rs` | Optional extra copies of the PDF |

Notes on the order:

- Steps 3 and 5 run twice (in `run_generation` and again in `generate_pdf`).
  The first run lets the CLI fail before touching the Markdown; the second
  keeps `generate_pdf` safe when it is called directly (as `--check-template`
  does).
- Step 8 only runs in `execute()`. `generate_pdf` and `--check-template` do not
  validate headings.
- `AGENTS.md` lists the same names in a simplified order
  (`read_markdown` -> `normalize_markdown` -> `resolve_layout` ->
  `missing_data` -> `validate_markdown`). In the code, layout and required data
  are checked first and the Markdown is read afterwards.
- Pandoc's output is a fragment (no preamble). The preamble comes from
  `template.ltx`, the base and the format.
- Pandoc content is **not** escaped and is inserted last, so a `%%X%%` written
  in the paper is never mistaken for a pending marker.
- The temp build folder (`investigacion-*`) is removed at the end; failing to
  remove it (Windows file locks) is not an error. Failed builds keep
  `last-error.tex/.log` next to the PDF ([section 17](#17-compilation)).

### The Pandoc call

```
pandoc - --from=markdown-raw_tex+mark+emoji+autolink_bare_uris --to=latex --wrap=none
       --resource-path=<md folder>:<cache>:<cache/remote>:<cache/diagrams>
       --request-header=User-Agent:investigacion/0.2 (academic paper generator; pandoc)
       --lua-filter=<tmp>/inline_html.lua --lua-filter=<tmp>/images.lua
       --lua-filter=<tmp>/diagrams.lua   --lua-filter=<tmp>/charts.lua
       --lua-filter=<tmp>/blocks.lua
```

- The Markdown goes in through stdin; the working directory is the Markdown's
  folder.
- `--from=markdown` (with `raw_tex`) replaces `markdown-raw_tex` when
  `--allow-latex` is given.
- The resource-path separator comes from `std::env::join_paths` (`:` or `;`).
- Environment variables set for Pandoc and its filters:
  `INVESTIGACION_REMOTE_IMAGES`, `INVESTIGACION_DIAGRAMS`,
  `INVESTIGACION_RESOURCES` (one folder per line), `INVESTIGACION_LANG`
  (interface language) and `INVESTIGACION_DOC_LANG` (document language). Paths
  go through `pandoc::posix()` (always `/`).
- Filters are embedded with `include_str!` (`LUA_FILTERS` in
  `src/pandoc.rs`) and written to a temp folder on each run.
- Timeout: 180 s (`TOOL_TIMEOUT`). On a non-zero exit the stderr text is shown
  after "Pandoc could not convert the Markdown".

## 5. Data model and precedence

### Precedence

```
command-line option  >  course profile
```

- There is **no `.env` any more**. All cover data comes from the profile or from
  options. `settings.toml` only remembers the interface language (`IDIOMA`) and
  an optional logos folder (`LOGOS`).
- `pick(option, profile)` returns the option when it is present (even if it is
  an empty string) and trims the result. An explicitly empty option therefore
  **overrides** the profile: `--course ""` or `--set UNIVERSIDAD=` blanks the
  value (and `missing_data` then complains if the design requires it). The menu
  never passes empty strings: an empty form field becomes `None`, so the
  profile applies.
- Design and format: option, then profile, then the design's first listed
  format, then `apa7`.
- Document language: `--doc-lang`, then the profile's `language`, then the
  format's `language`, then Spanish.

### `DocumentData` (`src/document.rs`)

| Field | Marker | Filled from (CLI) | Filled from (profile) |
|---|---|---|---|
| `title` | `TITULO` | `--title` (required by clap) | none |
| `university` | `UNIVERSIDAD` | `--set UNIVERSIDAD=...` | `university` |
| `faculty` | `FACULTAD` | `--set FACULTAD=...` | `faculty` |
| `student` | `ALUMNO` | `--set ALUMNO=...` | `student` |
| `semester` | `SEMESTRE` | `--set SEMESTRE=...` | `semester` |
| `course` | `MATERIA` | `--course` | `name` |
| `teacher` | `DOCENTE` | `--teacher`, then `--set DOCENTE=...` | `teacher` |
| `members` | `INTEGRANTES` | `--members`, then `--set INTEGRANTES=...` | `members` |
| `group` | `GRUPO` | `--group`, then `--set GRUPO=...` | `group` |
| `date` | `FECHA_ENTREGA` | today, in the document language | none |
| `fields` | custom markers | `--set NAME=value` (name uppercased) | `[fields]` table |

- `--set` also accepts the names of the standard data above (it is how a run
  without a profile supplies university, faculty, student or semester). Every
  `--set` pair also goes into `fields`; unused entries are ignored.
- `--set MATERIA=...` is **not** read; use `--course`.
- Members: one string split on `,` or `;`, trimmed, empty names dropped
  (`"Ana,,Luis,"` is two names). Members **replace** the student on the cover and
  in the PDF metadata.
- Custom field maps: profile keys are uppercased; `--set` entries are applied
  on top, so the CLI wins.
- Date format: `<Month> <day>, <year>`; Spanish months are capitalized
  (`Octubre 3, 2026`), English are `October 3, 2026`.

## 6. Settings and environment variables

### `settings.toml`

The file is named `.toml` but is parsed by `src/settings.rs` as **`KEY=VALUE`
lines** (a dotenv-like syntax), not as TOML:

- blank lines and lines starting with `#` are ignored;
- an optional `export ` prefix is accepted;
- values may be quoted with `"` or `'`; an unquoted value ends at ` #`
  (space, hash);
- a key must match `[A-Za-z_][A-Za-z0-9_]*`;
- any other line (for example a TOML `[table]` header) is an error: "Invalid
  syntax in <file>, line N: expected KEY=VALUE".

Keys the program reads:

| Key | Meaning |
|---|---|
| `IDIOMA` (alias `INTERFACE_LANGUAGE`) | Interface language: `es`/`en` (also `español`, `inglés`, `spanish`, `english`) |
| `LOGOS` | Folder with the logos, used when `--logos` is not given |

- A process environment variable with the same name wins over the file. The
  process environment is never modified.
- The menu writes `IDIOMA="es"` or `IDIOMA="en"` with `Settings::save_value`,
  which replaces that key's line (with or without `export`) or appends it,
  keeping every other line and comment.
- `--settings <file>` (old name `--env-file`) selects another file.
- A missing file is not an error. An unreadable one is logged; the CLI ignores
  it when it only needs the language, but `execute()` reports it as an error.

### Environment variables

| Variable | Read by | Meaning |
|---|---|---|
| `INVESTIGACION_HOME` | `Project::discover` | Project root |
| `INVESTIGACION_LOG` | `logging` | `debug`, `off`/`0`; anything else or unset is `info` ([section 18](#18-logging)) |
| `INVESTIGACION_TIMING` | `process::report_timing` | `1` prints `[timing] <step>: <s>` to stderr |
| `IDIOMA`, `INTERFACE_LANGUAGE`, `LOGOS` | `Settings::get` | Override `settings.toml` |
| `TEXINPUTS` | `compile::latex_search_path` | `.` (the temp folder), the Markdown folder and the image caches first; existing value preserved after them |
| `HOME` / `USERPROFILE` | `expand_home` | `~` expansion |
| `INVESTIGACION_REMOTE_IMAGES`, `INVESTIGACION_DIAGRAMS`, `INVESTIGACION_RESOURCES`, `INVESTIGACION_LANG`, `INVESTIGACION_DOC_LANG` | Lua filters | **Set by the program** for Pandoc; do not set by hand |

## 7. Course profiles and the wizard

A profile (`courses/<key>.toml`) keeps everything for one course: format,
design and all the cover data. Two courses can use different formats,
teachers or universities. The file name (without `.toml`) is the profile's
key: `ia.toml` is used with `-p ia`.

### Schema (`CourseProfile` in `src/courses.rs`)

All keys are strings except `fields`. Unknown keys are an error
(`#[serde(deny_unknown_fields)]`), so a typo is reported instead of ignored.

| Key | Required | Meaning | Marker it fills |
|---|---|---|---|
| `name` (alias `subject`) | yes (non-empty) | Course name shown on the cover | `MATERIA` |
| `university` | no | University | `UNIVERSIDAD` |
| `faculty` | no | Faculty or school | `FACULTAD` |
| `student` | no | Student (individual papers) | `ALUMNO` |
| `members` | no | Team members, separated by commas (or `;`); replace the student | `INTEGRANTES` |
| `teacher` | no | Teacher | `DOCENTE` |
| `group` | no | Group, for example `7-A` | `GRUPO` |
| `semester` | no | Term, for example `2026-2` | `SEMESTRE` |
| `format` | no | `apa7`, `harvard`, `mla` or one of yours | n/a |
| `design` (alias `template`) | no | `geometric-cover`, `classic-cover`, `report`, `starter` or one of yours; legacy `apa`/`apa-simple` still work | n/a |
| `language` | no | Document language `es` or `en`; default is the format's | n/a |
| `folder` | no | Subfolder of `input/` (lookup) and `output/` (PDFs) | n/a |
| `[fields]` | no | Table of the design's own fields, for example `SALON = "B-204"` | custom markers |

Example (placeholders; this is `courses/example.toml`):

```toml
name = "Nombre de la materia"            # required
university = "Nombre de la universidad"
faculty = "Nombre de la facultad"
student = "Nombre del alumno"
# members = "Ana Ruiz, Luis Paz"         # replaces the student
teacher = "Nombre del docente"
group = "7-A"
semester = "2026-2"
format = "apa7"
design = "geometric-cover"
# language = "es"                        # default: the format's
folder = "IA"                            # input/IA and output/IA

# [fields]
# SALON = "B-204"
```

Behavior:

- **Lookup** (`find_course`): `courses/<choice>.toml`; else a key that matches
  case-insensitively; else `<choice>` as a path to a file. Failure lists the
  available keys.
- **Validation:** TOML errors (the detail text comes from the `toml` crate and
  is in English) and an empty `name` are reported with the file path. A broken
  profile does not hide the others: `list_courses` returns the valid ones and
  the errors apart, and the menu shows the errors as warnings.
- **Old profiles:** `subject = ...` and `template = "apa"` still work.
- **`folder`:** the Markdown is looked up in `input/<folder>` first (only if
  that directory exists), and the PDF goes to `output/<folder>`.
- **Fields not used by the design** are silently ignored.
- **Saving** (`save_profile`): writes only non-empty keys, in the order
  `name, university, faculty, student, members, teacher, group, semester,
  format, design, language, folder`, then `[fields]`. The key is `slugify`'d.
  An existing file with the same key is replaced.
- **Privacy:** `courses/*.toml` is git-ignored except `example.toml`.
- **`example.toml`** holds placeholders. It is not listed in the menu and does
  not count as a profile (so the wizard opens on the first run), but
  `-p example` still works on the command line.

### The profile wizard (`src/tui/wizard.rs`)

Opened automatically the first time (no profiles), from the home view
(**New profile**) and from the options (`p` edits the profile chosen in the
form, or creates one if none is chosen; `n` creates an empty one).

Steps, in order. Each step shows "Step N of M"; required steps reject an empty
answer with "This field is required."

| # | Step | Widget | Required |
|---|---|---|---|
| 1 | Profile name (`Key`) | text, saved as `slugify(value)` | yes |
| 2 | Format | list (type to filter) | yes |
| 3 | Design | list, only designs that accept the format (or self-contained) | yes |
| 4+ | One step per **standard marker the design uses**, in this order: `UNIVERSIDAD`, `FACULTAD`, `ALUMNO`, `INTEGRANTES`, `MATERIA`, `DOCENTE`, `GRUPO`, `SEMESTRE` | text | `UNIVERSIDAD`, `FACULTAD`, `SEMESTRE`, `MATERIA` unless the design lists them in `optional`; others never |
| ... | One step per **custom field** of the design (label and help from `template.toml`) | text | only if `required = true` |
| last | Papers folder (`folder`) | text | no |

Keys: Enter accepts and moves on, Shift+Tab goes back one step, Esc cancels,
Backspace/Delete/Left/Right/Home/End edit text, Up/Down/typing work in lists.

Details:

- The steps after the design are rebuilt when the design changes. Changing
  the format clears the design (and the steps after it) if the design does not
  accept the new format.
- When editing an existing profile, every answer is pre-filled and the data
  steps are shown immediately. The key can be edited; saving under a new key
  creates a new file and leaves the old one.
- If the design does not use `MATERIA`, there is no course step and the profile
  `name` falls back to the key (a profile always needs `name`). If the design
  uses `MATERIA` but lists it in `optional` and the answer is empty, `name` also
  falls back to the key, which then appears as the course name.
- On finish: `courses::save_profile` writes the file, the profile list is
  reloaded and the new profile is selected in the form ("Profile saved: ...").

## 8. Formats and designs

A PDF combines two things chosen separately, on top of a universal base:

```
DESIGN  templates/designs/<d>/template.ltx + template.toml   cover or title block, look, custom fields
FORMAT  templates/formats/<f>/format.sty  + format.toml      the norm: font, spacing, headings, captions, references, page number
BASE    templates/common/investigacion-base.sty / -final.sty universal: Pandoc compatibility, diagrams, charts, Unicode
```

- **Base** (`templates/common/`) is the same for everything.
- **Format** decides how the paper is typeset.
- **Design** decides the cover and what data is shown.
- A design lists the formats it works with; a design without `%%FORMAT%%` is
  **self-contained** and carries all its own rules (no format is applied).
- User-made formats and designs live in `my-templates/formats|designs/`. The
  search order is `templates/<kind>/` first, then `my-templates/<kind>/`, so a
  user folder **cannot replace** a built-in one with the same name; pick a
  different name.
- Layout resolution (`resolve_layout`): load the design (name, folder, or path
  to a `.ltx`), then, unless self-contained, choose the format: the given one,
  else the first in the design's `formats`, else `apa7`. If the design does not
  accept it: "The design X does not work with the format Y. Compatible formats:
  ...".

### Built-in formats

All three: 12 pt Times (`newtx`), US letter (`class_options = "12pt,
letterpaper"`), single column, document language Spanish by default.

| Format | Spacing, indent | Margins | Headings | Captions | Page number | References | Recommended headings |
|---|---|---|---|---|---|---|---|
| `apa7` (default) | double, 1.27 cm indent, no paragraph skip | 2.54 cm | unnumbered; `#` centered bold, `##` left bold, `###` left bold italic, `####` run-in bold ending with a period, `#####` run-in bold italic | table and figure: bold label ("Tabla 1"/"Figura 1") on its own line, italic title below, figure caption on top | top right | hanging indent 1.27 cm | `Introducción`, `Desarrollo`, `Conclusión`, `Referencias` |
| `harvard` | 1.5, no indent, 0.8 em paragraph skip | 2.5 cm | numbered `1`, `1.1`, `1.1.1`; `#` large bold, `##` bold, `###` bold italic | bold label with colon ("Tabla 1:"); table title above, figure caption below | bottom center | hanging indent 1.27 cm | none |
| `mla` | double, 0.5 in indent | 1 in | unnumbered, text size; `#` bold, `##` italic, `###` bold italic | label with period; figures named "Fig." with the caption below; table title above | top right | hanging indent 0.5 in ("Works Cited") | none |

Details worth knowing:

- APA charts use a grayscale `pgfplots` style (`every axis/.append style`,
  `cycle list`, `bar cycle list`); Harvard and MLA leave the base defaults.
- Harvard and MLA are generic versions written from their published guides. If
  a school asks for something different, copy the format folder into
  `my-templates/formats/<name>/` and adjust it.
- Table of contents depth is 3 in all formats. APA renames "Índice" to
  "Contenido" when the document language is Spanish.
- Long tables are set at single spacing in all formats (`\AtBeginEnvironment{longtable}{\singlespacing}`).

### Built-in designs

| Design | Look | Formats | Standard data it shows | Required (default) | Logos |
|---|---|---|---|---|---|
| `geometric-cover` (default) | Constructivist cover: TikZ lines, arcs and small squares over the whole page; sans-serif text; then the table of contents | `apa7`, `harvard`, `mla` | `UNIVERSIDAD`, `FACULTAD`, `TITULO`, `ALUMNO`/`INTEGRANTES`, `MATERIA`, `DOCENTE`, `GRUPO`, `SEMESTRE`, `FECHA_ENTREGA` | `TITULO`, `UNIVERSIDAD`, `FACULTAD`, `SEMESTRE`, `MATERIA` | yes (university top-left 1.6 cm high, faculty top-right 0.88 cm high) |
| `classic-cover` | Centered title page without TikZ; labels "Alumno:", "Integrantes:", "Materia:", "Docente:", "Semestre:", "Grupo:", "Fecha de entrega:"; then the table of contents | `apa7`, `harvard`, `mla` | same as `geometric-cover` | same | yes (university 2 cm high, faculty 1.2 cm high) |
| `report` | Compact title block (no cover page) followed by the table of contents | `apa7`, `harvard`, `mla` | `TITULO`, `ALUMNO`/`INTEGRANTES`, `MATERIA`, `DOCENTE`, `GRUPO`, `FACULTAD`, `UNIVERSIDAD`, `FECHA_ENTREGA` | `TITULO` (the manifest sets `optional = ["UNIVERSIDAD", "FACULTAD", "MATERIA"]`) | no |
| `starter` | Minimal, heavily commented design to copy; centered title block, no table of contents | `apa7`, `harvard`, `mla` | `TITULO`, `ALUMNO`/`INTEGRANTES`, `MATERIA`, `DOCENTE`, `FECHA_ENTREGA` | `TITULO` (`optional = ["MATERIA"]`) | no |

Cover rules shared by the built-in covers:

- **Empty lines disappear:** student, team, teacher and group are omitted when
  empty; the course and the semester always appear in the covers.
- **Order is fixed in the design:** student (or team), course, teacher,
  semester, group, then the delivery date.
- **Team members replace the student,** one name per line. The PDF metadata
  follow the same rule (`\AutorPDF` in `investigacion-final.sty`).
- **Fixed labels belong to the design** and stay in Spanish (for example
  "FECHA DE ENTREGA") whatever the document language is.
- `geometric-cover` and `classic-cover` set the page counter to 2 after the
  cover (APA counts the cover as page 1).
- Logos are optional; without them the cover compiles unchanged.

### Legacy design names

`apa` maps to `geometric-cover` and `apa-simple` to `classic-cover`
(`project::legacy_design`). Because both list `apa7` first, they keep
producing the same output as before. The profile key `template` and the
`--template`/`--plantilla` options still work.

### Document language vs interface language

- The **document language** drives the PDF: babel (`spanish, es-tabla` or
  `english`), the delivery date, "Contents"/"Contenido", "Figure"/"Figura",
  "Table"/"Tabla" and the note-box titles.
- It comes from `--doc-lang`, then the profile's `language`, then the format's
  `language` (all three built-ins: `es`), then Spanish.
- The **interface language** (`--lang`, `IDIOMA`) controls menus, messages and
  help only. The two are independent.

### `format.toml` (`FormatManifest`)

Optional: a format only needs `format.sty`. Without `format.toml` the name is
the folder name, language `es`, no class options and no headings. Unknown keys
are errors.

```toml
name = { es = "APA 7", en = "APA 7" }     # required; plain string or { es, en }
language = "es"                           # default document language; default "es"
class_options = "12pt, letterpaper"       # inserted at %%CLASS_OPTIONS%%
headings = ["Introducción", "Desarrollo", "Conclusión", "Referencias"]   # [] = no structure check
```

### `template.toml` (`DesignManifest`)

Optional. Without it the design accepts every format and has no described
fields. Unknown keys are errors.

```toml
description = { es = "Mi portada", en = "My cover" }   # shown in the pickers
formats = ["apa7", "harvard", "mla"]   # empty = any; the first one is the default
optional = ["UNIVERSIDAD"]             # standard markers it shows but does not require
[fields.SALON]                         # description of a custom field
label = { es = "Salón", en = "Room" }
help  = { es = "Aula de entrega", en = "Delivery room" }
required = true                        # default false
```

Localized values (`Localized`) are either a plain string or `{ es, en }`.

## 9. The design contract, markers and custom fields

A design is `template.ltx` (a LaTeX file with `%%MARKERS%%`) and, optionally,
`template.toml`. It must follow this contract (`--check-template` verifies
it):

```latex
\documentclass[%%CLASS_OPTIONS%%]{article}   % the format sets font size and paper
\usepackage{investigacion-base}               % required, first
%%FORMAT%%                                    % the program inserts babel + the format
\newcommand{\TituloTrabajo}{%%TITULO%%}       % data (optional; final.sty provides empty defaults)
\usepackage{investigacion-final}              % required, last in the preamble
\begin{document}
  ... cover or title block ...
  \FormatBodySetup                            % before the body
  %%CONTENIDO_MARKDOWN%%                      % where the paper goes
\end{document}
```

- `%%FORMAT%%` becomes two lines:
  `\usepackage[<babel>]{babel}` and `\usepackage{investigacion-format}`, where
  `<babel>` is `spanish, es-tabla` for Spanish and `english` for English.
- A design without `%%FORMAT%%` is self-contained and gets no format and no
  babel line (it must set up its own language).
- `\FormatBodySetup` is a hook the base defines empty and each format redefines
  (alignment and indent after the cover).
- `ReferenceList` is an environment: neutral in the base, hanging indent in the
  three formats. The `blocks` filter wraps the references section in it.
- `CajaMarcada{title}` is the note-box environment (a thin black frame, no
  color) defined in the base.
- `investigacion-final.sty` loads `hyperref`, `bookmark`, `xurl`,
  `footnotehyper`, removes red link borders, `\providecommand`s
  `\TituloTrabajo`, `\ListaIntegrantes` and `\NombreAlumno`, defines
  `\AutorPDF` (members, else student) and sets the PDF title, author and
  subject. It must be last in the preamble.
- **Files the program copies** to the temp build folder (first in
  `TEXINPUTS`): every `*.sty` of `templates/common/`, every `*.sty` next to
  the design, the chosen `format.sty` renamed to `investigacion-format.sty`, and
  **every file** of the logos folder. The build runs with the Markdown's folder
  as working directory.
- **Inside a `.sty`:** no `\makeatletter` (`@` is already a letter); load
  packages with `\RequirePackage`.
- Anything Spanish-only in a format must be guarded for English, for example
  `\@ifundefined{captionsspanish}{}{\addto\captionsspanish{...}}`.

### Standard markers (`STANDARD_MARKERS`)

| Marker | Replaced by | Escaped | Notes |
|---|---|---|---|
| `TITULO` | Title (`--title`) | yes | Required if the design uses it and does not list it in `optional` |
| `UNIVERSIDAD` | University | yes | Required if used and not optional |
| `FACULTAD` | Faculty | yes | Required if used and not optional |
| `ALUMNO` | Student | yes | Never required |
| `INTEGRANTES` | Members | each name escaped, joined with a literal `\\` | Never required; design shows them instead of the student |
| `MATERIA` | Course (`--course` or profile `name`) | yes | Required if used and not optional |
| `DOCENTE` | Teacher | yes | Never required |
| `GRUPO` | Group | yes | Never required |
| `SEMESTRE` | Semester | yes | Required if used and not optional |
| `FECHA_ENTREGA` | Today's date in the document language | yes | Computed; never required |
| `CONTENIDO_MARKDOWN` | The Pandoc LaTeX fragment | **no** | Required by the contract; inserted last |
| `FORMAT` | babel + format package lines | n/a | Replaced before rendering; absence means self-contained |
| `CLASS_OPTIONS` | `class_options` of the format | n/a | Replaced before rendering |

### Custom fields

- Any `%%NAME%%` that is not a standard marker is a custom field. Names match
  `[A-Z_]+` (uppercase letters and underscore; **no digits**).
- Value: `DocumentData.fields[NAME]` (profile `[fields]`, `--set NAME=value`,
  menu extra fields), escaped with `latex_escape`; empty if not given.
- `template.toml` may describe it (`label`, `help`, `required`). Without a
  description the menu and the wizard use the bare name.
- The menu appends one extra form field per custom field after the fixed
  fields; the wizard asks for them after the standard data.
- Marker extraction (`extract_markers`) ignores any line whose first
  non-blank characters are `%` followed by something other than another `%`
  (a normal LaTeX comment). A line that starts with `%%` is scanned, which is
  how a standalone `%%FORMAT%%` line is found. Replacement, however, is
  textual over the whole file, including comments, so never write
  `%%FORMAT%%` (which expands to two lines) inside a comment.

### Required-data rules (`missing_data` in `src/generate.rs`)

- Standard data: only `TITULO`, `UNIVERSIDAD`, `FACULTAD`, `SEMESTRE`, `MATERIA`
  can be required, and only if the design **uses** the marker and does **not**
  list it in `optional`.
- Custom fields: required only when `[fields.NAME] required = true`.
- Whitespace-only values count as empty.
- Failure message: "Missing data used by the design <design>: <list>. Add them
  to the profile (Options -> p in the menu) or with --set NAME=value."
- `ALUMNO`, `INTEGRANTES`, `DOCENTE`, `GRUPO` and `FECHA_ENTREGA` are never
  required: the design hides their lines when empty.

### The base layer (`templates/common/investigacion-base.sty`)

- **Pandoc contract.** The block "PAQUETES QUE NECESITA LA SALIDA DE PANDOC"
  provides what Pandoc's LaTeX output assumes: `calc`, `\newcounter{none}`, the
  `longtable` patch, `\pandocbounded`, `Shaded` and the `\...Tok` highlighting
  commands, strikethrough and underline (`\st`, `\ul` redefined on `ulem`),
  highlight (`\hl`), `amsmath`, `footnotehyper`, `\tightlist`. **Remove
  nothing.** Compare with `pandoc f.md -s --to=latex`.
- **Traps:**
  - `soul` loops forever inside `longtable` and breaks in headings; `\st` and
    `\ul` are therefore redefined with `\DeclareRobustCommand` on `ulem`, and
    `\hl` becomes a `\colorbox` inside `longtable`.
  - `floatrow` and `float` conflict: never load `float`. The `H` placement
    line (`\@ifundefined{floatsetup}...\fps@figure`) must stay where Pandoc
    sets its own.
  - `\LTleft`/`\LTright` are set to `\fill` so long tables stay centered.
  - Babel caption renames must be wrapped in `\addto\captions<lang>`.
  - Package order: `hyperref`, then `footnotehyper`, at the end (in
    `investigacion-final.sty`).
  - `verbatim` blocks get `\singlespacing\small`.
- **Charts:** loads `pgfplots` (compat 1.18, `statistics` library),
  `pgfplotstable` and `pgf-pie` when present. Styles belong to the format.
  Never put `fill` in a `cycle list`.
- **Unicode:** `SÍMBOLOS UNICODE` declares (with `newunicodechar`) only the
  symbols that really failed (relations, arrows, sets, calculus, Greek,
  checkmarks, geometric shapes). Emoji are typeset as images through
  `twemojis` using the code point `inputenc` computed; variation selector
  U+FE0F and U+200D are declared empty. Anything else triggers
  `\UTFviii@undefined@err`, which writes the warning "Caracter Unicode sin
  definir: <char> (U+XXXX)" into the log and prints a bold `[?]`. Rust matches
  that exact text in `unsupported_character_warnings`; keep it identical.
  To support a symbol, add it to that block.
- **Logos** are wrapped in `\IfFileExists`.

## 10. Checking a design: `--check-template`

```bash
investigacion --check-template <design>      # also --check-template=<design>; alias --revisar-plantilla
```

Handled in `cli::main_with_args` **before** clap parsing (it needs no Markdown
or title) and implemented in `src/check.rs`. `<design>` is a name, a folder or a
path to a `.ltx` (legacy names work). A value that starts with `-` is not taken
as the name. Exit code: 0 passed, 1 failed (or the design could not be loaded),
2 when the name is missing.

Static checks (`inspect`):

| Result | Condition |
|---|---|
| Error | `\usepackage{investigacion-base}` missing |
| Error | `\usepackage{investigacion-final}` missing |
| Error | `%%CONTENIDO_MARKDOWN%%` missing |
| Warning | No `%%FORMAT%%`: self-contained design, no format applies |
| Warning | Has `%%FORMAT%%` but no `%%CLASS_OPTIONS%%`: the format cannot set font size or paper |
| Error | A format in `template.toml` `formats` does not exist (lists the available ones) |
| Warning | A custom field looks like a typo of a standard marker (edit distance <= 2): "did you mean `%%DOCENTE%%`?" |
| Warning | A custom field is not described in `template.toml` |
| Warning | `template.toml` describes a field the design does not use |
| Warning | `optional` mentions a marker the design does not use |

If there are errors, the check stops there. Otherwise it compiles
`examples/catalog.md` (skipped, with a warning, if the file is missing) once
per compatible format (or once with no format for a self-contained design)
using sample data: every standard datum filled, two members, every custom
field set to "Ejemplo". The output goes to a temp folder that is deleted. A
build failure shows the first four lines of its error.

Output symbols: `✗` for errors, `!` for warnings, `✓` for passing builds, then "Done: the design is ready to use." Failure exits 1.

## 11. Making your own design or format

### A design

1. Copy `templates/designs/starter/` to `my-templates/designs/<your-design>/`
   (git ignores it).
2. Edit the cover between the `TU DISEÑO` marks. Keep the contract
   ([section 9](#9-the-design-contract-markers-and-custom-fields)):
   `\documentclass[%%CLASS_OPTIONS%%]{article}`, `investigacion-base` first,
   `%%FORMAT%%`, `investigacion-final` last in the preamble, `\FormatBodySetup`
   and `%%CONTENIDO_MARKDOWN%%` in the body.
3. Use only the data you need. Standard markers: `%%TITULO%%`,
   `%%UNIVERSIDAD%%`, `%%FACULTAD%%`, `%%ALUMNO%%`, `%%INTEGRANTES%%`,
   `%%MATERIA%%`, `%%DOCENTE%%`, `%%GRUPO%%`, `%%SEMESTRE%%`,
   `%%FECHA_ENTREGA%%`. Any other uppercase name (`%%SALON%%`) is a field of your
   own; the menu and the wizard ask for it and it is saved in the profile.
4. Describe it in `template.toml` (`description`, `formats`, `optional`,
   `[fields.X]`). Use `\ifdefempty{\Cmd}{}{...}` to omit empty lines; put
   `\\[0.25cm]` inside each conditional branch.
5. Check it: `investigacion --check-template <your-design>`.
6. Use it: `--design <your-design>`, in the menu's Design list, or
   `design = "<your-design>"` in a profile. A design can also be given by path
   (a `.ltx` file or a folder containing `template.ltx`); its key is then the
   parent folder name.

Extras:

- **Self-contained design:** omit `%%FORMAT%%` to carry all your own rules (for a
  school format that follows no known norm). `--check-template` warns, which is
  expected.
- **Logos for one design:** put a `logos/` folder next to `template.ltx` (git
  ignores `/templates/*/logos/`).
- **Extra packages:** any `.sty` next to the design is copied to the build
  folder.
- **Members on the cover:** use `\ListaIntegrantes` (already escaped, names
  joined with `\\`); put it in a `tabular` to align the lines, as the built-in
  covers do.
- **Where optional data comes empty:** list it in `optional` so the menu and
  the wizard do not require it.

### A format

1. Copy `templates/formats/harvard/` to `my-templates/formats/<name>/`.
2. Edit `format.sty` (it is copied as `investigacion-format.sty`; keep
   `\ProvidesPackage{investigacion-format}`): geometry, fonts, spacing,
   `titlesec`, `\captionsetup`, the floatrow caption position (guard it with
   `\@ifpackageloaded{floatrow}`), `fancyhdr`, and redefine
   `ReferenceList` and `\FormatBodySetup`.
3. Edit `format.toml` (`name`, `language`, `class_options`, `headings`).
4. Keep it **single-column**: Pandoc tables are `longtable`, which fails in
   `twocolumn` (that is why IEEE was removed).
5. Add the format to the `formats` lists of the designs that should offer it
   (an empty list already means all). `tests/pdf.rs` compiles every design x
   format pair, and `--check-template` reports per format.
6. Do not add `\makeatletter`; load packages with `\RequirePackage`; never load
   `float`.

### Logos

- **Names:** `logo-universidad.png` (top left in the built-in covers) and
  `logo-facultad.png` (top right).
- **Lookup** (`Project::resolve_logos_directory`), first match wins:
  1. `--logos <folder>` or `LOGOS` in `settings.toml`/environment (must exist,
     else "The logos folder does not exist: ...");
  2. `logos/` next to the design's `template.ltx`;
  3. `templates/logos/`.
- Having none is not an error: the cover compiles without logos.
- Use dark or colored PNGs with transparent background (a white logo is
  invisible on the white cover). The built-in designs scale by **height**, so a
  large transparent margin makes a logo look small.
- To move them, edit the `LOGOS` block of
  `templates/designs/geometric-cover/template.ltx` (`xshift`, `yshift`,
  `height`).
- `examples/` never depend on logos.

## 12. Writing the paper: Markdown syntax

The program accepts Pandoc Markdown plus three extensions
(`MARKDOWN_EXTENSIONS`): `mark` (`==highlight==`), `emoji` (`:rocket:`) and
`autolink_bare_uris`. `raw_tex` is **off** by default.

**Do not write** the cover, the table of contents, the title, your name, the
course or the date: the design adds them. Start with the first heading.

**The golden rule:** leave a blank line before and after every block
(heading, list, table, code, diagram). Most badly formatted PDFs come from a
missing blank line.

### Headings and structure

Markdown `#` is LaTeX `\section`, `##` `\subsection`, `###` `\subsubsection`,
`####` `\paragraph`, `#####` `\subparagraph`. How each looks depends on the
format ([section 8](#built-in-formats)).

| Markdown | APA 7 | Harvard | MLA 9 |
|---|---|---|---|
| `#` | centered, bold | `1` numbered, large, bold | bold |
| `##` | left, bold | `1.1` numbered, bold | italic |
| `###` | left, bold italic | `1.1.1` numbered, bold italic | bold italic |
| `####` | run-in paragraph heading, bold, ends with a period | LaTeX default | LaTeX default |
| `#####` | run-in, bold italic | LaTeX default | LaTeX default |

- **Structure warnings** come from `format.toml` `headings`. Only `apa7`
  defines them (`Introducción`, `Desarrollo`, `Conclusión`, `Referencias`). The
  program warns "The recommended heading "X" is missing." per missing one and
  "The recommended headings are not in the expected order: ..." when they exist
  but are out of order. Matching ignores case, accents, numbering (`2. `) and
  closing `#`s; headings inside fenced code are ignored. They are warnings,
  never errors.
- **References** are written by hand in the style of the format, as a dash list
  or as separate paragraphs, under a heading named `Referencias`,
  `Bibliografía`, `Referencias bibliográficas`, `Lista de referencias`,
  `Obras citadas`, `Fuentes consultadas`, `Fuentes`, `References`,
  `Reference list`, `Bibliography` or `Works Cited` (case and
  accent insensitive). The `blocks` filter wraps the section in `ReferenceList`
  (the format adds the hanging indent) until the next heading.

### Syntax summary

| Element | How to write it |
|---|---|
| Bold, italic, both | `**bold**`, `*italic*`, `***both***` |
| Strikethrough, highlight | `~~text~~`, `==text==` |
| Sub/superscript | `H~2~O`, `X^2^` |
| Inline code | `` `code` `` |
| Lists | `- item`, `1. item`, nested with two spaces (up to 4 levels; deeper ones are moved up with a warning); tasks `- [x] done`, `- [ ] pending` |
| Definition list | the term, then `: definition` on the next line |
| Quote | `> text` |
| Link | `[text](https://...)` or a bare URL |
| Footnote | `text[^1]` and `[^1]: the note` on another line |
| Table | `\| A \| B \|` with `\|---\|---\|` below (required); `:---:` centers, `---:` right-aligns |
| Table caption | a line `: Title` under the table (APA: becomes "Tabla 1" bold with the title in italic) |
| Code block | ` ```python ` ... ` ``` `; the language enables syntax colors |
| Formula | `$E = mc^2$` inline, `$$ ... $$` on its own line; `\sum`, `\frac`, `\begin{bmatrix}`, `\begin{cases}` work |
| Line break | `<br>` (the form that never gets lost) |
| Horizontal rule | `---` on its own line |
| Emoji | pasted (🚀) or by code (`:rocket:`); typeset as images through `twemojis` |
| Note box | `::: nota` ... `:::` (see below) |
| Diagram | ` ```{.dot caption="..."} ` |
| Chart | ` ```{.pgfplot caption="..."} ` or `.tikz` |

Elements that are written as in any Markdown guide (tables, footnotes,
lists, quotes, code, math) are covered by Pandoc itself. For a rendered
version of every element see [`../examples/syntax.md`](../examples/syntax.md) and
[`../examples/catalog.md`](../examples/catalog.md)
([PDF](../examples/catalog.pdf)).

### Tables

- Pipe tables become `longtable`, set at single spacing, and break across
  pages.
- The dashes row under the header is required; without it the table prints as
  rows of bars.
- Cells accept inline formatting. Footnotes inside tables work
  (`footnotehyper`).

### Code

- A fenced block with a language gets syntax highlighting on a light gray
  background (`framed`/`snugshade`, breaks across pages).
- A block without a language is `verbatim` (single spacing, small).
- **ASCII art** (`└──► ┌─┐ │`) must go inside a code block without a language;
  outside one it falls apart. `pmboxdraw` composes box-drawing characters at
  the exact monospaced width.

### Math

`$...$` and `$$...$$` work in both raw modes. Greek letters, relations and
operators written as Unicode characters work as-is (see the Unicode block of the
base).

Math goes to LaTeX unchanged, so, like a `pgfplot` block, a formula with a
LaTeX error (`$\frac{1}{$`, an unknown command) **stops** the build, and the
message shows the LaTeX error with its line. Checking LaTeX before compiling
is not worth its cost; pdflatex already does it.

### Footnotes

Standard Pandoc footnotes, numbered at the bottom of the page. They also work in
long tables.

### Images

```markdown
![Escudo de la universidad](logo.png){width=40%}
![Modelo OSI](https://ejemplo.com/osi.png)
```

- **Local images:** resolved against `--resource-path` (the Markdown's folder,
  `cache/`, `cache/remote/`, `cache/diagrams/`), or the path as written (absolute
  or relative to the Markdown's folder). A missing image becomes its alt text in
  italics (or "[imagen no disponible]") plus a warning "Image not found: ...".
- **Web images:** downloaded once into `cache/remote/<sha1 of URL>.<ext>` and the
  path is rewritten; later runs work offline. The extension comes from the URL
  or, if absent, from the `Content-Type` (`png`, `jpg`, `pdf`).
- **Formats:** `pdflatex` only handles PNG, JPG and PDF. Other formats (SVG,
  WebP) from a URL are rejected with a warning; convert them first.
- **Failures** (download, format, write) become the alt text plus a warning; the
  PDF is still produced.
- **Size:** `{width=40%}` or `{height=...}` like any Pandoc image. Images never
  exceed the text width or height (`\pandocbounded`).
- **Placement:** the figure stays where written (`H` with floatrow). Caption:
  APA puts "Figura 1" in bold above; Harvard and MLA put it below.
- A figure whose only content was a failed image is dissolved so no orphan
  caption appears.

### Diagrams (Graphviz)

````markdown
```{.dot caption="Árbol binario de búsqueda"}
digraph { 50 -> 30; 50 -> 70; 30 -> 20; 30 -> 40; }
```
````

- Classes: `dot` or `graphviz`. Attributes: `caption` (Markdown allowed),
  `width`, `height`.
- Drawn with `dot -Tpdf` (a vector graphic) and cached in
  `cache/diagrams/<sha1 of the code>.pdf`; an unchanged diagram is not
  recompiled.
- With a caption it becomes a numbered figure; without one it is centered.
- Without Graphviz, or with a syntax error in the diagram, the block stays as
  code and a single warning is printed.

### Charts (`pgfplot`, `tikz`)

````markdown
```{.pgfplot caption="Horas por fase"}
\begin{axis}[ybar, ymin=0, symbolic x coords={Análisis,Diseño,Código}, xtick=data]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180)};
\end{axis}
```
````

- Classes: `pgfplot`, `pgfplots`, `grafica`, `tikz`. The code is wrapped in
  `tikzpicture` unless it already contains one; with `caption` it becomes a
  figure, otherwise it is centered.
- Supported by the base and `examples/catalog.md`: vertical, horizontal,
  grouped and stacked bars, lines, scatter with regression, histograms, box
  plots, error bars, functions, log scales and pie charts (`\pie`). APA renders
  them in grayscale; do not set colors.
- **An intended hard failure (with broken math):** this is real LaTeX, so a syntax error
  **stops** the build, and the message shows the LaTeX error with its line.
  `last-error.tex/.log` are saved next to the PDF.

### Note boxes

```markdown
::: nota
Content of the box.
:::

::: {.aviso title="Antes de entregar"}
Custom title.
:::
```

Classes (Spanish and English accepted): `nota`/`note`, `aviso`/`warning`,
`importante`/`important`, `ejemplo`/`example`, `definicion`/`definición`/
`definition`. The default title follows the document language ("Nota"/"Note",
"Aviso"/"Warning", "Importante"/"Important", "Ejemplo"/"Example",
"Definición"/"Definition"); the `title` attribute overrides it. The result is
a `CajaMarcada` frame (thin black rule, single spacing). Without the filter a
fenced div would be dropped.

### Inline HTML

Handled by `inline_html.lua` (the LaTeX writer would silently drop it):

| Tag | Result |
|---|---|
| `<br>` | line break |
| `<em>`, `<i>` | italic |
| `<strong>`, `<b>` | bold |
| `<del>`, `<s>`, `<strike>` | strikethrough |
| `<sub>`, `<sup>` | subscript, superscript |
| `<u>`, `<ins>` | underline |
| `<mark>` | highlight |
| `<kbd>`, `<samp>`, `<code>` | inline code (text only) |
| `<img src alt title>` | image (same pipeline as `![](...)`) |
| `<wbr>` | removed |
| `<hr>` (own paragraph) | horizontal rule |

Tags nest and pair up through a stack; an unpaired tag is left as it was so the
text is not lost. **Block HTML is not typeset:** its tags disappear. In a check
with Pandoc 3.10.2 the text inside `<div>` and `<table>` survives as plain
paragraphs (a table loses its structure). Use Markdown for those.

### Raw LaTeX (`--allow-latex`)

- **Default (raw_tex off):** backslashes print as typed. A path such as
  `C:\Users\alumno` comes out literally instead of breaking the build.
  `$...$` math still works.
- **`--allow-latex`:** LaTeX commands in the Markdown are interpreted
  (`\newpage`, `\clearpage`...). A stray backslash can then break the build. It
  is a CLI-only switch (not in the menu).
- `pgfplot`/`tikz` blocks are real LaTeX in both modes.

### Symbols and unknown characters

Unicode symbols (`≠ ≤ ≥ ≈ ∈ ∑ ∫ √ ∞ α β Δ π ✓ ★`), box-drawing characters and
emoji work as-is. An unknown symbol prints a bold `[?]` and a warning names it
(character and code point). To support it add it to the `SÍMBOLOS UNICODE`
block of `templates/common/investigacion-base.sty`.

### Pasted text and encoding

Text pasted from Word, Notion, the web or an AI often contains invisible
non-breaking spaces. Without cleanup `###` appears in the PDF and tables
become rows of bars, with no error. See
[section 14](#14-reading-cleaning-and-finding-the-markdown). Files may be UTF-8
(with or without BOM) or Windows-1252.

## 13. Lua filters

Embedded from `resources/filters/` (`LUA_FILTERS`), applied in this order. They
operate on the parsed tree and never see the inside of code blocks (except the
filters whose job is code blocks).

| Order | File | Handles | Reads env |
|---|---|---|---|
| 1 | `inline_html.lua` | `Inlines`, `RawBlock`: HTML tags listed above | none |
| 2 | `images.lua` | `Image`, `Figure`: web download, local check | `INVESTIGACION_REMOTE_IMAGES`, `INVESTIGACION_RESOURCES`, `INVESTIGACION_LANG` |
| 3 | `diagrams.lua` | `CodeBlock` with class `dot`/`graphviz` | `INVESTIGACION_DIAGRAMS`, `INVESTIGACION_LANG` |
| 4 | `charts.lua` | `CodeBlock` with class `pgfplot`/`pgfplots`/`grafica`/`tikz` | none |
| 5 | `blocks.lua` | `Div` note boxes; lists deeper than LaTeX allows and the references section (`Pandoc`) | `INVESTIGACION_DOC_LANG`, `INVESTIGACION_LANG` |

Conventions:

- **Warnings** are written to stderr as `[investigacion] <message>`.
  `filter_warnings()` extracts them and they reach the same `on_warning`
  callback as the LaTeX warnings (so the log too). Other Pandoc stderr lines are
  not shown; they are logged at debug level.
- **Two languages, two variables.** Warnings follow the interface language
  (`INVESTIGACION_LANG`); text that goes into the PDF (box titles) follows the
  document language (`INVESTIGACION_DOC_LANG`). `pandoc_to_latex` sets both.
- **Never fail:** on a problem the image becomes its alt text, the diagram stays
  as code, and a warning is printed (each message once).
- **Cache names:** `sha1` of the URL (images) or of the diagram code.
- **`User-Agent`:** Pandoc sends none and some sites answer 400, so
  `--request-header` supplies one.
- **Adding a filter:** put the `.lua` in `resources/filters/`, add it to
  `LUA_FILTERS` (order matters), keep the warning conventions, and document the
  syntax here, in `docs/prompts/paper.md` and in `examples/catalog.md`.

To see what Pandoc produces for a snippet:

```bash
pandoc file.md --from=markdown-raw_tex+mark+emoji+autolink_bare_uris \
  --to=latex --wrap=none --lua-filter=resources/filters/inline_html.lua
```

## 14. Reading, cleaning and finding the Markdown

### Encoding (`src/encoding.rs`)

- `decode_text`: removes a UTF-8 BOM, tries UTF-8, then strict Windows-1252.
  Otherwise: "Could not read <file>: save it as UTF-8 or Windows-1252."
- Tool output (`decode_process_output`) is UTF-8 or lossy Windows-1252.
- The `pdflatex` log is decoded **line by line** (`decode_latex_log`): messages
  are UTF-8 but hyphenation lines are in T1; decoding the whole file as
  Windows-1252 would turn `└` into `â””`.
- Profiles, manifests and settings use `decode_text` too.

### Invisible spaces (`normalize_markdown`)

- **The bug it fixes:** text pasted from Word, Notion or an AI has non-breaking
  spaces on "empty" lines and trailing double spaces; Pandoc then merges
  everything into one paragraph (`###` printed, tables as rows of bars) with no
  error.
- **Disguised spaces:** U+00A0 (no-break), U+2007 (figure), U+202F (narrow
  no-break), U+200B (zero width).
- It trims disguised spaces and tabs at line ends and repairs markers (`#`
  to `######`, bullets `-*+`, numbered `1.`/`1)`, `>`) separated from the
  text by a disguised space.
- Trailing double spaces are both garbage and syntax (a hard line break).
  `has_disguised_spaces()` decides: if any line ends in a disguised space, normal
  trailing spaces are trimmed too; otherwise line breaks are preserved.
- It never touches hard spaces inside text or the inside of fenced ` ``` ` /
  `~~~` blocks. Line endings `\r\n` are normalized.

### Finding the Markdown (`find_markdown`, `resolve_markdown_path`)

In order:

1. the path as given (relative to the current directory, `~` expanded);
2. the path inside `input/`;
3. if only a bare name was given, a recursive search for that file name under
   `input/`.

With a profile that has `folder`, `locate_markdown` first searches
`input/<folder>` (only when the path as given does not exist), so two courses
can each have a `Tarea1.md`. An existing path always wins. Two files with the
same name are an error listing their subfolders ("There are several files
named X in input: ... Say which one with its subfolder."), never an arbitrary
pick. The result must exist, be a file and have the `.md` extension (case
insensitive); an empty file is "The Markdown file is empty."

## 15. The menu (TUI)

Run `investigacion` with no arguments. ratatui + crossterm; it duplicates no
logic: `App::build_args()` builds the same `cli::Args` and a thread named
`generation` runs `cli::execute()` with a channel-based `Reporter`.

### Modes and screens

| Mode | Kind | Notes |
|---|---|---|
| `ChooseLanguage` | window | First run only (no saved `IDIOMA`); the system language is preselected |
| `Home` | full screen | The menu opens here |
| `Form` | full screen | "Generate a PDF" |
| `Options` | window | Over home or the form |
| `Editing` | window | Text input over the form |
| `Picking` | window | Pickers (profile, Markdown, format, design, folder) |
| `Wizard` | window | Profile wizard |
| `Generating` | over the form | Keys are ignored while the PDF builds |

Windows remember which screen opened them (`App::base`) and return to it.
`app.rs` handles keys and state and never draws; `ui.rs` only draws.

Start (`App::start`): if no `IDIOMA` is saved, ask for the language (arrows or
Tab move the mark, Enter saves it in `settings.toml`, Esc uses it without
saving, so it asks again next time); then, if `courses/` has no profiles, the
wizard; then the home view. Windows' key-release events are ignored
(`KeyEventKind::Press` filter). **Ctrl+C quits from any mode**, even while
generating.

### Home view

- A short description of the program.
- Menu: **Generate a PDF** (form), **New profile** (wizard), **Options**,
  **Quit**.
- Status panel: your profiles (keys), the last PDF generated in this session,
  and the tool check: Pandoc and pdflatex (required, red if missing) and
  Graphviz (optional, yellow warning). The check runs in a background thread
  (`pandoc --version`, `pdflatex --version`, `dot -V`, 5 s limit; a timeout counts
  as installed). A missing required tool shows in red with a pointer to
  `INSTALL.md`. Messages (profile saved, profile errors, results) appear under
  the status.

Home keys: Up/Down/Tab/Shift+Tab move (wraps), Enter chooses, `s` or `q`
quits, `v` opens the last PDF (hidden).

### The form (Generate a PDF)

| # | Field | Kind | Required | If empty |
|---|---|---|---|---|
| 1 | Course profile | list | no | fill the fields by hand |
| 2 | Markdown file | list (folder browser, `.md` only) | **yes** | n/a |
| 3 | Title | text | **yes** | n/a |
| 4 | PDF file name | text | no | name of the Markdown (proposed automatically from the chosen file until you type one) |
| 5 | University | text | only if the design uses `UNIVERSIDAD` and does not list it optional | profile's; if none, omitted |
| 6 | Faculty | text | only if the design uses `FACULTAD` and does not list it optional | profile's; if none, omitted |
| 7 | Student | text | no | profile's; if none, omitted |
| 8 | Course | text | only if the design uses `MATERIA` and does not list it optional | n/a |
| 9 | Teacher | text | no | profile's; if none, omitted |
| 10 | Team members | text | no | profile's; if none, the student |
| 11 | Group | text | no | profile's; if none, omitted |
| 12 | Semester | text | only if the design uses `SEMESTRE` and does not list it optional | profile's; if none, omitted |
| 13 | Format | list | no | first one the design accepts |
| 14 | Design | list (filtered by the chosen format) | no | `geometric-cover` |
| 15 | Output folder | text (relative to the project root) | no | `output/` or the profile's |
| 16 | Extra copies | text, folders separated by `;` | no | no copies |
| 17+ | The design's own fields | text | per `template.toml` | n/a |

- A `*` after the label marks required fields; `▸` before the label marks
  fields chosen from a list. Missing required fields show `[missing]` in red.
- The design's own fields (for example *Salón*) appear at the end when the
  chosen design uses them; values are kept while the design still has them.
- University, faculty, student and semester reach `execute()` as `--set
  UNIVERSIDAD=…`, `FACULTAD`, `ALUMNO`, `SEMESTRE` (there are no dedicated CLI
  options for them), so the form works without a profile too.
- Choosing a profile fills university, faculty, student, semester, course,
  teacher, members, group, format, design, the
  design's own fields and the output folder; what you change afterwards wins
  (the same as a CLI option over a profile).
- Choosing a format removes the chosen design if it does not accept it.
- `--allow-latex`, `--settings`, `--logos`, `--doc-lang` and a design by path are
  CLI-only on purpose.
- Pressing `g` with missing required fields prints "Cannot generate yet.
  Missing: ..."; otherwise the result panel is cleared and fills with warnings
  and the final "PDF generated: ..." or the error. On success: "Done. Press v to
  view the PDF; the folders are in Options (o)."

Form keys:

| Key | Action |
|---|---|
| Up, `k`, Shift+Tab / Down, `j`, Tab | Move between fields (wraps) |
| Enter or Right | Edit the field, or open its list if marked `▸` |
| `1` to `9` | Jump to that field and activate it |
| Del or Backspace | Clear the field (clearing Design refreshes its fields) |
| `g` or F5 | Generate the PDF |
| `v` | Open the last PDF |
| `o` | Open the options |
| Esc | Back to the home view |
| `s` or `q` | Quit |
| option letters | Hidden shortcuts: `p`, `n`, `c`/`f`, `l`, `r`, `i`/`h` (the form's own keys win on a clash) |

Text editing (`Editing`): Enter saves (trimmed), Esc cancels, Ctrl+U clears,
Left/Right/Home/End/Backspace/Delete as usual.

### Options view

One table (`OPTIONS` in `src/tui/app.rs`) drives both the logic and the
drawing. Enter runs the highlighted row, or press its letter; Esc closes the
list. Both languages' letters work in either language.

| Key (es/en) | Action |
|---|---|
| `p` | Edit the chosen profile (wizard), or create one if none is chosen |
| `n` | New profile (empty wizard) |
| `c` / `f` | Open a project folder with the system's program: `input`, `output`, `courses`, `templates`, `cache`, or the project root `.` (a missing folder is created) |
| `l` | Switch Spanish/English; saved as `IDIOMA` in `settings.toml`. The row starts with the *other* language's name |
| `r` | Open the log file (a warning if it is off or does not exist) |
| `i` / `h` | Back to the home view |

### Pickers

One component (`src/tui/picker.rs`) serves the Markdown browser, profiles,
formats, designs and project folders.

- Typing filters the list. Ranking: exact name, then prefix, then substring,
  then match in the description; so `ia` picks the profile `ia`, not `example`
  ("mater**ia**").
- Markdown browser: starts in the chosen profile's `input/<folder>` (else
  `input/`, else the project root); shows `../`, folders, then `.md` files, each
  group alphabetical ignoring case, hidden entries (starting with `.`) excluded.
  Enter or Right opens a folder or chooses a file; Left, or Backspace with an
  empty filter, goes up and keeps the folder you left selected.
- Other keys: Up/Down, PageUp/PageDown (10 rows), Esc cancels.
- The design list shows only designs that accept the chosen format (and
  self-contained designs), with their `description`.

### Wizard

See [section 7](#the-profile-wizard-srctuiwizardrs).

### Robustness

- `event_loop` runs key handling and drawing inside `tui::guard::run`, and the
  worker thread runs `execute()` inside it too. A panic becomes an error line,
  "Internal error: <message>. It is in the log (Options -> r); you can keep using
  the menu.", and the window closes back to its base screen (a running
  generation is not cut). The backtrace goes to the log.
- Two failed draws in a row end the menu cleanly with a pointer to the log.
- `guard::install_hook()` runs **after** `ratatui::init()`. Inside `guard::run`
  the hook only logs; outside it the normal chain runs (ratatui restores the
  terminal, `logging` writes, Rust prints).
- Log: every key press and mode change at debug level; tool detection, opened
  paths, saved language and profile saves at info level; picker choices at debug
  level.

## 16. The command line

```bash
investigacion paper.md --title "Ecuaciones diferenciales" --course "Cálculo"
investigacion Tarea1.md -p ia --title "Búsqueda heurística"
investigacion paper.md --title "Tema" --format mla --design report --set SALON="B-204"
investigacion --check-template starter
```

Only the Markdown file and `--title` are always required by clap. Other data
is required only if the chosen design uses it ([section 9](#required-data-rules-missing_data-in-srcgeneraters)).
Option order does not matter. Values with spaces go in quotes; an unquoted stray
word is reported as "unexpected argument ... If the title or another value has
spaces, put it in quotes."

Compact option list (full detail, examples and edge cases in
[`cli.md`](cli.md)):

| Option | Hidden aliases | Value | Summary |
|---|---|---|---|
| `<MARKDOWN>` | | file | The `.md` file; a bare name is also searched in `input/` |
| `--title` | `--titulo` | text | Title (required) |
| `--file-name` | `--nombre` | name | PDF name, with or without `.pdf` |
| `--course` | `--materia`, `--subject` | text | Course name (overrides profile `name`) |
| `-p`, `--profile` | `--perfil`, `--subject-profile` | key | Profile from `courses/` |
| `--teacher` | `--docente` | text | Teacher |
| `--members` | `--integrantes` | `"A, B"` | Team members |
| `--group` | `--grupo` | text | Group |
| `--output` | `--salida` | folder | Folder for the PDF |
| `--copy` | `--copia` | folder | Extra copy folder, repeatable |
| `--settings` | `--env-file` | file | Settings file (default `settings.toml`) |
| `--allow-latex` | `--permitir-latex` | flag | Interpret raw LaTeX in the Markdown |
| `--design` | `--template`, `--plantilla`, `--diseno` | name or `.ltx` path | Cover and look |
| `--format` | `--formato` | name | The norm |
| `--set` | none | `NAME=value` | Design's own field, or a standard datum; repeatable; name uppercased |
| `--doc-lang` | `--idioma-documento` | `es`/`en` | Document language |
| `--logos` | none | folder | Logos folder (falls back to `LOGOS`) |
| `--lang` | `--idioma` | `es`/`en` | Interface language for this run |
| `--check-template` | `--revisar-plantilla` | design | Check a design and build the catalog with it |
| `-h`, `--help`, `-V`, `--version` | | | Localized help and version |

Behavior details:

- **Exit codes:** `0` success (also `--help`/`--version`); `1` generation error
  (printed as `Error: <message>` on stderr) or a failed check; `2` argument
  errors reported by clap and `--check-template` without a name.
- **Output:** warnings go to stderr as `Warning: ...` (`Aviso: ...` in Spanish);
  results go to stdout: `PDF generated: <path>` and `Copy saved: <path>`.
- **Localized help and errors:** `prescan_language()` reads `--lang` and
  `--settings` before parsing so `--help` and argument errors are already in the
  right language. `ARG_HELP` in `src/cli.rs` holds the bilingual help; custom
  `-h`/`-V` are defined so they can be translated. `describe_clap_error()`
  translates missing required arguments, unknown arguments, and invalid values.
- **`--set` accepted names:** custom fields of the design, plus the standard
  `UNIVERSIDAD`, `FACULTAD`, `ALUMNO`, `SEMESTRE`, `DOCENTE`, `INTEGRANTES`,
  `GRUPO`. `NAME=` (empty value) is valid and blanks the value.
- **Timing:** `INVESTIGACION_TIMING=1 investigacion ...` prints the time of each
  Pandoc/pdflatex run.
- **Hidden old Spanish names** (`--titulo`, `--materia`, `--plantilla`,
  `--docente`, ...) stay valid; never remove them.

## 17. Compilation

`compile_pdf` (`src/compile.rs`):

- **Command:** `pdflatex -interaction=nonstopmode -halt-on-error
  -file-line-error [-draftmode] trabajo.tex`, run inside the temp folder (only the
  file name goes on the command line: a full path breaks on spaces and on the `~`
  of Windows short paths such as `C:\Users\USUARI~1\...`). On MiKTeX it also
  gets `--enable-installer`. `TEXINPUTS` is `.`, then
  `pandoc::resource_dirs()` (the Markdown's folder, `cache/`, `cache/remote/`,
  `cache/diagrams/`, where images and diagrams are found by name), then any
  existing `TEXINPUTS`, then an empty entry (meaning "plus the default paths").
  Built with `std::env::join_paths`.
- **Passes:** up to `MAX_LATEX_RUNS = 4`. The first pass uses `-draftmode`
  (writes `.aux` and `.toc` but not the PDF) when there is no previous state. A
  new pass is needed when the log says "Rerun to get" or "Rerun LaTeX" or the
  `.toc` changed. "stable after pass N" is logged; if the table of contents
  still changes after the last pass a warning is logged.
- **State cache:** `.aux` and `.toc` are saved after a successful build in
  `cache/latex/<pdf-stem>-<16 hex>/trabajo.{aux,toc}`. The hash covers the output
  PDF path and `Layout::cache_key()` (design file path + format key), because
  another design loads other packages. With a cache hit a re-run usually takes a
  single (non-draft) pass. Designs without a table of contents write no `.toc`;
  an empty one is saved so the cache works for them too. If the build with the
  cached state fails, the state files are deleted and the build is retried from
  scratch (only a failure of that second attempt is a real error). Saving the
  cache is best-effort.
- **Timeout:** `TOOL_TIMEOUT = 180 s` per Pandoc or pdflatex run
  (`src/process.rs`; process killed). Message: "pdflatex kept working for more
  than 180 seconds and was stopped; look in the document for something LaTeX
  cannot typeset." (soul loops are the classic cause). The tool check in the menu
  uses a 5 s limit.
- **Failure:** "The LaTeX build failed:" followed by the real errors
  (`summarize_latex_errors`: lines `file:line:` and `!`, up to three blocks;
  package paths are dropped; if none are found, the last ~2000 characters of the
  log). `last-error.tex` and `last-error.log` are saved **next to the PDF** (in
  the output folder) and the message ends with "Full log: <path>".
- **Warnings from the log:** `unsupported_character_warnings` turns each
  "Caracter Unicode sin definir" entry into one warning per distinct symbol.
- **Result:** the PDF is copied from the temp folder to the output path; a copy
  failure is a write error mentioning permissions or an open PDF viewer.
- **Output location of the temp folder** is the system temp directory
  (`investigacion-<random>`), removed at the end.

## 18. Logging

`src/logging.rs` writes `cache/logs/investigacion.log` for both the menu and
the command line. It is a diagnostic aid and must never break the program.

- **What is recorded:** session header (version, OS/arch, interface language,
  project root, working directory, arguments, level), the options received, the
  design/format/document language chosen, each Pandoc and pdflatex run with its
  exit status and time, LaTeX-state cache hits, every warning and error the user
  saw (`LoggingReporter` copies them), `last-error.*` locations, menu events and
  panics with a backtrace.
- **Format:** `YYYY-MM-DD HH:MM:SS.mmm [LEVEL] [thread] message`; extra message
  lines are indented under a `|`. A new session starts after a blank line.
  Log lines are technical English.
- **Levels:** `debug` (full commands with cwd and environment, folders, Pandoc's
  own stderr, every key pressed in the menu, timings), `info` (steps), `warn`
  (recovered problems), `error` (failures). Call
  `logging::{debug,info,warn,error}(format_args!(...))` at every step that can
  fail or that changes the PDF.
- **`INVESTIGACION_LOG`:** `debug` lowers the threshold; `off` or `0` disables
  logging; anything else or unset means `info`.
- **Rotation:** when the log is larger than 1 MB (`MAX_LOG_BYTES`) at `init`, it
  is renamed to `investigacion.old.log`, replacing the previous one.
- **Lifecycle:** a no-op until `logging::init(&project)` runs (only `main.rs`
  calls it, so tests leave no files). `init` does nothing if the root has no
  templates, so no stray `cache/` is created elsewhere. Writes are unbuffered
  and flushed per entry. The logger never returns errors and never panics; if a
  write fails it turns itself off. The state lock tolerates poisoning.
- **Panic hook:** `logging::install_panic_hook()` (before the TUI starts) logs
  every panic at error level with a backtrace and then calls the previous hook.
- **I/O errors:** `std::io::Error -> GenerationError` is `#[track_caller]`: it
  logs the `?` location and, when the log is active, appends "Details in the
  log: <path>" to the message.
- **Privacy:** the log lives in the git-ignored `cache/` and contains paths,
  titles and the messages shown. Review it before sharing.
- **In the menu:** Options -> `r` opens it.

## 19. Errors, reporting and robustness

- **`GenerationError`** (`src/error.rs`): a `String` message ready to show. Every
  expected failure (missing file, tool, bad profile, broken LaTeX) is one. The
  CLI prints `Error: <message>` and exits 1; the menu shows it in the result
  panel.
- **The library never prints.** It reports through the `Reporter` trait
  (`warning`, `info`): `ConsoleReporter` in the CLI (stderr/stdout) and a
  channel reporter in the TUI. `execute()` wraps the reporter in
  `LoggingReporter`, so everything the user sees is also logged.
- **Always produce a PDF** ([section 1](#1-overview)): warnings, not errors, for
  images, symbols, Graphviz and heading structure.
- **Hard failures:** missing Pandoc or pdflatex, timeouts, broken `pgfplot`/`tikz`
  or math LaTeX, missing required data, a missing or non-`.md` or empty Markdown, unknown
  design/format/profile, an incompatible design/format pair, an invalid
  profile or manifest, an unwritable output folder, a bad `--logos` folder.
- **TUI:** panics are caught (see [section 15](#robustness)).

## 20. Internationalization

- **Interface language order** (`i18n::resolve`): `--lang es|en` (alias
  `--idioma`) > `IDIOMA`/`INTERFACE_LANGUAGE` (environment, then
  `settings.toml`) > the OS language (`sys-locale`: `es*` is Spanish, anything
  else English). `i18n::parse` accepts `es`, `español`, `espanol`, `spanish`,
  `en`, `inglés`, `ingles`, `english` (case-insensitive).
- **Per-thread:** the language is a `thread_local!`; tests set their own with
  `i18n::set`. The menu copies it into the worker thread before `execute()`; any
  new thread that produces messages must do the same.
- **Messages with arguments:** `tr!(es: "...{x}...", en: "...{x}...", args)`. The
  macro requires both versions, so a message cannot be added in one language.
- **Constant texts** (labels, help): `Text::new(es, en)` plus `.get()` (it also
  implements `Display`).
- **Never** pass clap help templates through `format!`/`tr!` (their
  `{usage}`/`{options}` would be consumed); use `Text`.
- **Rules:** every user-facing string goes through `tr!` or `Text` (messages,
  warnings, help, TUI); Spanish texts have accents; messages describe the
  action from the user's point of view and avoid internal names; log lines are
  English and are not translated.
- **Manifests:** `Localized` (`"Plain"` or `{ es, en }`) in `format.toml` and
  `template.toml`.
- **Lua filters:** receive the interface language in `INVESTIGACION_LANG` and the
  document language in `INVESTIGACION_DOC_LANG`.
- **Not translated:** error text from the `toml` crate (English), clap messages
  of kinds other than those handled in `describe_clap_error`, and the fixed
  labels of a design's cover.

## 21. Cross-platform rules

The program runs on Linux and Windows, and the code must stay portable so the
macOS update needs no redesign:

- No `cfg!(windows)`, no fixed paths, no hand-written path separators.
- Build search paths with `std::env::join_paths` / `split_paths` (the
  `--resource-path` and `TEXINPUTS` values).
- Use `project::absolute()` (wraps `std::path::absolute`), never
  `canonicalize()`: on Windows it returns `\\?\C:\...` paths that Pandoc and
  `pdflatex` do not understand.
- Use `pandoc::posix()` for any path that ends up inside LaTeX or a filter
  (always `/`).
- `~` is expanded from `HOME` or `USERPROFILE`.
- The OS language comes from `sys-locale` (Windows has no `LANG`).
- Windows sends key-release events: keep the `KeyEventKind::Press` filter.
- A PDF open in a viewer can lock the output file on Windows: write errors say
  to close it. A temp folder that cannot be removed immediately is ignored.
- No full path goes inside LaTeX. The `.tex` is passed by name with the temp
  folder as working directory, and diagrams and downloaded images by file name,
  found through `TEXINPUTS`. A full path breaks on spaces, on the `~` of Windows
  short paths (`C:\Users\USUARI~1\...`) and on accents (on Windows, `os.getenv`
  in the Lua filters does not return UTF-8).
- MiKTeX is detected once per run (`pdflatex --version`) and gets
  `--enable-installer`, so a fresh install downloads missing packages instead
  of waiting on a dialog. The first build can be slow for that reason.

## 22. Source map and how to extend

### Modules

| Path | Responsibility |
|---|---|
| `src/main.rs` | Entry point: initializes the log and panic hook; no arguments -> TUI, else CLI |
| `src/lib.rs` | Module list; `tr!` macro available everywhere (`#[macro_use] i18n` first) |
| `src/cli.rs` | `Args` (clap), localized help/errors, `Reporter`, `execute()`, `--check-template` dispatch |
| `src/i18n.rs` | `Lang`, `tr!`, `Text`, detection and resolution |
| `src/logging.rs` | Diagnostic log, levels, rotation, panic hook |
| `src/error.rs` | `GenerationError`, `Result` |
| `src/template.rs` | `Format`, `Design`, manifests, `STANDARD_MARKERS`, `resolve_layout`, `Layout` |
| `src/check.rs` | `--check-template` |
| `src/generate.rs` | `generate_pdf`, `missing_data`, `output_file_name`, `copy_pdf_to`, state cache directory |
| `src/markdown.rs` | Read/normalize Markdown, lookup in `input/`, heading validation, `expand_home`, `strip_accents` |
| `src/pandoc.rs` | Pandoc call, embedded Lua filters, `posix()`, `filter_warnings` |
| `src/latex.rs` | `latex_escape`, `render_template`, log summaries and Unicode warnings |
| `src/compile.rs` | `copy_template_assets`, `latex_search_path`, `compile_pdf`, passes, state cache, failure artifacts |
| `src/project.rs` | Project root, folders, logos lookup, legacy design names, defaults |
| `src/settings.rs` | `settings.toml` (KEY=VALUE) + environment; `save_value` |
| `src/courses.rs` | Course profiles: `CourseProfile`, `find_course`, `list_courses`, `save_profile` |
| `src/document.rs` | `DocumentData`, `parse_members`, dates, `slugify` |
| `src/encoding.rs` | UTF-8 with Windows-1252 fallback, log decoding |
| `src/process.rs` | Run tools with a timeout (`TOOL_TIMEOUT` 180 s), timing output |
| `src/tui/` | `app.rs` (state and keys), `ui.rs` (drawing), `wizard.rs`, `picker.rs`, `guard.rs`, `tools.rs`, `tests.rs` |
| `resources/filters/` | `inline_html`, `images`, `diagrams`, `charts`, `blocks` Lua filters |
| `templates/` | `common/`, `formats/{apa7,harvard,mla}/`, `designs/{geometric-cover,classic-cover,report,starter}/`, `logos/` |
| `tests/` | `templates.rs`, `pandoc.rs`, `pdf.rs`, `cli.rs`, `logging.rs`, `common/` |

### Checklists

**Add a CLI option**

1. Add a field to `Args` (English name; `alias` if a natural Spanish one exists;
   never remove old aliases).
2. Add an `ARG_HELP` entry with `es` and `en` help and a value name
   (`mut_arg` panics on an unknown id; the help tests catch it).
3. Use it in `execute()`; if it should fall back to a profile, go through
   `pick()`.
4. If the menu needs it, add a `FIELDS` entry (`label`, `help`, `example`,
   `empty`, all `Text`) and map it in `App::build_args()`.
5. Add tests (`src/cli.rs` tests and `tests/cli.rs`).
6. Update `cli.md`, this page, `CHANGELOG.md` and, if it applies, `README.md`.

**Add a profile key:** add it to `CourseProfile`, `save_profile`, `pick()` in
`execute()`, the wizard if it is cover data, and `App::apply_profile()`; document
it in `courses/README.md`, `courses/example.toml` and this page.

**Add a standard marker:** change `DocumentData`, `render_template`,
`STANDARD_MARKERS`, `missing_data` (if it can be required), the wizard's
`STANDARD_STEPS` (and `REQUIRED_STANDARD`), the designs and `--set` handling in
`run_generation`.

**Add a menu option row:** append an entry to `OPTIONS` and handle its action in
`App::run_option()`.

**Add Markdown syntax:** decide among `MARKDOWN_EXTENSIONS`, a Lua filter, or the
base `.sty`; then update `docs/prompts/paper.md`, this page and `examples/catalog.md`.

**Add a format or design:** [section 11](#11-making-your-own-design-or-format);
for built-ins also add them to the `formats` lists, the README, this page and the
catalog check.

### Rules for contributors (`AGENTS.md`)

- **Code in English, comments in Spanish** (short; one per block).
- **Bilingual interface:** every user-facing string through `tr!`/`Text`.
- **No logic in the TUI:** it builds `cli::Args` and calls `cli::execute()`.
- **Formats and designs are separate layers;** a design follows the contract.
- **APA output must not change by accident** ([section 23](#23-testing-and-regression-rules)).
- **Keep old names working:** hidden clap aliases, serde aliases `subject` and
  `template`, legacy designs `apa`/`apa-simple`.
- **Log what happens, never break because of it.**
- **Before finishing:** `cargo fmt`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test`; for template or filter changes also generate
  `examples/catalog.md` and check the pages.
- **Update the docs** when behavior changes: `README.md`, `INSTALL.md`, this
  page and `cli.md`, `docs/prompts/paper.md` (whenever accepted Markdown syntax
  changes), `CHANGELOG.md`.

## 23. Testing and regression rules

```bash
cargo build --release                     # target/release/investigacion
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test                                # PDF/Pandoc tests skip if the tools are missing
cargo test --lib markdown                 # one module
cargo test --test pdf                     # real PDFs (~45 s)
target/release/investigacion --check-template starter
INVESTIGACION_TIMING=1 target/release/investigacion examples/catalog.md --title T -p example
INVESTIGACION_LOG=debug target/release/investigacion ...
```

- Unit tests live next to the code; the TUI has `src/tui/tests.rs` (`press()`,
  `type_text()`, `render()` on a 120x32 `TestBackend`).
- `tests/pdf.rs` compiles every design x format pair (also without optional
  fields), an English document, a minimal design with a custom field, and the
  starter through the check; `tests/pandoc.rs`, `tests/templates.rs`,
  `tests/cli.rs` and `tests/logging.rs` cover the rest.
- **APA must not change by accident.** For any change to the base, the `apa7`
  format or the APA designs, render the catalog with both covers before and
  after (`pdftoppm -r 40`) and compare every page with `cmp`; they must stay
  byte-identical unless the change is intended.
- After template or filter changes, regenerate the versioned catalog and
  screenshots with the `/regenerate-catalog` skill (placeholders from
  `courses/example.toml`, empty logos folder).
- To check a Pandoc contract change: compare
  `pandoc f.md -s --to=latex | sed -n '/documentclass/,/begin{document}/p'` with the base.

## 24. Troubleshooting

Only the real LaTeX error is shown. After a failed build, `last-error.tex` and
`last-error.log` are next to the PDF. For anything unclear read the end of
`cache/logs/investigacion.log` (Options -> `r` in the menu); run with
`INVESTIGACION_LOG=debug` for the full commands.

| Symptom or message | Likely cause and fix |
|---|---|
| "Missing data used by the design X: ..." | The design uses that data and it is empty. Fill it in the profile (`p` in the menu options), or give it with `--set NAME=value` (or `--course`) |
| "The design X does not work with the format Y" | Choose a format the design lists, or another design |
| "Design X not found" / "Format X not found" | Typo, or the folder lacks `template.ltx`/`format.sty`. The message lists the available names. A same-named folder in `my-templates/` never overrides a built-in |
| "Course profile not found: X" | Wrong key; `courses/<key>.toml` must exist. The message lists the profiles |
| "The course profile ... is not valid" | A typo in a key (unknown keys are errors) or bad TOML; the file path and the parser's message are shown |
| "The course profile ... needs a name" | Add `name = "..."` |
| "The Markdown file does not exist" | Wrong path; a bare name is searched in `input/` and its subfolders |
| "There are several files named X ..." | Include the subfolder: `IA/Tarea1.md` |
| "The input file must have the .md extension." | Rename the file |
| "The Markdown file is empty." | The file has no content |
| "Could not read <file>: save it as UTF-8 or Windows-1252." | Re-save the file in a supported encoding |
| "Pandoc was not found" / "pdflatex was not found" | Install the tool and make sure it is on the `PATH`; reopen the terminal ([`../INSTALL.md`](../INSTALL.md)) |
| "Pandoc did not answer within 180 seconds" / "pdflatex kept working for more than 180 seconds" | A hang: look for something LaTeX cannot typeset; check `last-error.log` |
| "The LaTeX build failed" pointing to a line of your text | A stray backslash with `--allow-latex` |
| ... pointing to a chart | `pgfplot` syntax error (real LaTeX); the message gives the line; fix or delete the block |
| LaTeX reports a missing `.sty` | Install the package (`tlmgr install <name>` or the MiKTeX Console) |
| "Could not write to <folder>" | Permissions, or the PDF is open in a viewer; close it |
| "The logos folder does not exist" | Fix `--logos`/`LOGOS`, or remove it to use `templates/logos/` |
| `###` printed in the PDF | Invisible spaces or a missing blank line before the heading |
| A table shows as rows of bars | The dashes row under the header is missing, or a blank line before the table |
| A diagram is misaligned | ASCII art outside a code block |
| Diagram appears as code | Graphviz missing (`dot -V`) or a syntax error in the diagram (one warning) |
| "Image not found: ..." | The path is resolved from the Markdown's folder (and `cache/`) |
| Web image missing | Not PNG/JPG/PDF (SVG, WebP), no connection, or the site refused; convert and use a local file |
| `[?]` in the PDF and a warning naming a symbol | The font has no glyph; replace it or declare it in the Unicode block of `investigacion-base.sty` |
| Warnings about missing headings | APA 7 recommends `Introducción`, `Desarrollo`, `Conclusión`, `Referencias`; it is only a warning |
| Wrong language on the cover or "Contents" | Set the document language: `language` in the profile or `--doc-lang` |
| Cover logos missing | Files must be named `logo-universidad.png` and `logo-facultad.png` and be in `templates/logos/`, next to the design, or in `--logos` |
| Your own design fails | `investigacion --check-template <design>` |
| "Invalid syntax in settings.toml, line N: expected KEY=VALUE" | `settings.toml` is KEY=VALUE lines, not TOML tables; fix or delete the file |
| Menu shows "Internal error: ..." | A bug; the menu keeps running; send the log |
| The menu says a required tool is missing | Install Pandoc/pdflatex ([`../INSTALL.md`](../INSTALL.md)) |
| `investigacion: command not found` | Open a new terminal or `source ~/.cargo/env` |
| The program cannot find its templates after moving the folder | Run `cargo install --path .` again or set `INVESTIGACION_HOME` |
| Log empty or missing | `INVESTIGACION_LOG=off`, or the project root has no templates, or the folder is read-only |

## 25. Glossary

| Term | Meaning |
|---|---|
| Base | The universal LaTeX layer (`templates/common/`): Pandoc compatibility, diagrams, charts, Unicode |
| Format | The norm (`apa7`, `harvard`, `mla`): `format.sty` + `format.toml` |
| Design | The cover or title block and the look: `template.ltx` + `template.toml` |
| Layout | A design plus its format (`Layout` in `src/template.rs`); the format is absent for a self-contained design |
| Self-contained design | A design without `%%FORMAT%%`; carries all its own rules |
| Contract | What a design must contain (`investigacion-base`, `%%FORMAT%%`, `investigacion-final`, `%%CONTENIDO_MARKDOWN%%`) |
| Marker | A `%%NAME%%` placeholder in `template.ltx` |
| Standard marker | A marker the program fills (`STANDARD_MARKERS`) |
| Custom field | Any other marker; filled from `[fields]`, `--set` or the menu |
| Profile / course | `courses/<key>.toml`; the data of one course (`CourseProfile`, `Course`) |
| Wizard | The step-by-step profile editor in the menu |
| Fragment | Pandoc's LaTeX output without a preamble |
| Document language | The language of the PDF (`es`/`en`) |
| Interface language | The language of menus, messages and help |
| Legacy design | `apa` and `apa-simple`, mapped to `geometric-cover`/`classic-cover` |
| Disguised space | An invisible space (U+00A0, U+2007, U+202F, U+200B) |
| `ReferenceList` | LaTeX environment wrapping the references section (hanging indent per format) |
| `CajaMarcada` | LaTeX environment of a note box |
| `\FormatBodySetup` | Hook a format redefines to set alignment and indent after the cover |
| Draft mode | `pdflatex -draftmode`: first pass without producing the PDF |
| State cache | Saved `.aux`/`.toc` in `cache/latex/` that lets a re-run finish in one pass |
| `TOOL_TIMEOUT` | 180 s limit for each Pandoc and pdflatex run |
| Reporter | Trait through which the library reports warnings and info |
| `GenerationError` | An expected failure with a message ready to show |
| Picker | The list component of the menu |
| Extras | A design's custom fields appended to the menu form |
| Catalog | `examples/catalog.md`/`.pdf`: the visual test bench with every element |
