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
  (`--titulo`, `--materia`, `--subject`, `--perfil`…); never remove them.
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
- **Running a generation.** `execute()` resolves `.env`, the profile and the
  Markdown, builds `DocumentData` with `pick()` (option > profile > `.env`) and
  reports through `Reporter`.

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
- **Modes:** `ChooseLanguage` → `Form` ↔ `Editing` / `Picking` → `Generating`.
- **First run.** `tui::run()` opens `ChooseLanguage` when no `IDIOMA` is saved,
  with the system language preselected. Enter saves it with
  `Settings::save_value()`; Esc uses it without saving.
- **Keys work in both languages:** `g`/F5 generate, `v` view PDF, `c`/`f`
  folders, `l` toggle language (and save), `s`/`q`/Esc quit, digits jump to a
  field. Each language's footer shows its own letters, and the `l` entry shows
  the *other* language's name.
- **One picker for everything.** `picker.rs` serves the Markdown (folder
  browsing, `.md` only), profiles, templates and project folders. The filter
  ranks an exact name, then a prefix, then a substring, then the description;
  without that, `ia` picked the profile `example` ("mater**ia**").
- **Windows** also sends key-release events; keep the
  `KeyEventKind::Press` filter.
- **Not in the menu on purpose:** `.env` path, `--allow-latex`, template by
  path, `--logos` (CLI only).
- **The PDF name** is proposed from the chosen Markdown until the user types
  one (`file_name_is_auto`).

## Course profiles and settings

- **`courses/<key>.toml`** (`CourseProfile`) holds `name` (required; alias
  `subject`), `teacher`, `group`, `members`, `template` and `folder`.
- **`deny_unknown_fields`:** a typo is an error, not silently ignored. The
  `toml` crate's detail message stays English.
- **`folder`:** the Markdown is looked up in `input/<folder>` first, and the
  output goes to `output/<folder>`.
- **Adding a profile key:** add it to `CourseProfile`, use it through `pick()`,
  fill it in `App::apply_profile()`, and document it in `courses/README.md`,
  `courses/example.toml` and `GUIDE.md`.
- **`Settings::get(&[names])`:** the first non-empty value, with the
  environment first and `.env` second. Spanish and English key names are both
  accepted. It never mutates the process environment.
- **`Settings::save_value`:** replaces the key's line (with or without
  `export`) or appends it, keeping all other lines and comments.
