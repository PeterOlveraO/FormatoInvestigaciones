---
name: latex-templates
description: The three template layers (universal base, format, design), the design contract and its markers, custom fields, manifests (format.toml, template.toml), known LaTeX traps (soul, floatrow, babel, hyperref), Unicode/emoji, logos/TEXINPUTS and the pdflatex pass cache. Use when editing templates/**, src/template.rs, src/latex.rs, src/compile.rs or src/check.rs, adding a format, a design or a cover field, or debugging a LaTeX build failure.
paths: "templates/**,src/template.rs,src/latex.rs,src/compile.rs,src/check.rs,src/document.rs,src/generate.rs,tests/templates.rs,tests/pdf.rs"
---

# LaTeX templates: base, format and design

## The three layers

```
DESIGN  templates/designs/<d>/template.ltx + template.toml   cover, look, custom fields
FORMAT  templates/formats/<f>/format.sty  + format.toml      the norm: fonts, spacing, headings, captions, references
BASE    templates/common/investigacion-base.sty / -final.sty  universal: Pandoc compatibility, diagrams, charts, Unicode
```

- **Built-ins:**
  - formats `apa7`, `harvard`, `mla`;
  - designs `geometric-cover`, `classic-cover`, `report`, `starter`.
- **The user's own** go in `my-templates/formats|designs/`, searched after the
  built-ins (`template.rs`: `roots()`).
- **Selection.** `resolve_layout(project, design, format)` loads the design,
  then picks the format: the given one, else the first in the design's
  `formats`, else `apa7`. It errors if the design does not accept it. A design
  without `%%FORMAT%%` is self-contained and gets no format.
- **Legacy names.** `apa` → `geometric-cover` and `apa-simple` →
  `classic-cover` (`legacy_design`).

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
  any `.sty` next to the design, the chosen `format.sty` (renamed to
  `investigacion-format.sty`) and the logos into the temp dir, which is first
  in `TEXINPUTS`.
- **Babel follows the document language** (`Layout::format_block`): es →
  `spanish, es-tabla`, en → `english`. Anything Spanish-only in a format must
  be guarded, e.g. `\@ifundefined{captionsspanish}{}{\addto\captionsspanish{…}}`.
- **Hooks the base defines and formats redefine:**
  - `\FormatBodySetup`: alignment and indent after the cover;
  - `ReferenceList`: neutral in the base; hanging indent in apa7, harvard and
    mla.
- **Inside a `.sty`:** no `\makeatletter`; load packages with
  `\RequirePackage`.
- **Comment lines are not scanned for markers** (`extract_markers`), so a
  comment can mention `%%X%%`. Never write `%%FORMAT%%` inside a comment
  anyway, because it is replaced by a two-line block.

## Markers and data

- **Standard markers** (`STANDARD_MARKERS`): `TITULO UNIVERSIDAD FACULTAD
  ALUMNO INTEGRANTES MATERIA DOCENTE GRUPO SEMESTRE FECHA_ENTREGA
  CONTENIDO_MARKDOWN FORMAT CLASS_OPTIONS`.
- **Custom fields.** Any other `%%NAME%%` is a custom field: filled from
  `DocumentData.fields` (the profile's `[fields]`, `--set`, the TUI extras),
  escaped, empty if missing.
- **Required data** (`generate::missing_data`):
  - standard: `TITULO`, `UNIVERSIDAD`, `FACULTAD`, `SEMESTRE`, `MATERIA`,
    only if the design uses them and does not list them in `optional`;
  - custom: only if `[fields.X] required = true`.
- **Escaping.** `render_template` escapes user data with `latex_escape`.
  Pandoc's content is not escaped and goes in **last**.
- **`%%INTEGRANTES%%`:** each name is escaped separately, then joined with a
  literal `\\`.
- **Built-in covers:**
  - order: student or team, course, teacher, semester, group;
  - members replace the student (`\ifdefempty{\ListaIntegrantes}`);
  - `\\[0.25cm]` goes inside each conditional branch.
- **Adding a standard marker** means changing `DocumentData`,
  `render_template`, `STANDARD_MARKERS`, `missing_data`, the wizard's
  `STANDARD_STEPS` and the designs.

## Manifests

```toml
# format.toml
name = { es = "APA 7", en = "APA 7" }
language = "es"                       # document language default
class_options = "12pt, letterpaper"
headings = ["Introducción", "Desarrollo", "Conclusión", "Referencias"]  # [] = no structure check

# template.toml (optional)
description = { es = "…", en = "…" }
formats = ["apa7", "harvard", "mla"]  # empty = any
optional = ["UNIVERSIDAD"]            # standard markers shown but not required
[fields.SALON]
label = { es = "Salón", en = "Room" }
help = { es = "…", en = "…" }
required = true
```

Both manifests use `deny_unknown_fields`. Bilingual values are `Localized`
(plain or `{es, en}`).

## Adding a format

1. Copy `templates/formats/harvard/` and edit `format.sty`:
   - geometry, fonts, spacing, `titlesec`, captions;
   - floatrow caption position (guard it with
     `\@ifpackageloaded{floatrow}`), `fancyhdr`;
   - `\renewenvironment{ReferenceList}`, `\renewcommand{\FormatBodySetup}`.
2. Keep it **single-column**. Pandoc tables are `longtable`, which fails in
   `twocolumn`; that is why IEEE was removed.
3. Add it to the designs' `formats` lists. `tests/pdf.rs` then compiles every
   design × format pair, and `--check-template` reports per format.

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

- **Names:** `logo-universidad.png` and `logo-facultad.png`, wrapped in
  `\IfFileExists`.
- **Lookup** (`resolve_logos_directory`):
  1. `--logos`/`LOGOS`;
  2. `logos/` next to the design;
  3. `templates/logos/`.
- **`examples/` never depend on logos.**

## Compilation and speed

- **Draft first pass.** Without previous state, the first pass uses
  `-draftmode`.
- **State cache.** `.aux`/`.toc` are cached in `cache/latex/<pdf>-<hash>/`,
  hashed on output path + `Layout::cache_key()` (design + format). A re-run
  usually takes one pass. If the old state breaks the build, it is deleted and
  the build is retried.
- **On failure:** `summarize_latex_errors()` keeps the real errors, and
  `last-error.tex/.log` are saved next to the PDF.

## Regression rule

Before committing a change to the base, the `apa7` format or the APA designs:

1. Render the APA catalog with both covers before the change (`pdftoppm -r 40`).
2. Render it again after the change.
3. Compare every page with `cmp`; they must stay byte-identical unless the
   change is intended.

After template changes, regenerate the versioned catalog with
`/regenerate-catalog`.
