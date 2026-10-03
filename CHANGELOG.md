# Changelog

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
