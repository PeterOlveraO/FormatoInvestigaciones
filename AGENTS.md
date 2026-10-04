# AGENTS.md

Rust CLI and full-screen menu (`investigacion`) that turns a paper written in
Markdown into a PDF: Pandoc → LaTeX (format + design) → pdflatex. Built-in
formats: APA 7 (default), Harvard and MLA 9. The project belongs to a
Spanish-speaking student; papers are Spanish by default.

## Commands

```bash
cargo build --release                     # target/release/investigacion
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test                                # all tests (PDF/Pandoc tests skip if the tools are missing)
cargo test --lib markdown                 # one module
cargo test --test pdf                     # only the tests that build real PDFs (~45 s)
cargo run --release -- Tarea1.md -p ia --title "Tema"   # CLI; no arguments opens the menu
target/release/investigacion --check-template starter    # check a design, build the catalog with it
INVESTIGACION_TIMING=1 target/release/investigacion examples/catalog.md --title T --course M
INVESTIGACION_LOG=debug target/release/investigacion …   # full log in cache/logs/investigacion.log (off: INVESTIGACION_LOG=off)
```

System dependencies (not crates): Pandoc, pdflatex, Graphviz (optional).
Minimum Rust: 1.88 (edition 2024, let-chains).

## Rules that always apply

- **Code in English, comments in Spanish.**
  - English: identifiers, modules, files, folders, CLI options
    (`--title`, `--course`…) and profile keys (`name`, `teacher`…).
  - Spanish: comments, short; one per block, per line only for special cases.
- **The interface is bilingual (Spanish/English).**
  - Every user-facing string (messages, warnings, help, TUI) goes through
    `tr!(es: "…", en: "…")` or `Text::new(es, en)` from `src/i18n.rs`. Never
    add a user-facing literal in one language only.
  - Spanish texts use accents.
- **The PDF language is the document language,** separate from the interface
  language. The format sets the default (`format.toml`), and the profile
  `language` or `--doc-lang` can change it. It drives babel, the date and the
  filters' box titles.
- **Formats and designs are separate layers.**
  - **Base** (`templates/common/`, universal): Pandoc compatibility, diagrams,
    charts, Unicode.
  - **Format** (`templates/formats/<f>/`): the norm's style.
  - **Design** (`templates/designs/<d>/`): cover and look.
  - A design must follow the contract (`latex-templates` skill), and
    `--check-template` verifies it.
- **APA output must not change by accident.** For any change to the base, the
  `apa7` format or the APA designs, compare the rendered pages before and
  after (`pdftoppm` + `cmp`). Every phase so far kept them byte-identical.
- **Keep old names working.** These stay:
  - the hidden clap aliases `--titulo`, `--materia`, `--subject`,
    `--template`…;
  - the serde aliases `subject` and `template` in profiles;
  - the legacy design names `apa`/`apa-simple`.
- **Nothing OS-specific.** It must behave the same on Linux, Windows and macOS:
  - no `cfg!(windows)`, fixed paths or hand-written path separators;
  - use `std::env::join_paths`/`split_paths`;
  - use `project::absolute()`, never `canonicalize()`;
  - use `pandoc::posix()` for paths that end up inside LaTeX.
- **Expected failures** are `GenerationError` with a message ready to show. The
  library never prints; it reports through the `Reporter` trait (console in the
  CLI, channel in the TUI).
- **Log what happens, never break because of it.** `src/logging.rs` writes
  `cache/logs/investigacion.log`.
  - Call `logging::{debug,info,warn,error}(format_args!(…))` at each step
    that can fail or that changes the PDF: `info` for steps, `warn` for
    recovered problems, `error` for failures, `debug` for detail (full
    commands, paths, keys in the menu).
  - Log lines are technical English. `execute()` wraps the `Reporter`, so
    every warning and info the user sees is already logged.
  - It is a no-op until `logging::init` (only `main.rs` calls it), so tests
    leave no files. It never returns errors and never panics; a failed write
    turns it off.
  - `std::io::Error → GenerationError` logs the `?` location
    (`#[track_caller]`) and, when the log is active, adds "Details in the
    log: <path>" to the message.
- **The menu must not crash.** Key handling, drawing and the generation
  thread run inside `tui::guard::run`: a panic is logged with its backtrace
  and shown as an error line, and the menu keeps running.
- **Always produce a PDF.** A missing image, symbol or Graphviz becomes a
  warning, not an error. The only intended hard failure is a broken
  ` ```pgfplot `/` ```tikz ` block, because that is real LaTeX.
- **Required data follows the design.** A standard field is required only if
  the design uses its marker and does not list it in `optional`. A custom
  field is required only if its spec says so (`missing_data()`).
- **No logic in the TUI.** It builds the same `cli::Args` and calls
  `cli::execute()`.
- **Precedence of values:** CLI option (incl. `--set UNIVERSIDAD=…`) > course
  profile (`pick()` in `src/cli.rs`). There is no `.env`: cover data lives in
  profiles; `settings.toml` only remembers the language and `LOGOS`.
- **Minimal dependencies:** justify every new crate.
- **Before finishing:** run `cargo fmt`, `cargo clippy --all-targets -- -D
  warnings` and `cargo test`. For template or filter changes, also generate
  `examples/catalog.md` and check the pages.
- **Never commit personal data or institutional logos.**
  - `settings.toml`, `.env` (old), `input/`, `output/`, `cache/`,
    `my-templates/`, `courses/*.toml` (except `example.toml`) and
    `templates/logos/*` are git-ignored on purpose.
  - Generated files in git (`examples/catalog.pdf`, `docs/images/*.png`) use
    the placeholders of `courses/example.toml` (`-p example`) and no logos (`/regenerate-catalog`).

## Layout

```
src/
  main.rs        CLI entry; no arguments → TUI
  cli.rs         Args (clap), localized help/errors, execute() shared with the TUI, --check-template
  i18n.rs        Lang (thread-local), tr!, Text, detect/resolve
  logging.rs     diagnostic log (cache/logs/), levels, rotation, panic hook
  template.rs    Format/Design, manifests, markers, resolve_layout (design × format)
  check.rs       --check-template: contract checks, typo suggestions, test builds
  generate.rs    the pipeline: generate_pdf, missing_data, copy_pdf_to, output_file_name
  markdown.rs    read/normalize Markdown, lookup in input/, structure warnings
  pandoc.rs      Pandoc call; Lua filters embedded with include_str!
  latex.rs       latex_escape, render_template (standard markers + custom fields)
  compile.rs     pdflatex passes, TEXINPUTS, .aux/.toc cache
  project.rs     project root, folders, logos
  settings.rs    settings.toml (language, LOGOS) + environment (never mutates the process env)
  courses.rs     course profiles (courses/*.toml), save_profile
  document.rs    DocumentData, dates (es/en), slugify
  encoding.rs    UTF-8 with Windows-1252 fallback
  process.rs     run tools with a timeout (TOOL_TIMEOUT = 180 s)
  tui/           app.rs (state), ui.rs (drawing), wizard.rs (profile wizard), picker.rs,
                 guard.rs (panic safety net), tools.rs (Pandoc/pdflatex/Graphviz check), tests.rs
tests/           integration: templates.rs, pandoc.rs, pdf.rs, cli.rs
resources/filters/   Lua filters: inline_html, images, diagrams, charts, blocks
templates/       common/ (base + final .sty), formats/{apa7,harvard,mla}/,
                 designs/{geometric-cover,classic-cover,report,starter}/, logos/
my-templates/    the user's own formats/ and designs/ (git-ignored)
courses/         course profiles; only example.toml and README.md are tracked
examples/        Spanish sample papers; catalog.md is the visual test bench
```

Pipeline: `read_markdown` → `normalize_markdown` → `resolve_layout` (design +
format) → `missing_data` → `validate_markdown` (the format's headings) →
`pandoc_to_latex` (a LaTeX **fragment**) → `render_template` → `compile_pdf`
→ `copy_pdf_to`.

## Domain knowledge (load when relevant)

Each area has a skill with the details. They load automatically when you work
on the matching files; other agents can open them by path.

| Working on | Skill |
|---|---|
| `templates/**`, `src/template.rs`, `src/latex.rs`, `src/compile.rs`, `src/check.rs`: the three layers, design contract, markers and custom fields, LaTeX traps, Unicode, logos, LaTeX cache | `.claude/skills/latex-templates/SKILL.md` |
| `src/markdown.rs`, `src/pandoc.rs`, `resources/filters/**`: supported syntax, Lua filters, document language in filters, image/diagram cache, invisible spaces, Markdown lookup | `.claude/skills/pandoc-pipeline/SKILL.md` |
| `src/i18n.rs`, `src/cli.rs`, `src/tui/**`, `src/courses.rs`: language resolution, localized clap, TUI and wizard, adding a CLI option, profiles | `.claude/skills/bilingual-interface/SKILL.md` |
| Rebuilding `examples/catalog.pdf` and the README screenshots | `.claude/skills/regenerate-catalog/SKILL.md` |

## Docs to update when behavior changes

- `README.md`: overview, install, first run, quick use.
- `INSTALL.md`: per-OS installation, configure, update, installation problems.
- `GUIDE.md`: formats and designs, profiles and wizard, menu, options,
  syntax, troubleshooting, making a design.
- `AI-PROMPT.md`: the spec an AI follows to write papers. **Update it whenever
  the accepted Markdown syntax changes.**
- `CHANGELOG.md`: user-visible changes.
- These docs are in English; `examples/` are Spanish papers.
