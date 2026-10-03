---
name: latex-templates
description: Contract between Pandoc's LaTeX output and the templates, cover-page markers and rules, known LaTeX traps (soul, floatrow, babel, hyperref), Unicode/emoji handling, logos/TEXINPUTS and the pdflatex pass cache. Use when editing templates/**, src/latex.rs, src/compile.rs or DocumentData, adding a cover field or template, or debugging a LaTeX build failure.
paths: "templates/**,src/latex.rs,src/compile.rs,src/document.rs,src/generate.rs,tests/templates.rs,tests/pdf.rs"
---

# LaTeX templates

## Structure

- `templates/common/investigacion.sty`: the shared preamble, with everything
  Pandoc's output assumes.
- `templates/common/investigacion-final.sty`: hyperref, footnotehyper and
  `\AutorPDF`. It is loaded right before `\begin{document}`.
- `templates/<name>/template.ltx`: only the data markers and the cover. It
  loads both files with `\usepackage`.
- `copy_template_assets()` copies `common/*.sty` (and any `.sty` next to the
  template) into the temp dir, which `latex_search_path()` puts first in
  `TEXINPUTS`. The trailing separator means "plus the defaults"; without it
  pdflatex cannot find its own packages.
- Inside a `.sty`: no `\makeatletter` (`@` is already a letter); load packages
  with `\RequirePackage`.
- A new template: copy `templates/apa-simple/` and change only the cover.
  `every_template_compiles_with_and_without_the_optional_fields` compiles all of
  them.

## The Pandoc contract (most common failure)

Pandoc emits a **fragment**, so the preamble must define everything it uses. A
document with tables, images or code fails while a text-only one compiles.

The `PAQUETES QUE NECESITA LA SALIDA DE PANDOC` block covers:

- tables: `calc`, `\newcounter{none}`, the `longtable` patch;
- images: `\pandocbounded`;
- code: `Shaded`/`Highlighting` and the `\...Tok` commands;
- strikethrough: `soul`/`ulem`;
- math: `amsmath`;
- footnotes in tables: `footnotehyper`.

**Remove nothing from that block.** Compare it with what the installed Pandoc
expects:

```bash
pandoc file.md -s --to=latex | sed -n '/documentclass/,/begin{document}/p'
```

Other traps:

- **Code blocks without a language** arrive as `verbatim` and inherit
  `\doublespacing`. `\AtBeginEnvironment{verbatim}{\singlespacing\small}` keeps
  ASCII art intact.
- **Figures use `floatrow`.** Never also load `float`: the two together abort
  the build.
- **Figure placement.** The `H` placement line
  (`\@ifundefined{floatsetup}...\fps@figure`) must sit where Pandoc defines its
  own; earlier, it gets overridden and figures float.
- **Long tables.** `floatrow` pushes them to the left margin, hence
  `\LTleft`/`\LTright` set to `\fill`.
- **Captions.** Babel-spanish sets captions at `\begin{document}`; wrap
  renames in `\addto\captionsspanish{...}`.
- **Package order.** `hyperref`, then `footnotehyper`, at the end of the
  preamble.

## soul: \st, \hl, \ul

soul parses letter by letter:

- inside `longtable`, `\st`/`\hl` make pdflatex **loop forever** (no error, no
  PDF);
- in a heading, the command breaks the TOC.

The fix:

- `\st` and `\ul` are redefined on `ulem` (`\sout`, `\uline`) with
  `\DeclareRobustCommand`.
- `\hl` stays soul's in running text, and `\AtBeginEnvironment{longtable}`
  swaps it for a `\colorbox`.
- Covered by `the_extended_syntax_compiles_without_lost_symbols`.
- Safety net: every tool runs with `TOOL_TIMEOUT`.

## Markers and the cover

`render_template()` replaces `%%UNIVERSIDAD%% %%FACULTAD%% %%TITULO%%
%%ALUMNO%% %%INTEGRANTES%% %%MATERIA%% %%GRUPO%% %%DOCENTE%% %%SEMESTRE%%
%%FECHA_ENTREGA%% %%CONTENIDO_MARKDOWN%%`.

- User data goes through `latex_escape()`. Pandoc's content is not escaped.
- The content is inserted **last**, so a `%%X%%` written in the paper is not
  taken as an unresolved marker.
- `%%INTEGRANTES%%`: escape **each name separately**, then join with a literal
  `\\`. Escaping the joined string turns `\\` into `\textbackslash{}`.
- The cover order is fixed in the template: student (or members), course,
  teacher, semester, group. `the_cover_order_is_fixed` checks it.
- Members **replace** the student. This is an outer
  `\ifdefempty{\ListaIntegrantes}` with the student nested in its empty branch.
  `DocumentData` keeps both values.
- `\\[0.25cm]` goes **inside** each conditional branch. A bare `\\` at the start
  of the TikZ node breaks.
- The members list is a `tabular[t]` inside
  `\raisebox{0pt}[0pt][\depth]{}`, so its height does not push the block down.
- `\AutorPDF` follows the same student/members rule;
  `\pdfstringdefDisableCommands` turns `\\` into commas.

**Adding a cover field** (use `group` as the model):

1. Add it to `DocumentData`.
2. Add the marker to `render_template()`.
3. Add a `\newcommand` plus an `\ifdefempty` line to **every** template.
4. Add the CLI option, profile key and TUI field (see the
   `bilingual-interface` skill).
5. Add tests.

## pgfplots style

The `GRÁFICAS DE DATOS` block uses `every axis/.append style`, so each chart
can override it.

- Bars use `bar cycle list`, whose default is in color; redefine it there.
- Never put `fill` in `cycle list`: it fills the area under line charts.

## Unicode and emoji

- **Declared list.** `SÍMBOLOS UNICODE` declares, with `newunicodechar`, only
  the symbols that really failed. The list was measured from the log, not
  invented.
- **Safety net.** `\UTFviii@undefined@err` is redefined to warn and print
  `[?]`, so an undeclared symbol never aborts the build.
  `unsupported_character_warnings()` reads those warnings. Their text,
  "Caracter Unicode sin definir", is matched by a regex; keep it identical.
- **Read the log with `decode_latex_log()`.** It decodes line by line, because
  pdflatex mixes UTF-8 with T1-encoded lines.
- **Box drawing:** handled by `pmboxdraw`. Rounded corners and shapes
  (`► ▲ ●`) are declared by hand.
- **Emoji:** the handler computes the code point and uses
  `\texttwemoji{<hex>}` if it exists. U+FE0F and U+200D are declared empty.

## Logos

- **Names:** `logo-universidad.png` and `logo-facultad.png`, each wrapped in
  `\IfFileExists`; both are optional.
- **Lookup** (`resolve_logos_directory()`):
  1. `--logos` (or `LOGOS`); a missing folder is an error;
  2. `logos/` next to the template;
  3. `templates/logos/`;
  4. none, which is fine.
- **Why they are copied:** pdflatex runs with the Markdown's folder as cwd, so
  the logos are copied to the temp dir (on `TEXINPUTS`) and referenced by bare
  name.
- **`examples/` never depend on logos.**

## Compilation and speed

Time is spent in pdflatex (~1.7 s per pass on the catalog), not in Rust.
`compile_pdf()` reruns until the `.toc` is stable (`MAX_LATEX_RUNS`).

- **Draft first pass.** Without previous state, the first pass uses
  `-draftmode`.
- **State cache.** `.aux`/`.toc` are cached in `cache/latex/<pdf>-<hash>/` (per
  output PDF and template). A re-run usually needs one pass.
- **Broken old state.** If the cached state breaks the build, it is deleted and
  the build is retried from scratch, without leaving `last-error.*` from the
  first attempt.
- **On failure,** `summarize_latex_errors()` keeps only the `file:line:` and `!`
  lines, and `keep_failure_artifacts()` saves `last-error.tex`/`.log` next to
  the PDF.
- After changing templates, regenerate the versioned catalog with
  `/regenerate-catalog`.
