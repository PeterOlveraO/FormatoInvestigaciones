# Changelog

## Unreleased (branch `mejoras-rust`)

### Added

- **Formats and designs, kept separate.**
  - The format is the norm (`apa7`, `harvard`, `mla`); the design is the
    cover and the look (`geometric-cover`, `classic-cover`, `report`,
    `starter`).
  - Each design lists the formats it works with.
  - Choose them with `--format`/`--design`, in the menu or in the profile.
- **Harvard and MLA 9** formats (generic versions) next to APA 7.
- **`report` design:** a compact title block without a cover.
- **`starter` design:** heavily commented, to copy when making your own.
- **Your own formats and designs** in `my-templates/` (not uploaded to git).
  - Any `%%NAME%%` a design uses that is not a standard field becomes a field
    of its own, asked for in the menu and the wizard, saved in the profile or
    given with `--set NAME=value`.
  - `investigacion --check-template <design>` checks a design and builds the
    example catalog with each format it accepts.
- **Profile wizard.** It opens the first time and with `p`, and asks for the
  format, the design and all the data the design shows. Each profile keeps
  all its own data (university, faculty, student, semester…).
- **Document language** (`--doc-lang`, or `language` in the profile): the
  date, "Contents", "Figure" and the box titles.
- **Home view in the menu.** The menu opens on it: generate a PDF, create a
  profile, options, quit. It also shows your profiles, the last PDF and
  whether Pandoc, pdflatex and Graphviz are installed.
- **Options view** (`o` in the form): edit or create a profile, open a
  project folder, switch the language, view the log, go back home. Their
  letters (`p`, `n`, `c`/`f`, `l`, `r`, `i`/`h`) still work in the form.
- **Diagnostic log** in `cache/logs/investigacion.log`, for both the menu and
  the command line. It records each step: the options, the design and format
  chosen, every Pandoc/pdflatex run with its exit status and time, the
  LaTeX-state cache hits, every warning and error you saw, and crashes with
  their backtrace.
  - It rotates to `investigacion.old.log` past 1 MB.
  - `INVESTIGACION_LOG=debug` adds full commands, Pandoc's own messages and
    every key pressed in the menu. `INVESTIGACION_LOG=off` turns it off.
  - If it cannot be written, the program keeps working without it.
  - An unexpected file error (permissions, disk full…) now ends with
    "Details in the log: <path>", and the log records where it happened.

### Changed

- Only the data the chosen design uses is required. A design without
  `%%UNIVERSIDAD%%` no longer asks for it.
- The structure warning (Introducción… Referencias) only applies to formats
  that define it (APA 7).
- `--template` is now `--design`. The old name and the old profile key
  `template` still work, and `apa`/`apa-simple` map to
  `geometric-cover`/`classic-cover` with `apa7`.
- The form's key bar only shows move, edit/choose, clear, generate, view
  PDF, options and quit. `Esc` in the form goes back to the home view
  instead of quitting.
- Release builds keep function names (`strip = "debuginfo"`, about +0.6 MB),
  so a crash backtrace in the log shows where it happened.
- `templates/apa/` and `templates/apa-simple/` moved to
  `templates/designs/geometric-cover/` and `classic-cover/`. The shared
  preamble is now `templates/common/investigacion-base.sty`, plus one
  `format.sty` per format.

### Fixed

- The menu no longer closes when something inside it fails: the error is
  shown, logged with its backtrace, and the menu keeps running.
- Editing a design's own field (such as `%%SALON%%`) in the menu crashed it.
- On the first-run language screen, the arrow keys jumped to the profile
  wizard without choosing a language.
- Designs without a table of contents (`starter` and copies of it) never
  reused the saved LaTeX state, so every run took two pdflatex passes.
- An output or copy folder that cannot be written is reported at once, with
  its path and a hint (permissions, or the PDF open in another program),
  instead of a bare "Permission denied" after the whole build.
- An empty Markdown no longer warns about missing headings before saying it
  is empty.

## 0.2.0 (2026-10, branch `mejoras-rust`)

The program was rewritten in Rust and gained a full-screen menu, course
profiles and a choice of templates.

### Added

- **Interactive menu** (`investigacion` with no arguments). Files, course
  profiles and templates are chosen from lists, so no paths are typed. `v`
  opens the last PDF and `c`/`f` opens a project folder.
- **Bilingual interface (Spanish/English).** The menu asks the first time,
  using the system language as the default, and saves the choice as `IDIOMA` in
  `.env`. `l` switches it later, and `--lang` overrides it for one command.
- **Course profiles** in `courses/<key>.toml`: course name, teacher, group,
  members, template and folder (`-p ia`).
- **Template `apa-simple`**, a classic centered cover, alongside `apa`.
  `--template` takes a name or a path.
- **`--file-name`**: the PDF is now named after the Markdown file, independent
  of the title.

### Changed

- **Speed.** Most of the time is spent in pdflatex, not in the program, so
  re-runs now reuse the previous `.aux`/`.toc`:

  | Document | 0.1 | 0.2, first run | 0.2, re-run |
  |---|---|---|---|
  | `catalog.md`, 23 pages | 5.5 s | 5.5 s | **2.0 s** |
  | `binary-trees.md`, 12 pages | 1.9 s | 1.9 s | **1.1 s** |

- **A single binary.** There is no virtual environment, and the Lua filters
  are built into the program.
- **It runs from any folder.** `output/` is now the project's output folder,
  not the current directory's.
- **English names** for options, folders and files:

  | Before | Now |
  |---|---|
  | `--titulo`, `--materia`, `--docente`, `--integrantes`, `--grupo`, `--salida`, `--copia`, `--plantilla`, `--permitir-latex` | `--title`, `--course`, `--teacher`, `--members`, `--group`, `--output`, `--copy`, `--template`, `--allow-latex` |
  | `Latex/base.ltx`, `Latex/logos/` | `templates/apa/template.ltx` (plus `templates/common/`), `templates/logos/` |
  | `imagenes/` | `cache/` |
  | `ejemplo/` | `examples/` |
  | `ultimo-error.log` | `last-error.log` |

  The old option names still work.

### Removed

- The Python package, `pyproject.toml` and the `investigacion-tui` command.
  `investigacion` alone opens the menu.
- `.env`, "allow LaTeX", template path and logos from the menu. They remain
  available on the command line.

### Upgrading from 0.1 (Python)

1. Install Rust and run `cargo install --path .` (see [INSTALL.md](INSTALL.md)).
2. Run `deactivate` if the old environment is active, then delete `.venv/`.
3. Move your logos from `Latex/logos/` to `templates/logos/`. The old
   `imagenes/` cache can be deleted.
4. Optionally create course profiles in `courses/` from `courses/example.toml`.
