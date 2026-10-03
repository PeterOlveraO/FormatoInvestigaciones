# AGENTS.md

Rust CLI and full-screen menu (`investigacion`) that turns a paper written in
Markdown into an APA 7 PDF: Pandoc → LaTeX template → pdflatex. The project
belongs to a Spanish-speaking student; the generated paper is always Spanish.

## Commands

```bash
cargo build --release                     # target/release/investigacion
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo test                                # all tests (PDF/Pandoc tests skip if the tools are missing)
cargo test --lib markdown                 # one module
cargo test --test pdf                     # only the tests that build real PDFs
cargo run --release -- Tarea1.md -p ia --title "Tema"   # CLI; no arguments opens the menu
INVESTIGACION_TIMING=1 target/release/investigacion examples/catalog.md --title T --course M
```

System dependencies (not crates): Pandoc, pdflatex, Graphviz (optional).
Minimum Rust: 1.88 (edition 2024, let-chains).

## Rules that always apply

- **Code in English, comments in Spanish.**
  - English: identifiers, modules, files, folders, CLI options
    (`--title`, `--course`…) and course-profile keys (`name`, `teacher`…).
  - Spanish: comments, short; one per block, per line only for special cases.
- **The interface is bilingual (Spanish/English).**
  - Every user-facing string (messages, warnings, help, TUI) goes through
    `tr!(es: "…", en: "…")` or `Text::new(es, en)` from `src/i18n.rs`. Never
    add a user-facing literal in one language only.
  - Spanish texts use accents.
  - The PDF itself (templates, captions, dates) stays in Spanish.
- **Keep old names working.** The old Spanish/previous names (`--titulo`,
  `--materia`, `--subject`…) stay as hidden clap aliases and the profile key
  `subject` as a serde alias. Do not remove them.
- **Nothing OS-specific.** It must behave the same on Linux, Windows and macOS:
  - no `cfg!(windows)`, fixed paths or hand-written path separators;
  - use `std::env::join_paths`/`split_paths`;
  - use `project::absolute()`, never `canonicalize()`, which yields `\\?\`
    paths on Windows;
  - use `pandoc::posix()` for paths that end up inside LaTeX.
- **Expected failures** are `GenerationError` with a message ready to show. The
  library never prints; it reports through the `Reporter` trait (console in the
  CLI, channel in the TUI).
- **Always produce a PDF.** A missing image, symbol or Graphviz becomes a
  warning, not an error. The only intended hard failure is a broken
  ` ```pgfplot `/` ```tikz ` block, because that is real LaTeX.
- **No logic in the TUI.** It builds the same `cli::Args` and calls
  `cli::execute()`.
- **Precedence of values:** CLI option > course profile > `.env` (`pick()` in
  `src/cli.rs`).
- **Minimal dependencies:** justify every new crate.
- **Before finishing:** run `cargo fmt`, `cargo clippy --all-targets -- -D
  warnings` and `cargo test`. For template or filter changes, also generate
  `examples/catalog.md` and check the pages.
- **Never commit personal data or institutional logos.**
  - `.env`, `input/`, `output/`, `cache/`, `courses/*.toml` (except
    `example.toml`) and `templates/logos/*` are git-ignored on purpose.
  - Generated files in git (`examples/catalog.pdf`, `docs/images/*.png`) must
    use the placeholders of `.env.example` and no logos (`/regenerate-catalog`).

## Layout

```
src/
  main.rs        CLI entry; no arguments → TUI
  cli.rs         Args (clap), localized help/errors, execute() shared with the TUI
  i18n.rs        Lang (thread-local), tr!, Text, detect/resolve
  generate.rs    the pipeline: generate_pdf, copy_pdf_to, output_file_name
  markdown.rs    read/normalize Markdown, lookup in input/, structure warnings
  pandoc.rs      Pandoc call; Lua filters embedded with include_str!
  latex.rs       latex_escape, render_template, log parsing
  compile.rs     pdflatex passes, TEXINPUTS, .aux/.toc cache
  project.rs     project root, folders, templates, logos
  settings.rs    .env + environment (never mutates the process env)
  courses.rs     course profiles (courses/*.toml)
  encoding.rs    UTF-8 with Windows-1252 fallback
  process.rs     run tools with a timeout (TOOL_TIMEOUT = 180 s)
  tui/           app.rs (state, no drawing), ui.rs (drawing), picker.rs, tests.rs
tests/           integration: templates.rs, pandoc.rs, pdf.rs, cli.rs
resources/filters/   Lua filters: inline_html, images, diagrams, charts, blocks
templates/       common/ (shared preamble .sty), apa/, apa-simple/, logos/
courses/         course profiles; only example.toml and README.md are tracked
examples/        Spanish sample papers; catalog.md is the visual test bench
```

Pipeline: `read_markdown` → `normalize_markdown` → `validate_markdown` →
`pandoc_to_latex` (a LaTeX **fragment**) → `render_template` → `compile_pdf` →
`copy_pdf_to`.

## Domain knowledge (load when relevant)

Each area has a skill with the details. They load automatically when you work
on the matching files; other agents can open them by path.

| Working on | Skill |
|---|---|
| `templates/**`, `src/latex.rs`, `src/compile.rs`: Pandoc ↔ template contract, cover markers, soul/floatrow traps, Unicode, logos, LaTeX cache | `.claude/skills/latex-templates/SKILL.md` |
| `src/markdown.rs`, `src/pandoc.rs`, `resources/filters/**`: supported syntax, Lua filters, image/diagram cache, invisible spaces, Markdown lookup | `.claude/skills/pandoc-pipeline/SKILL.md` |
| `src/i18n.rs`, `src/cli.rs`, `src/tui/**`, `src/courses.rs`: language resolution, localized clap, TUI, adding a CLI option, profiles | `.claude/skills/bilingual-interface/SKILL.md` |
| Rebuilding `examples/catalog.pdf` and the README screenshots | `.claude/skills/regenerate-catalog/SKILL.md` |

## Docs to update when behavior changes

- `README.md`: overview, install, configure, quick use.
- `INSTALL.md`: per-OS installation, update, installation problems.
- `GUIDE.md`: options table, profiles, templates, syntax, troubleshooting.
- `AI-PROMPT.md`: the spec an AI follows to write papers. **Update it whenever
  the accepted Markdown syntax changes.**
- `CHANGELOG.md`: user-visible changes.
- These docs are in English; `examples/` are Spanish papers.
