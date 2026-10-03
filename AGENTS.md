# AGENTS.md

Rust project (CLI and full-screen menu) that turns a paper written in Markdown
into an APA PDF.

Rules that are not negotiable when adding code:

- **Code in English, comments in Spanish.** Identifiers, modules, files,
  folders, CLI options and profile keys are in English. Comments are in
  Spanish and short.
- **Bilingual interface.** Every user-facing text has both versions through
  `tr!(es: …, en: …)` or `Text::new(es, en)` (see `src/i18n.rs`). The PDF
  stays in Spanish.
- **Minimal dependencies.** Every new crate must be justified. Pandoc, pdflatex
  and Graphviz are system dependencies.
- **Nothing system-specific.** It must work the same on Linux, Windows and
  macOS.
- **Before finishing,** run `cargo fmt`, `cargo clippy --all-targets -- -D
  warnings` and `cargo test`.

Architecture, internal contracts and the known traps are in `CLAUDE.md`.
