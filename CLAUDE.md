# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Language rules

The project belongs to a Spanish-speaking student:

- **Code is in English:** identifiers, modules, files, folders, CLI options and
  course-profile keys. The old Spanish option names (`--titulo`, `--materia`,
  `--subject`…) stay as hidden clap aliases, and the old profile key `subject`
  as a serde alias.
- **Comments are in Spanish**, short: one per block, and per line only for
  special cases (LaTeX traps, portability).
- **The interface is bilingual** (Spanish/English): menu, messages, help and
  warnings. Every user-facing string goes through `tr!` or `Text` (see
  `i18n.rs`). Spanish is written with accents; Rust writes Unicode to the
  Windows console, so the old Python "no accents" rule no longer applies.
- **The PDF is always Spanish** (templates, "Contenido", dates). That is the
  paper's language, not the interface's.
- **Docs are in English** (README, INSTALL, GUIDE, CHANGELOG, AI-PROMPT, this
  file). `examples/` are Spanish papers.

## Commands

```bash
cargo build --release                 # target/release/investigacion
cargo install --path .                # installs into ~/.cargo/bin
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test                            # all tests
cargo test --lib markdown             # one module
cargo test --test pdf                 # the ones that build real PDFs

# Run (no arguments opens the TUI)
cargo run --release -- Economia.md --title "Tema" --course "Materia" --lang en
cargo run --release -- Tarea1.md -p ia --title "Tema"

# Where the time goes
INVESTIGACION_TIMING=1 target/release/investigacion examples/catalog.md --title T --course M
```

Crate dependencies: `clap`, `regex`, `unicode-normalization`, `tempfile`,
`thiserror`, `chrono`, `serde` + `toml` (profiles), `ratatui` + `crossterm`
(TUI), `opener` (open folders and PDFs), `sys-locale` (system language).
Pandoc, pdflatex and Graphviz are **system** dependencies. Minimum Rust: 1.88
(`rust-version`; the code uses edition 2024 let-chains).

## Architecture

One-way pipeline, no persistent intermediate state (except the caches):

```
paper.md
  → read_markdown()      UTF-8 with CP1252 fallback
  → normalize_markdown() strips invisible spaces (called by read_markdown)
  → validate_markdown()  warnings only, never blocks
  → pandoc_to_latex()    Pandoc emits a LaTeX FRAGMENT (no preamble), with
                         MARKDOWN_EXTENSIONS and the Lua filters
  → render_template()    replaces the %%NAME%% markers of the template
  → compile_pdf()        pdflatex in a temp dir, as many passes as needed
  → copy_pdf_to()        optional copies
```

| Module | Role |
|---|---|
| `i18n.rs` | Interface language: thread-local `Lang`, `tr!`, `Text`, `detect()`, `resolve()` |
| `encoding.rs` | UTF-8 with Windows-1252 fallback; pdflatex log decoded line by line |
| `markdown.rs` | `read_markdown`, `normalize_markdown`, lookup in `input/`, validation |
| `latex.rs` | `latex_escape`, `render_template`, log parsing |
| `pandoc.rs` | `pandoc_to_latex`; the Lua filters are **embedded** with `include_str!` |
| `compile.rs` | pdflatex passes, TEXINPUTS, `.aux`/`.toc` cache |
| `generate.rs` | `generate_pdf` (the pipeline), `copy_pdf_to`, PDF name |
| `project.rs` | project root, folders, templates and logos |
| `settings.rs` | `.env` + environment (never mutates the process env); `save_value` |
| `courses.rs` | course profiles (`courses/*.toml`) |
| `cli.rs` | `Args` (clap), localized help/errors, `execute()` shared with the TUI |
| `tui/` | full-screen menu: `app.rs` (state), `picker.rs`, `ui.rs` |
| `process.rs` | running tools with a timeout |

Every expected failure is a `GenerationError` (a message to show as-is);
`cli::main_with_args` prints it and returns 1. `execute()` reports through a
`Reporter` (console in the CLI, a channel in the TUI); the library never prints
by itself.

### The interface language

- **Resolution:** `--lang` > `IDIOMA` (env var or `.env`, also
  `INTERFACE_LANGUAGE`) > `sys-locale` (`es*` → Spanish, anything else →
  English).
- **Thread-local language:** each test thread sets its own, so parallel tests
  never race. The TUI copies it into the worker thread that generates the PDF.
- **`tr!(es: "…", en: "…", args)`** requires both versions at the call site, so
  a message cannot be added untranslated. **`Text { es, en }`** holds constant
  labels. clap's help template uses `Text`, never `format!`, because its
  `{usage}`-style placeholders would be eaten.
- **CLI:** `main_with_args` pre-scans argv for `--lang`/`--env-file` before
  parsing, so `--help` and argument errors already come out in the right
  language.
  - `localized_command()` rebuilds the help per language (`mut_arg` over
    `ARG_HELP`, custom `-h`/`-V`).
  - `describe_clap_error()` translates the common error kinds.
- **TUI:** with no `IDIOMA` saved it opens `Mode::ChooseLanguage` (system
  language preselected), and Enter saves the choice with
  `Settings::save_value`. `l` switches and saves.
- **Lua filters:** they receive `INVESTIGACION_LANG` and pick their warning
  text with `texto(es, en)`.

### The critical contract: template ↔ Pandoc output

Pandoc produces a **fragment**, not a full document, so the template must
provide the whole preamble that Pandoc's output assumes. This is the project's
most common source of failures: a document with tables, images or code blocks
fails while a text-only one compiles fine.

The templates have two levels:

- **`templates/common/investigacion.sty`:** the shared preamble (everything
  below).
- **`templates/common/investigacion-final.sty`:** the closing part (hyperref,
  footnotehyper, `\AutorPDF`).
- **`templates/<name>/template.ltx`:** only the data and the cover; it loads
  both files with `\usepackage`.

`copy_template_assets()` copies the `.sty` files of `common/` (and any the
template brings) into the temp dir, which is first in `TEXINPUTS`. Inside a
`.sty` there is no `\makeatletter` (`@` is already a letter) and packages are
loaded with `\RequirePackage`.

The `PAQUETES QUE NECESITA LA SALIDA DE PANDOC` block of `investigacion.sty`
currently covers:

- tables: `calc`, `\newcounter{none}`, the `longtable` patch;
- images: `\pandocbounded`;
- code: `Shaded`/`Highlighting` and the `\...Tok` commands;
- strikethrough: `soul`;
- math: `amsmath`;
- footnotes in tables: `footnotehyper`.

**Do not remove anything from that block.** Before touching it, compare with
what the installed Pandoc expects:

```bash
pandoc file.md -s --to=latex | sed -n '/documentclass/,/begin{document}/p'
```

A code block without a language arrives as `verbatim`, not `Shaded`, so it
inherits the document's `\doublespacing`: ASCII art comes out stretched with
gaps in the vertical connectors. `\AtBeginEnvironment{verbatim}{\singlespacing\small}`
gives it the same treatment.

Figures go through `floatrow`:

- **`floatrow` and `float` cannot coexist.** Loading both aborts the build.
  The `H` placement (the figure stays where it was written) comes from
  `floatrow` itself.
- **Where the placement line goes.** The line
  `\@ifundefined{floatsetup}...\fps@figure` must sit **where Pandoc defines its
  own**. Placed earlier, Pandoc overrides it and figures float again.
- **Long tables.** `floatrow` pushes long tables to the left margin, hence
  `\LTleft`/`\LTright` set to `\fill`.

Two more template traps:

- **Babel-spanish sets the captions at `\begin{document}`.** A bare
  `\renewcommand{\contentsname}{...}` has no effect; wrap it in
  `\addto\captionsspanish{...}`.
- **`hyperref` and `footnotehyper`** go at the end of the preamble, in that
  order.

### Template markers

`render_template()` replaces `%%UNIVERSIDAD%%`, `%%FACULTAD%%`, `%%TITULO%%`,
`%%ALUMNO%%`, `%%INTEGRANTES%%`, `%%MATERIA%%`, `%%GRUPO%%`, `%%DOCENTE%%`,
`%%SEMESTRE%%`, `%%FECHA_ENTREGA%%` and `%%CONTENIDO_MARKDOWN%%`.

- **Escaping.** User data goes through `latex_escape()`. Pandoc's content is
  **not** escaped.
- **Content goes in last**, so a `%%...%%` mentioned inside the paper is not
  mistaken for an unresolved marker.
- **Adding a marker** means touching `DocumentData`, the list in
  `render_template()` and **every** template at once. The test
  `every_template_compiles_with_and_without_the_optional_fields` guards this.
- **`%%INTEGRANTES%%`** is the only marker not produced by a single
  `latex_escape()`. `DocumentData.members` is a `Vec<String>`; **each name is
  escaped separately** and then joined with a literal `\\`. The other way
  round, `latex_escape()` would turn those backslashes into `\textbackslash{}`
  and the names would end up on one line.

### Optional cover data: student, members, teacher, group

- **What is required.** Only `--title` is mandatory, plus a course from
  `--course` or a profile.
- **Optional lines.** `--teacher`, `--members` and `--group` follow the same
  pattern: option > profile > `.env` (`pick()` in `cli.rs`). An empty line is
  omitted, so only `MATERIA` and `SEMESTRE` always appear.
- **`.env` keys.** `REQUIRED_ENV` is `UNIVERSIDAD`, `FACULTAD` and `SEMESTRE`
  (each also accepts an English name, e.g. `UNIVERSITY`). `ALUMNO` and
  `INTEGRANTES` are optional.
- **Members** are given as one argument and split by `parse_members()`.
- **The template to copy for a new optional cover field** is `--group`,
  together with its `\ifdefempty`. `--logos` repeats the same pattern with a
  path.
- **The order of the cover block is fixed** (student or members, course,
  teacher, semester, group) and lives in the template, not in the CLI.
  `the_cover_order_is_fixed` (`tests/templates.rs`) checks `apa`; `apa-simple`
  follows the same order.
- **Members and student are mutually exclusive.** With members the cover lists
  the team and omits `ALUMNO:` even if it is set; without members it shows the
  student; with neither, no line. This is an outer
  `\ifdefempty{\ListaIntegrantes}` with the student case nested in its empty
  branch. The decision lives in the template, so `DocumentData` keeps both
  values and `render_template()` does not filter them.
- **`\\[0.25cm]` goes inside each conditional branch.** A bare `\\` at the
  start of the TikZ node leaves an empty line or errors.
- **The members list** goes in a `tabular[t]`, wrapped in
  `\raisebox{0pt}[0pt][\depth]{}` so its height does not exceed `\topskip` and
  push the block down.
- **PDF metadata** use `\AutorPDF`, with the same rule as the cover; `\\`
  becomes a comma through `\pdfstringdefDisableCommands`.

### Supported Markdown

The goal is to cover the whole [Markdown Guide](https://www.markdownguide.org),
basic and extended. Pandoc's `markdown` dialect has almost everything; the rest
is added in three places:

1. **`MARKDOWN_EXTENSIONS` in `pandoc.rs`:** `mark` (`==highlight==`), `emoji`
   (`:joy:`) and `autolink_bare_uris`.
2. **The Lua filters of `resources/filters/`,** applied in `LUA_FILTERS` order.
   They are embedded in the binary, not in `templates/`, because they do not
   depend on `--template`. They work on the parsed tree, so they never touch
   the inside of a code block.
   - `inline_html`: inline HTML (`<br>`, `<mark>`, `<sub>`, `<kbd>`, `<img>`…).
     The LaTeX writer **drops HTML silently**, so without this filter that
     content vanishes from the PDF.
   - `images`: downloads web images and warns about missing local ones.
   - `diagrams`: draws ` ```dot ` blocks with Graphviz.
   - `charts`: passes ` ```pgfplot `/` ```tikz ` blocks to LaTeX inside
     `tikzpicture`, and inside a figure when they have a `caption`. It is the
     only piece where a document error **stops** the build, because it is real
     LaTeX.
   - `blocks`: `::: nota` boxes and the hanging indent of references. They emit
     `CajaMarcada` and `ReferenciasAPA`, defined in `investigacion.sty`; change
     one, change the other.
3. **`templates/common/investigacion.sty`:** the commands Pandoc assumes.

Filter warnings go to stderr prefixed with `[investigacion]`.
`filter_warnings()` separates them from Pandoc's noise and passes them to the
same `on_warning` callback that `compile_pdf()` uses.

```bash
pandoc file.md --from=markdown-raw_tex+mark+emoji+autolink_bare_uris \
  --to=latex --wrap=none --lua-filter=resources/filters/inline_html.lua
```

### The soul trap: \st, \hl and \ul

Pandoc emits `\st`, `\hl` and `\ul` and expects `soul` to define them. soul
parses text letter by letter, and that breaks in two places a school paper uses
daily:

- **Inside `longtable`** (Pandoc's tables), `\st` and `\hl` leave pdflatex
  **looping forever**: no error, no PDF, no growing log.
- **In a heading,** the command travels to the TOC and the build aborts.

The fix in `investigacion.sty`:

- **`\st` and `\ul`** are redefined on top of `ulem` (`\sout`, `\uline`) with
  `\DeclareRobustCommand`.
- **Highlight** has no line-breaking substitute, so soul's version stays in
  running text, and `\AtBeginEnvironment{longtable}` swaps it for a
  `\colorbox` inside tables.
- **Covered by** `the_extended_syntax_compiles_without_lost_symbols`
  (`tests/pdf.rs`).
- **Safety net:** `compile_pdf()` and `pandoc_to_latex()` run with
  `TOOL_TIMEOUT` (180 s).

### Charts with pgfplots

The `GRÁFICAS DE DATOS` block of `investigacion.sty` loads `pgfplots` with a
sober style (grayscale, faint grid) in `every axis/.append style`, so each
chart can override it.

- **Bars use `bar cycle list`, not `cycle list`.** The default bar list is in
  color.
- **No `fill` in `cycle list`.** It would fill the area under line charts; fill
  lives only in `bar cycle list`.

### Images and diagrams: the project cache

pdflatex downloads nothing, so `filters/images.lua` fetches web images with
`pandoc.mediabag.fetch` and rewrites the path to the local copy.
`Project::media_directories()` creates `cache/remote/` and `cache/diagrams/`.

- **File names:** the `sha1` of the URL or the diagram code.
- **Paths for the filters:** through `INVESTIGACION_REMOTE_IMAGES`,
  `INVESTIGACION_DIAGRAMS` and `INVESTIGACION_RESOURCES`, together with
  `--resource-path`.
- **Failures never break the build.** On a failure (no network, missing image,
  no Graphviz), the filter replaces the image with its alt text, or leaves the
  diagram as code, and warns.
- **Formats:** pdflatex only handles PNG, JPG and PDF; the filter rejects the
  rest with a warning.
- **`User-Agent`:** Pandoc sends none, and some sites answer 400 without one,
  hence the `--request-header`.

### The TUI

`src/tui/` uses ratatui + crossterm (works on the Windows console; curses does
not). It duplicates no logic: `App::build_args()` builds the same `Args` as the
CLI, and a thread calls `cli::execute()` through a `ChannelReporter`. Empty
fields are not passed, so profile and `.env` values still apply.

- **State and drawing are separate.** `app.rs` holds the state and the key
  handling and **does not draw**; it is tested with simulated keys and
  `TestBackend` (`src/tui/tests.rs`). `ui.rs` only draws.
- **One picker for everything.** `picker.rs` serves the Markdown (folder
  browsing), profiles, templates and project folders. The filter ranks an exact
  name, then a prefix, then a substring, then the description; without that,
  `ia` picked `example` because its course name contains "mater**ia**".
- **Keys work in both languages:** `g` generate, `v` view, `c`/`f` folders,
  `l` language, `s`/`q`/Esc quit. Each language's footer shows its own letters.
- **Windows** also sends key-release events; the loop keeps only
  `KeyEventKind::Press`.
- **What is not in the menu:** `.env`, "allow LaTeX", a template path and
  logos (they stay in the CLI). A new CLI option that belongs in the menu goes
  into `FIELDS` (label, help, example and "if empty", all as `Text`) and
  `build_args()`.

### Course profiles and the PDF name

- **`courses/<key>.toml`** holds `name`, `teacher`, `group`, `members`,
  `template` and `folder` (`CourseProfile`).
- **Typos are errors.** `deny_unknown_fields` makes a misspelled field fail
  instead of being ignored.
- **`folder`:** `locate_markdown()` searches `input/<folder>` first, and the
  default output is `output/<folder>`.
- **The PDF name does not come from the title.** `output_file_name()` uses the
  Markdown's name, or `--file-name`. The TUI proposes it when a Markdown is
  chosen and stops once the user types one.

### Where the Markdown is looked up

`find_markdown()` tries the path as given, then the path inside `input/`, then
(for a bare name) a recursive search under `input/`. Two decisions not to
reverse:

- **An existing path always wins.** The name search is the last resort.
- **Two files with the same name are an error,** not an arbitrary choice; the
  message lists the subfolders.

### Invisible spaces in the Markdown

Documents exported from other tools (or written by an AI) often carry
**non-breaking spaces** on the lines between paragraphs and two spaces at the
end of each line. Those lines are not empty, so Pandoc merges the whole
document into one block: `###` appear literally, tables become rows of bars,
the TOC has one entry. It compiles without error, which is why it goes
unnoticed.

- **What it fixes.** `normalize_markdown()` trims the end of each line (hard,
  figure, narrow and zero-width spaces) and repairs markers separated from the
  text by a hard space (`###`, bullets, numbered lists, quotes).
- **Trailing spaces are both garbage and syntax** (a line break).
  `has_disguised_spaces()` decides: if any line ends in a disguised space,
  everything is trimmed; otherwise only the disguised spaces and tabs are
  trimmed, and line breaks survive.
- **What it leaves alone:** hard spaces inside the text and the inside of
  ``` / ~~~ blocks.

### Unicode symbols

pdflatex with `inputenc` only typesets declared characters; a stray `≠` aborts
with "Unicode character not set up for use with LaTeX".

- **The declared list.** The `SÍMBOLOS UNICODE` block declares, with
  `newunicodechar`, the symbols that were really missing. The list was
  measured, not invented: a document with all candidates was compiled and the
  log was read.
- **The safety net.** `\UTFviii@undefined@err` is redefined so an undeclared
  symbol produces a **warning** and a visible `[?]` instead of aborting.
  `unsupported_character_warnings()` reads those warnings, which the CLI and
  TUI show.
- **Reading the log.** It must be read with `decode_latex_log()` (line by
  line). pdflatex mixes UTF-8 with lines in the font encoding (T1); a cp1252
  fallback for the whole file turned "└" into "â””".
- **Box drawing** (`└ ─ ┼ │`) is handled by `pmboxdraw`. Rounded corners and
  shapes (`► ▲ ●`) are declared by hand.
- **Emoji:** `\UTFviii@undefined@err` computes the code point and uses
  `\texttwemoji{<hex>}` **if it exists**. U+FE0F and U+200D are declared empty,
  so no `[?]` appears next to each emoji.

### Logos and TEXINPUTS

- **No logos in git.** `templates/logos/` is in `.gitignore` (except its
  README): institutional logos are rarely redistributable. The template looks
  for `logo-universidad.png` and `logo-facultad.png`, each wrapped in
  `\IfFileExists`.
- **Where they come from.** `Project::resolve_logos_directory()` tries:
  1. `--logos` (or `LOGOS`);
  2. `logos/` next to the template;
  3. `templates/logos/`.

  A missing explicit folder is an error; no folder at all is not.
- **Why TEXINPUTS.** pdflatex runs with the Markdown's folder as cwd, and TeX
  resolves images against the cwd. So `copy_template_assets()` copies the logos
  and the `.sty` files into the temp dir, and `latex_search_path()` puts that
  dir first in `TEXINPUTS`. The trailing separator means "plus the defaults";
  without it pdflatex cannot even find its own packages.
- **`examples/` must not depend on the logos;** they use
  `examples/sample-image.png`.

### LaTeX compilation and speed

`compile_pdf()` repeats pdflatex until the `.toc` is stable (at most
`MAX_LATEX_RUNS`). **Time is spent in pdflatex** (~1.7 s per pass on the
catalog; Pandoc ~0.3 s), not in the program. Two optimizations:

- **Draft first pass.** Without previous state, the first pass runs with
  `-draftmode` (no PDF written); it can never be the final one.
- **LaTeX state cache.** The `.aux` and `.toc` of the last build are kept in
  `cache/latex/<pdf>-<hash>/`, one folder per output PDF and template. With
  them the first pass already reads the right TOC, and if nothing changed it is
  the only pass (catalog: from 3 passes and 5.5 s to 1 pass and 2.0 s). If the
  old state breaks the build, `compile_pdf()` deletes it and starts over,
  **without** leaving `last-error.*` from the first attempt.
- **When a build fails,** `summarize_latex_errors()` keeps only the real errors
  (`file:line:` and `!` lines), and `keep_failure_artifacts()` saves
  `last-error.tex`/`.log` next to the PDF.

### raw_tex disabled by default

`pandoc_to_latex()` uses `markdown-raw_tex` unless `--allow-latex` is given.
Otherwise a stray backslash (`C:\Users\...`) would be sent to LaTeX as a
command and abort the build. `$...$` formulas work in both modes.

### Structure validation

`validate_markdown()` only warns, and never blocks, if `Introducción`,
`Desarrollo`, `Conclusión` or `Referencias` are missing or out of order. These
heading names are Spanish in both interface languages, because they are the
paper's. `markdown_headings()` ignores fenced code blocks.

## Portability

Developed on Linux, but it must work the same on Windows and macOS, so
**nothing system-specific**: no fixed paths, no `cfg!(windows)`, no hand-written
separators. Already solved, do not undo:

- **Search-path separators:** `std::env::join_paths`/`split_paths` for
  `TEXINPUTS` and `--resource-path` (`;` on Windows, `:` elsewhere).
- **No `canonicalize()`.** `project::absolute()` (`std::path::absolute`) is
  used instead, because on Windows `canonicalize` returns `\\?\C:\...` paths
  that pdflatex and Pandoc do not understand.
- **Forward slashes for the filters.** `pandoc::posix()` is applied to the
  paths passed to the Lua filters; they end up inside `\includegraphics`,
  where a backslash would start a command.
- **Temp dir cleanup.** `temporary.close()` errors are ignored, because Windows
  may keep a pdflatex file locked for a moment after the PDF is copied.
- **System programs.** crossterm (filtering `KeyEventKind::Press`) and `opener`
  for opening folders and PDFs; `sys-locale` for the language, since `LANG`
  does not exist on Windows.
- **Text encoding.** All text reading goes through `decode_text()` (UTF-8 or
  Windows-1252, never the system default).
- **Line endings.** `.gitattributes` normalizes them to LF.

## Directories

- `src/`: the Rust code. `tests/`: integration tests (`templates.rs`,
  `pandoc.rs`, `pdf.rs`, `cli.rs`).
- `templates/`: `common/` (shared preamble), one folder per template (`apa/`,
  `apa-simple/`) and `logos/` (ignored except README).
- `resources/filters/`: Lua filters, embedded in the binary.
- `courses/`: course profiles (ignored except `example.toml` and README).
- `cache/`: `remote/`, `diagrams/`, `latex/`. Created automatically, safe to
  delete, ignored.
- `input/`, `output/`: the user's papers and PDFs, ignored.
- `examples/`:
  - `paper-template.md`, `syntax.md`, `binary-trees.md`;
  - `catalog.md`, one of each element, which is **the visual test bench** for
    template or filter changes;
  - `catalog.pdf`, its versioned output.
- `docs/images/`: README screenshots.

`examples/catalog.pdf` and the screenshots are the only generated files in git,
and **must not contain personal data or logos**. Regenerate them like this:

```bash
mkdir -p /tmp/nologos
investigacion examples/catalog.md \
  --title "Catalogo de elementos" --course "Nombre de la materia" \
  --teacher "Nombre del docente" --group "7-A" \
  --env-file .env.example --logos /tmp/nologos --output /tmp/pub --lang es
cp /tmp/pub/catalog.pdf examples/catalog.pdf
pdftoppm -r 110 -png -f 1 -l 1 examples/catalog.pdf /tmp/cover && cp /tmp/cover-01.png docs/images/cover.png
```

`--env-file .env.example` makes the cover show the same placeholders a new
clone sees, and the empty `--logos` folder avoids publishing an institution's
marks.

## Documentation

When behavior changes, check which file it affects:

- `README.md`: overview, install, configure, quick use.
- `INSTALL.md`: per-OS installation, update, installation problems.
- `GUIDE.md`: every option, profiles, templates, syntax, troubleshooting.
- `AI-PROMPT.md`: the prompt the AI reads. **If the accepted syntax changes,
  update it**: it is the specification the model follows.
- `CHANGELOG.md`: user-visible changes per version.
- `CLAUDE.md`: this file, with internal contracts and traps.
