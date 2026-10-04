---
name: bilingual-interface
description: Rules and checklists for everything the user sees, including the Spanish/English language system (tr!, Text, resolution, saving IDIOMA), the localized clap help and errors, the full-screen TUI (state, picker, keys), course profiles and settings. Use when adding or changing a message, a CLI option, a TUI field or key, a course-profile key, or anything in src/i18n.rs, src/cli.rs, src/tui/**, src/courses.rs or src/settings.rs.
paths: "src/i18n.rs,src/cli.rs,src/tui/**,src/courses.rs,src/settings.rs,src/main.rs,tests/cli.rs"
---

# Bilingual interface, CLI and TUI

## Language system (`src/i18n.rs`)

- **Order:** `--lang es|en` > `IDIOMA`/`INTERFACE_LANGUAGE` (environment or
  `.env`) > `detect()` via `sys-locale`. `es*` gives Spanish; anything else,
  English.
- **Per-thread language** (`thread_local!`). Tests set their own with
  `i18n::set(Lang::Es|En)` without racing. The TUI copies the language into its
  worker thread before calling `execute()`; any new thread that produces
  messages must do the same.
- **Messages with arguments:** `tr!(es: "…{x}…", en: "…{x}…", args)`. Both
  versions are required by the macro.
- **Constant texts:** `Text::new(es, en)` plus `.get()` (it also implements
  `Display`).
- **Never pass clap help templates through `format!`/`tr!`.** Their
  `{usage}`/`{options}` placeholders would be consumed; use `Text`.
- **Spanish with accents.** Messages describe the action from the user's point
  of view and never mention internal names.
- **Lua filters** get the language through `INVESTIGACION_LANG` (see the
  `pandoc-pipeline` skill).

## CLI (`src/cli.rs`)

- **Option names** are English. Old names are hidden `alias`/`aliases`
  (`--titulo`, `--materia`, `--subject`, `--perfil`, `--template` for
  `--design`…); never remove them.
- **Layout and data options:**
  - `--format`, `--design`;
  - `--set NAME=value` (repeatable, `parse_field`, the name is uppercased);
  - `--doc-lang` (document language, separate from `--lang`).
- **`--check-template <design>`** is handled in `main_with_args` **before**
  clap parsing (it needs no Markdown or title) and runs `check::run_cli`.
- **Help text** lives in `ARG_HELP` (`(id, Text help, Option<Text> value
  name)`), applied by `localized_command()` with `mut_arg`. It provides custom
  `-h/--help` and `-V/--version` and a `help_template` with translated
  headings.
- **Language before parsing.** `prescan_language()` reads `--lang` and
  `--env-file` before parsing, so `--help` and argument errors are already in
  the right language.
- **Argument errors.** `describe_clap_error()` translates
  `MissingRequiredArgument`, `UnknownArgument` (adding the "put values with
  spaces in quotes" hint when the stray argument has no leading `-`),
  `InvalidValue` and `ValueValidation`. Other kinds use clap's message.
- **Running a generation.** `execute()`:
  1. loads `.env` and the profile, then `resolve_layout` (design + format);
  2. builds `DocumentData` with `pick()` (option > profile > `.env`), all
     cover data included;
  3. checks `missing_data()` before reading the Markdown;
  4. validates with the format's headings and reports through `Reporter`.

### Checklist: adding a CLI option

1. Add a field to `Args` (English name; `alias` if there is a natural Spanish
   one).
2. Add an `ARG_HELP` entry with `es` and `en` help and a value name. `mut_arg`
   panics on an unknown id, and the help tests catch it.
3. Use it in `execute()`. If it should fall back to `.env` or a profile, go
   through `pick()`.
4. If the menu needs it: add a `FIELDS` entry (`label`, `help`, `example`,
   `empty`, all `Text`) and map it in `App::build_args()`.
5. Add tests (`src/cli.rs` tests and `tests/cli.rs`).
6. Update `GUIDE.md` (options table), `CHANGELOG.md` and, if it applies,
   `README.md`.

## TUI (`src/tui/`)

ratatui + crossterm. It duplicates no logic: `App::build_args()` builds
`cli::Args` (empty fields become `None`, so profile and `.env` still apply),
and a thread runs `cli::execute()` with a `ChannelReporter`.

- **State and drawing are separate.** `app.rs` handles keys and state and
  **never draws**; `ui.rs` only draws.
- **Tests** go in `src/tui/tests.rs`, with `press()`/`type_text()` and
  `render()` (`TestBackend`, 120×32).
- **Modes:** `ChooseLanguage` → `Wizard` → `Home` ↔ `Form` ↔ `Editing` /
  `Picking` / `Options` → `Generating`.
  - `Home(i)` and `Form` are full screens (`Screen`); the others are windows
    drawn over one. `App::base` remembers which one opened the window and
    `back()` returns to it (Esc in options, wizard cancel or done, picker).
  - `ui::draw` draws `app.screen()` first, then the window for the mode.
- **Start** (`App::start`):
  - `ChooseLanguage` when no `IDIOMA` is saved, with the system language
    preselected. Enter saves it with `Settings::save_value()`; Esc uses it
    without saving. Arrows only move the mark.
  - Then the profile wizard if `courses/` has no profiles
    (`pending_wizard`).
  - Then `Home(0)`. With a saved language and profiles, straight to home.
  - Tests that call `App::new()` without `start()` begin in `Form`.
- **Home view:** `HOME_ITEMS` (Generate a PDF → form, New profile → empty
  wizard, Options, Quit) plus a status panel: profiles, the session's last
  PDF, external tools and the log.
- **Tool check** (`tools.rs`): `App::detect_tools()` (called in `tui::run()`,
  never in `App::new`) runs `pandoc --version`, `pdflatex --version` and
  `dot -V` in a background thread; `poll_tools()` collects the result.
  Not found = `RunError::NotFound`; a timeout counts as installed.
- **Options view:** one const table, `OPTIONS` (`keys: [es, en]`, `Text`
  label, `OptionAction`), drives both `app.rs` and `ui.rs`. To add a row:
  append an entry and handle its action in `App::run_option()`. The letters
  also work, hidden, in the form (`p`, `n`, `c`/`f`, `l`, `r` log,
  `i`/`h`); the
  form's own keys win on a clash.
- **Profile wizard** (`wizard.rs`; New profile on home, `p`/`n` in the
  options):
  - steps: key → format → design (filtered by `accepts`) → one step per marker
    the design uses (`STANDARD_STEPS` order) plus its custom fields → folder;
  - required data follows the same rules as `missing_data`;
  - Shift+Tab goes back;
  - `courses::save_profile` writes the TOML (only non-empty keys);
  - each profile holds **all** its data.
- **Form:**
  - `FIELDS` are the fixed fields (including Format and Design pickers; the
    design list is filtered by the chosen format);
  - `App::extras` are the design's custom fields, appended after `FIELDS` and
    refreshed by `refresh_design()`;
  - use `total_fields()`, `field_label()`, `field_value()` and
    `is_required()` instead of indexing `FIELDS`. The course is required only
    if the design uses `MATERIA`.
- **Keys work in both languages.**
  - Home: ↑↓, Enter, `s`/`q` quit (`v` view PDF, hidden).
  - Form: `g`/F5 generate, `v` view PDF, `o` options, Esc home, `s`/`q`
    quit, digits jump to a field.
  - Options: ↑↓ (wrap around), Enter or the row's letter, Esc back.
  - The form footer only shows move, edit/choose, clear, generate, view
    PDF, options and quit; each language shows its own letters (`s`/`q`,
    `c`/`f`, `i`/`h`). The language row starts with the *other* language's
    name.
- **One picker for everything.** `picker.rs` serves the Markdown (folder
  browsing, `.md` only), profiles, templates and project folders. The filter
  ranks an exact name, then a prefix, then a substring, then the description;
  without that, `ia` picked the profile `example` ("mater**ia**").
- **Windows** also sends key-release events; keep the
  `KeyEventKind::Press` filter.
- **Never crash the menu** (`guard.rs`).
  - `event_loop` runs `handle_key` and `ui::draw` inside `guard::run`, and
    the worker runs `execute()` inside it too. A panic there becomes
    `App::recover()`: an error line, and the window closes back to its base
    screen.
  - Two failed draws in a row end the menu cleanly, with a pointer to the
    log.
  - `guard::install_hook()` goes **after** `ratatui::init()`. Inside
    `guard::run` it only logs the panic; restoring the terminal there would
    break the running menu. Outside it, the normal chain runs: ratatui
    restores, `logging` writes, Rust prints.
- **Logging in the TUI.** `main.rs` calls `logging::init()` and
  `logging::install_panic_hook()` before `tui::run()`.
  - Everything through `cli::execute()` is already logged.
  - `handle_key` logs each key and mode change at debug level
    (`Mode::name()`).
  - Log TUI-only events yourself in English: opened paths and `opener`
    errors, picker choices, tool detection, a worker without a result.
  - Options → `r` opens `logging::path()`.
- **Not in the menu on purpose:** `.env` path, `--allow-latex`, template by
  path, `--logos` (CLI only).
- **The PDF name** is proposed from the chosen Markdown until the user types
  one (`file_name_is_auto`).

## Course profiles and settings

- **`courses/<key>.toml`** (`CourseProfile`) holds:
  - `name` (required; alias `subject`);
  - `university`, `faculty`, `student`, `semester`, `members`, `teacher`,
    `group`;
  - `format` and `design` (alias `template`);
  - `language`, `folder`, and `[fields]` for custom fields.
- **`deny_unknown_fields`:** a typo is an error, not silently ignored. The
  `toml` crate's detail message stays English.
- **`folder`:** the Markdown is looked up in `input/<folder>` first, and the
  output goes to `output/<folder>`.
- **Adding a profile key:** add it to `CourseProfile`, `save_profile`, `pick()`
  in `execute()`, the wizard if it is cover data, and `App::apply_profile()`.
  Document it in `courses/README.md`, `courses/example.toml` and `GUIDE.md`.
- **`Settings::get(&[names])`:** the first non-empty value, with the
  environment first and `.env` second. Spanish and English key names are both
  accepted. It never mutates the process environment.
- **`Settings::save_value`:** replaces the key's line (with or without
  `export`) or appends it, keeping all other lines and comments.
