---
name: latex-templates
description: The three template layers (universal base, format, design), the design contract and its markers, custom fields, manifests (format.toml, template.toml), known LaTeX traps (soul, floatrow, babel, hyperref), Unicode/emoji, logos/TEXINPUTS and the pdflatex pass cache. Use when editing templates/**, src/template.rs, src/latex.rs, src/compile.rs or src/check.rs, adding a design or a cover field, or debugging a LaTeX build failure.
paths: "templates/**,src/template.rs,src/latex.rs,src/compile.rs,src/check.rs,src/document.rs,src/generate.rs,tests/templates.rs,tests/pdf.rs"
---

# LaTeX templates: base, format and design

## The three layers

```
DESIGN  templates/designs/<d>/template.ltx + template.toml   cover, look, custom fields
FORMAT  templates/formats/apa7/format.sty + format.toml      the norm: fonts, spacing, headings, captions, references
BASE    templates/common/investigacion-base.sty / -final.sty  universal: Pandoc compatibility, diagrams, charts, Unicode
```

- **Built-ins:** the format `apa7` (the only one; `load_format`) and the
  design `geometric-cover`.
- **The user's own designs** go in `my-templates/designs/`, searched after the
  built-ins (`template.rs`: `roots()`). There are no user formats.
- **Selection.** `resolve_layout(project, design)` loads the design and, unless
  it is self-contained (no `%%FORMAT%%`), the APA 7 format.
- **Retired names.** `RETIRED_DESIGNS` (`apa`, `apa-simple`, `classic-cover`,
  `report`, `starter`) lead to `geometric-cover`, and
  `retired_design_warning` gives the one-line warning; a user design with one
  of those names wins, without a warning. Any `--format`/profile `format`
  other than `apa7` is ignored with a warning in `run_generation`.

## The design contract

```latex
\documentclass[%%CLASS_OPTIONS%%]{article}  % format.toml class_options (size, paper)
\usepackage{investigacion-base}              % required, first
%%FORMAT%%                                   % program inserts \usepackage[<babel>]{babel} + \usepackage{investigacion-format}
\newcommand{\TituloTrabajo}{%%TITULO%%} ...  % data (optional; final.sty provides empty defaults)
\usepackage{investigacion-final}             % required, last in the preamble
\begin{document} ...cover... \FormatBodySetup ... %%CONTENIDO_MARKDOWN%% \end{document}
```

- **Files the program copies.** `copy_template_assets()` copies `common/*.sty`,
  any `.sty` next to the design, `format.sty` (renamed to
  `investigacion-format.sty`) and the other files of the logos folder into the
  temp dir, which is first in `TEXINPUTS`; `generate::copy_logos` adds the two
  cover logos.
- **Babel follows the document language** (`Layout::format_block`): es →
  `spanish, es-tabla`, en → `english`. Anything Spanish-only in a format must
  be guarded, e.g. `\@ifundefined{captionsspanish}{}{\addto\captionsspanish{…}}`.
- **Hooks the base defines and the format redefines:**
  - `\FormatBodySetup`: alignment and indent after the cover;
  - `ReferenceList`: neutral in the base; hanging indent in apa7.
- **Inside a `.sty`:** no `\makeatletter`; load packages with
  `\RequirePackage`.
- **Comment lines are not scanned for markers** (`extract_markers`), so a
  comment can mention `%%X%%`, and `%%CONTENIDO_MARKDOWN%%` in a comment is
  not filled (`latex::is_comment_line`). Never write `%%FORMAT%%` inside a
  comment anyway, because it is replaced by a two-line block.
- **Cover labels:** `\SegunIdioma{<es>}{<en>}` (base) follows the document
  language; `geometric-cover` uses it. Spanish output must stay identical.
- **babel-spanish `\%` in math:** `investigacion-final.sty` wraps
  `\es@sppercent` so `$28.9\,\%$` compiles; text mode keeps the original.

## Markers and data

- **Standard markers** (`STANDARD_MARKERS`): `TITULO UNIVERSIDAD FACULTAD
  ALUMNO INTEGRANTES MATERIA DOCENTE GRUPO FECHA_ENTREGA
  CONTENIDO_MARKDOWN FORMAT CLASS_OPTIONS`. `SEMESTRE` is no longer one: in an
  old design it is a custom field and `--check-template` suggests `GRUPO`.
- **Custom fields.** Any other `%%NAME%%` is a custom field: filled from
  `DocumentData.fields` (the profile's `[fields]`, `--set`, the TUI extras),
  escaped, empty if missing.
- **Required data** (`generate::missing_data`):
  - standard: `TITULO`, `UNIVERSIDAD`, `FACULTAD`, `MATERIA`
    (`REQUIRED_MARKERS`), only if the design uses them and does not list them in `optional`;
  - custom: only if `[fields.X] required = true`.
- **Escaping.** `render_template` escapes user data with `latex_escape`.
  Pandoc's content is not escaped and goes in **last**.
- **`%%INTEGRANTES%%`:** each name is escaped separately, then joined with a
  literal `\\`.
- **`geometric-cover`:**
  - order: student or team, course, teacher, group;
  - members replace the student (`\ifdefempty{\ListaIntegrantes}`);
  - `\\[0.25cm]` goes inside each conditional branch.
- **Adding a standard marker** means changing `DocumentData`,
  `render_template`, `STANDARD_MARKERS`, `missing_data`, the wizard's
  `STANDARD_STEPS` and the designs.

## Manifests

```toml
# format.toml (required; unknown keys are ignored)
name = { es = "APA 7", en = "APA 7" }  # optional and ignored; kept for older binaries
language = "es"                       # document language default
class_options = "12pt, letterpaper"
headings = ["Introducción", "Desarrollo", "Conclusión", "Referencias"]  # [] = no structure check

# template.toml (optional)
description = { es = "…", en = "…" }
optional = ["UNIVERSIDAD"]            # standard markers shown but not required
[fields.SALON]
label = { es = "Salón", en = "Room" }
help = { es = "…", en = "…" }
required = true
```

`template.toml` uses `deny_unknown_fields`; an old `formats` list is still
accepted (and ignored, with a `--check-template` warning). `format.toml` has
no `deny_unknown_fields`, so `name` and extra keys are ignored. Bilingual
values are `Localized` (plain or `{es, en}`).

The format must stay **single-column**: Pandoc tables are `longtable`, which
fails in `twocolumn`.

## The Pandoc contract (base)

The `PAQUETES QUE NECESITA LA SALIDA DE PANDOC` block covers:

- tables: `calc`, `\newcounter{none}`, the `longtable` patch;
- images: `\pandocbounded`;
- code: `Shaded`/`\…Tok`;
- strikethrough: soul/ulem;
- math: `amsmath`;
- footnotes in tables: `footnotehyper`.

**Remove nothing.** Compare with
`pandoc f.md -s --to=latex | sed -n '/documentclass/,/begin{document}/p'`.

Traps:

- **`verbatim`** gets `\singlespacing\small`.
- **floatrow and `float` conflict.** Never load `float`. The `H` placement line
  (`\@ifundefined{floatsetup}…\fps@figure`) must stay where Pandoc sets its
  own.
- **Long tables:** `\LTleft`/`\LTright` are set to `\fill`.
- **Babel captions:** wrap renames in `\addto\captions<lang>`.
- **Package order:** `hyperref`, then `footnotehyper`, at the end (in
  `investigacion-final.sty`, which also `\providecommand`s `\TituloTrabajo`,
  `\ListaIntegrantes`, `\NombreAlumno`).

### soul: \st, \hl, \ul

soul loops forever inside `longtable` and breaks in headings. The fix:

- `\st` and `\ul` are redefined on ulem with `\DeclareRobustCommand`;
- `\hl` is swapped for a `\colorbox` inside `longtable`;
- covered by `the_extended_syntax_compiles_without_lost_symbols`;
- safety net: `TOOL_TIMEOUT`.

### TikZ and babel

The base loads `\usetikzlibrary{babel}`: Spanish babel makes `<` and `>`
active, and without it `->` in a ```tikz``` block or a design fails with
"Argument of \language@active@arg> has an extra }".

### pgfplots

The base only loads pgfplots. Styles live in the format (apa7: grayscale via
`every axis/.append style` and `bar cycle list`). Never put `fill` in
`cycle list`.

## Unicode and emoji (base)

- **Declared symbols.** `SÍMBOLOS UNICODE` lists only the symbols that really
  failed.
- **Undefined symbols.** `\UTFviii@undefined@err` warns, prints `[?]` and tries
  `\texttwemoji`. Its text "Caracter Unicode sin definir" is matched by
  `unsupported_character_warnings`; keep it identical.
- **Reading the log:** use `decode_latex_log()` (line by line).

## Logos

- **Names:** `logo-universidad` and `logo-facultad`, found as svg, pdf, png,
  jpg, jpeg, webp (`find_logo`). `generate::copy_logos` leaves each in the
  temp dir as `.pdf`/`.png`/`.jpg` by content (SVG/WEBP converted to PNG);
  the design tests them with `\SiHayLogo`.
- **Lookup** (`resolve_logos_directory`):
  1. `--logos`/`LOGOS`;
  2. `logos/` next to the design;
  3. `logos/` in the work folder, if it holds a logo;
  4. `templates/logos/`, if it holds a logo.
- **`examples/` never depend on logos.**

## Compilation and speed

- **Draft first pass.** Without previous state, the first pass uses
  `-draftmode`.
- **State cache.** `.aux`/`.toc` are cached in `cache/latex/<pdf>-<hash>/`,
  hashed on output path + `Layout::cache_key()` (design + `apa7`, or `-` when
  self-contained). `--check-template` skips it (`skip_latex_state`). A re-run
  usually takes one pass. If the old state breaks the build, it is deleted and
  the build is retried.
- **Designs without a table of contents** write no `.toc`. `save_state` then
  saves an empty one, so the cache works for them too.
- **On failure:** `summarize_latex_errors()` keeps the real errors, and
  `last-error.tex/.log` are saved next to the PDF.
- **Logged:**
  - each pdflatex pass (draft or not) and the cache hit or miss;
  - "stable after pass N";
  - a failed attempt with the cached state, with its error, before the
    clean retry;
  - where `last-error.*` were saved.

  With `INVESTIGACION_LOG=debug`, also the full command, `TEXINPUTS` and
  timings.

## Regression rule

Before committing a change to the base, the `apa7` format or the design:

1. Render the catalog before the change (`pdftoppm -r 40`).
2. Render it again after the change.
3. Compare every page with `cmp`; they must stay byte-identical unless the
   change is intended.

After template changes, regenerate the versioned catalog with
`/regenerate-catalog`.
