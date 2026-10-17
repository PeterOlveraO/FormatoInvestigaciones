# Changelog

## Unreleased

**Windows is supported** (tested on Windows 11 with MiKTeX). Installation steps
are in `INSTALL.md`.

### Changed

- The profile wizard no longer asks for an output folder: the PDF goes to
  `output/<folder>` of the profile. The `output` key still works in profiles
  that have it, and the menu still lets you choose the folder.

### Fixed

- A Spanish paper with `\,\%` (or `\;\%`, `\!\%`) inside math, such as
  `$28.9\,\%$`, no longer fails with "Incompatible glue units".
- English documents (`--doc-lang en`) no longer get Spanish heading warnings
  («Introducción»…) or Spanish cover labels (Alumno, Materia, Fecha de entrega).
- A language with a region (`en-US`, `es_MX`) now works in profiles,
  `--doc-lang`, `--lang`, `settings.toml` and `format.toml`; an unknown
  `language` in `format.toml` is an error instead of silently becoming Spanish.
- A heading or list pasted with an invisible space in front (from Word) is no
  longer printed as plain text with a literal `#`.
- One invisible space inside a code block no longer removes the two-space line
  breaks of the whole document.
- A file that is mostly UTF-8 with one Windows-1252 character no longer turns
  every accent into `Ã³`; UTF-16 files (Notepad "Unicode") are read.
- Raw LaTeX in a diagram or chart caption no longer breaks the build (it is
  only allowed with `--allow-latex`, as in the body).
- An image path that points to a folder gives the "image not found" warning
  instead of a LaTeX error.
- A project in a folder whose name contains `:` (Linux/macOS) builds; its
  images are copied to the cache.
- When the PDF cannot be written (open in a viewer, permissions), the new PDF
  is kept in the cache and the error says where.
- `last-error.tex`/`last-error.log` are removed after a later successful build.
- PDF names: Windows reserved names (`con`, `aux`…) get a prefix and very long
  names are shortened.
- `install.ps1` reports a failed `cargo clean`.
- `--check-template` warns about malformed markers (`%%AULA2%%`, `%%salon%%`),
  and `%%CONTENIDO_MARKDOWN%%` in a comment no longer receives the paper.
- Menu: `v` no longer opens the previous PDF after a failed generation; a
  broken `settings.toml` is reported at start and never appended to;
  `IDIOMA="en" # nota` reads `en`; Ctrl+letter no longer acts as the letter
  (Ctrl+G does not generate); Ctrl+C during a generation asks before leaving; a
  panic while collecting a result no longer closes the menu.
- Choosing the Markdown again in the menu opens the folder of the file already
  chosen instead of the project folder.
- `~` in the menu's output and copy folders now means the home folder.
- `--set MATERIA=…` now sets the course, as the "Missing data" message
  suggests; before, it was ignored and the error repeated.
- Renaming a profile in the wizard no longer leaves the old file behind (two
  profiles with the same data).
- A ```` ```python ```` line inside a ```` ```` ```` block no longer ends the
  block early, so `#` comments in that code are not taken as headings.

- The PDF build no longer fails with "I can't find file `C:/Users/...'" when
  the temp folder is a short path with `~` or the user folder has spaces.
  pdflatex now runs inside the temp folder and gets only the file name.
- Diagrams and downloaded images work when the project is in a folder with
  accents or spaces (`C:\Users\José Pérez\...`): LaTeX gets only their file
  name and finds them through `TEXINPUTS`.
- With MiKTeX, missing LaTeX packages are installed without a dialog
  (`--enable-installer`); before, pdflatex waited on it until the time limit.
- When pdflatex cannot open the `.tex`, its `texput.log` is kept as
  `last-error.log`.
- A local SVG, WEBP or GIF image no longer stops the build: like a remote one,
  it becomes its caption with a warning (pdflatex cannot typeset them).
- A web address that returns a page instead of an image (for example a
  Wikipedia `File:` page) is no longer saved as a `.png` in `cache/remote/`,
  where it broke every later build. The format is read from the downloaded
  bytes, and a bad file left by an older version is downloaded again.
- In the menu, choosing another profile clears the cover data the new profile
  leaves empty (teacher, group, custom fields) and resets the output folder.
  Before, the previous profile's values stayed and ended up on the cover.
- The profile wizard no longer overwrites an existing profile when a new one
  gets the same name (`IA` and `ia` are both `courses/ia.toml`); it asks for
  another name.
- A list nested more than 4 levels deep no longer stops the build with "Too
  deeply nested": the deeper levels are moved up to the fourth, with a warning.
- With a Pandoc older than 3.0 (what `apt` installs on Ubuntu 22.04 and
  Debian 12), a failed conversion now says that Pandoc 3.0 is needed instead
  of showing a Lua error. `INSTALL.md` says so too.
- An image path with spaces written as `%20` (`img/mi%20foto.png`, as many
  editors save it) is found instead of being reported as missing.
- In the profile wizard, Esc leaves from the first step at once; from the
  second step it first warns "Press Esc again to leave without saving.", so a
  stray key does not lose what you typed.
- If a profile cannot be saved (no permission, full disk, a removed USB), the
  wizard stays open on its last step with the error and keeps what you typed.
- Picking a profile in the form now uses its output folder as written, even
  outside the work folder, so the menu and the command line save the PDF in
  the same place.
- A failed extra copy (a file in the way, no permission, a disconnected USB) is
  a warning: the PDF is generated, the other copies are made and the command
  ends with exit code 0.
- Changing the work folder (Options → `w`) copies `input/`, `courses/`,
  `my-templates/` and `logos/` from the current work folder, not from the
  program folder, so profiles no longer seem to disappear.
- A new work folder inside `input/`, `courses/` or `my-templates/` is no longer
  copied into itself without end.
- Profiles, `settings.toml`, the PDF and its copies are written to a hidden
  `.<name>.tmp` in the same folder and then renamed, so a power cut or a full
  disk no longer leaves an empty or half-written file.
- Searching the Markdown by name in `input/` and copying to a new work folder
  no longer follow symbolic links to folders (a link to a parent folder
  crashed the program); linked files are still found and copied.

### Changed

- Fewer libraries are compiled (104 → 86 crates): `thiserror` is gone, `clap`
  is built without colors and suggestions, and `crossterm` comes from
  `ratatui`.
- The Spanish cover date is written as Spanish does it, "8 de octubre de
  2026", instead of "Octubre 8, 2026". This changes the cover of every
  Spanish paper.

### Added

- **Work folder.** The first time, the menu asks where your papers should live
  and creates `input/`, `output/`, `courses/`, `my-templates/` and `logos/`
  there, copying what the program folder already had (nothing is
  overwritten). You no longer need to open the program folder. Saved as
  `CARPETA_TRABAJO`; change it in Options → `w`. Without one, everything stays
  in the program folder as before.
- `install.sh` (Linux) and `install.ps1` (Windows) check the tools, run
  `cargo install --path . --locked` and delete the intermediate build files.
- In the menu, the output folder and the extra copies are chosen with the same
  folder browser as the Markdown (`./` picks the folder you are in). The
  profile wizard asks for both too, and profiles gain the keys `output` and
  `copies` (`--copy` replaces the profile's copies).
- Color themes for the menu (Options → `t`): Classic, Dracula, Nord, Gruvbox,
  Catppuccin Mocha and Tokyo Night. The choice is saved as `TEMA` in
  `settings.toml`.
- CI on GitHub Actions: format, lint and tests on Linux (with real PDFs) and
  Windows.
- The profile wizard's "Papers folder" step is now a folder picker inside
  `input/`: `./` chooses the folder and it is saved relative to `input/`.
- Every folder picker (output folder, extra copies, work folder and the
  wizard's papers folder) can create a folder: type a new name and choose
  `+ <name>/  create and open`.
- On the root of a drive, the folder pickers list the other drives that exist,
  so you can switch to a USB drive without typing its path.

## 2.0.0 (2026-10-04)

The program was rewritten in Rust. It gained a full-screen menu, course
profiles with a wizard, separate formats (APA 7, Harvard, MLA 9) and cover
designs, a bilingual interface, a diagnostic log and documentation for
people and AI assistants.

**Supported platform: Linux.** Windows and macOS will come in a later update.

### Added

- **Interactive menu** (`investigacion` with no arguments). Files, profiles,
  formats and designs are chosen from lists, so no paths are typed.
- **Bilingual interface (Spanish/English).** The menu asks the first time,
  using the system language as the default, and remembers it in
  `settings.toml`. `l` in the options switches it, and `--lang` overrides it
  for one command.
- **Course profiles** in `courses/<key>.toml` with all the cover data, the
  format, the design and the course folder (`-p ia`).
- **`--file-name`**: the PDF is named after the Markdown file, independent of
  the title.
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
- **No more `.env`.** All the cover data (university, faculty, student,
  semester, teacher, group, members) comes from the profile; the wizard asks
  for all of it. Without a profile, give it with `--set UNIVERSIDAD=…`
  (also `FACULTAD`, `ALUMNO`, `SEMESTRE`, `DOCENTE`, `GRUPO`, `INTEGRANTES`).
  The menu language is now remembered in `settings.toml` (with an optional
  `LOGOS`), and `--env-file` became `--settings` (the old name still works).
  `.env.example` was removed; the catalog uses `-p example`.

- **Documentation reorganized.** `README.md` is now a short introduction.
  `GUIDE.md` became `docs/reference.md` (the complete reference), plus
  `docs/usage.md` (basic use of the menu and the CLI) and `docs/cli.md` (every
  option). `AI-PROMPT.md` became `docs/prompts/paper.md`, and the new
  `docs/prompts/design.md` turns a cover idea into a compatible design.
  `llms.txt` indexes it all for AI assistants.
- The menu form has **University, Faculty, Student and Semester** fields. The
  profile fills them, and the design decides which are required, so the menu
  also works without a profile.
- `--check-template` also accepts `--check-template=<design>`, and no longer
  takes another option (`--lang`) as the design name.
- `Obras citadas`, `Fuentes consultadas` and `Fuentes` are recognized as
  reference headings (hanging indent), for MLA in Spanish.

### Fixed

- The placeholder profile `example.toml` counted as a profile, so a new user
  never got the wizard on the first run. It is no longer listed in the menu;
  `-p example` still works.
- If the project folder cannot be found (for example, the clone was moved
  after `cargo install`), the program says so and how to fix it, instead of
  reporting "no profiles" or "no designs".
- Arrows (`->`, `<->`) in ` ```tikz ` blocks and designs broke the build in
  Spanish documents: the base now loads TikZ's `babel` library.
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

### Removed

- The Python package, `pyproject.toml` and the `investigacion-tui` command.
  `investigacion` alone opens the menu.
- `.env`, "allow LaTeX", template path and logos from the menu. They remain
  available on the command line.

### Upgrading from the `mejoras-rust` preview

Copy what your `.env` had (`UNIVERSIDAD`, `FACULTAD`, `ALUMNO`, `SEMESTRE`…)
into each profile as `university`, `faculty`, `student`, `semester`…, or open
each profile with `p` in the menu's options and fill it in. Then delete `.env`;
the menu asks for the language again once and saves it in `settings.toml`.

### Upgrading from 0.1 (Python)

1. Install Rust and run `cargo install --path .` (see [INSTALL.md](INSTALL.md)).
2. Run `deactivate` if the old environment is active, then delete `.venv/`.
3. Move your logos from `Latex/logos/` to `templates/logos/`. The old
   `imagenes/` cache can be deleted.
4. Optionally create course profiles in `courses/` from `courses/example.toml`.
