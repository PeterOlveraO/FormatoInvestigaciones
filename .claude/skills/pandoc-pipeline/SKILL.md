---
name: pandoc-pipeline
description: How Markdown is read, cleaned, found and converted, including supported syntax, Pandoc extensions, the Lua filters, the image/diagram cache, invisible-space normalization, Markdown lookup in input/ and raw_tex. Use when editing src/markdown.rs, src/pandoc.rs, src/encoding.rs or resources/filters/**, adding Markdown syntax, or debugging content that is lost or misformatted in the PDF.
paths: "src/markdown.rs,src/pandoc.rs,src/encoding.rs,resources/filters/**,tests/pandoc.rs,examples/**"
---

# Markdown → LaTeX pipeline

## Where syntax support lives

The goal is the whole [Markdown Guide](https://www.markdownguide.org) (basic
and extended). Check which of three places to change:

1. **`MARKDOWN_EXTENSIONS`** (`src/pandoc.rs`): `mark` (`==x==`), `emoji`
   (`:joy:`), `autolink_bare_uris`.
2. **Lua filters** (`resources/filters/`, applied in `LUA_FILTERS` order).
   - **How they ship:** embedded with `include_str!` and written to a temp dir
     on each run.
   - **What they see:** the parsed tree, never the inside of code blocks.
   - **The filters:**
     - `inline_html`: `<br> <mark> <sub> <u> <kbd> <img>`… The LaTeX writer
       **drops HTML silently**, so without this filter the content vanishes.
     - `images`: downloads web images and warns about missing local ones.
     - `diagrams`: draws ` ```dot ` blocks with Graphviz; without Graphviz the
       block stays as code and a warning is printed.
     - `charts`: wraps ` ```pgfplot `/` ```tikz ` in `tikzpicture`, plus a
       figure when there is a caption. Real LaTeX, so errors stop the build.
     - `blocks`: `::: nota`/`::: note` boxes and the references section. It
       emits `CajaMarcada` (box title in the **document** language, from
       `INVESTIGACION_DOC_LANG`) and `ReferenceList`. The latter is defined
       neutral in the base and redefined by each format. Reference headings
       are recognized in Spanish and English (`referencias`, `obras citadas`,
       `references`, `bibliography`, `works cited`…).
3. **`templates/common/investigacion-base.sty`:** the commands Pandoc assumes
   (see the `latex-templates` skill).

**When the accepted syntax changes, update `docs/prompts/paper.md`, `docs/reference.md` and
`examples/catalog.md`.**

To see what Pandoc produces:

```bash
pandoc file.md --from=markdown-raw_tex+mark+emoji+autolink_bare_uris \
  --to=latex --wrap=none --lua-filter=resources/filters/inline_html.lua
```

## Filter conventions

- **Warnings.** Write them to stderr as `[investigacion] <message>`.
  `filter_warnings()` picks them out and they reach the same `on_warning`
  callback as the LaTeX warnings, and so the log. Pandoc's other stderr
  lines are not shown; they are logged at debug level
  (`INVESTIGACION_LOG=debug`).
- **Two languages, two variables.**
  - **Warnings** to the user follow the interface language: use
    `texto(es, en)` driven by `INVESTIGACION_LANG`.
  - **Text that goes into the PDF** (box titles) follows the document
    language: `INVESTIGACION_DOC_LANG`.
  - `pandoc_to_latex()` sets both; the document language comes from
    `Layout::document_language()`.
- **Never fail.** On a problem, replace the image with its alt text (or leave
  the diagram as code) and warn.
- **Paths from Rust.** `INVESTIGACION_REMOTE_IMAGES`, `INVESTIGACION_DIAGRAMS`
  and `INVESTIGACION_RESOURCES` (one folder per line) arrive through
  `pandoc::posix()`, with `/` even on Windows. The filters use them only to
  read and write files: `\includegraphics` gets just the cache file name, and
  pdflatex finds it through `TEXINPUTS` (`pandoc::resource_dirs()`). A full
  path there breaks on spaces, the `~` of Windows short paths and accents
  (`os.getenv` is not UTF-8 on Windows).

## Image and diagram cache

- **Folders:** `Project::media_directories()` creates `cache/remote/` and
  `cache/diagrams/`.
- **File names:** the `sha1` of the URL or the diagram code. The cache is safe
  to delete.
- **Formats:** pdflatex only handles PNG/JPG/PDF; anything else is rejected
  with a warning.
- **`User-Agent`:** Pandoc sends none; `--request-header` adds one because
  some sites answer 400.
- **Lookup paths:** `--resource-path` includes the Markdown's folder and
  `cache/`.

## Reading and cleaning the Markdown

- **Encoding.** `read_markdown()` = `decode_text()` (UTF-8 with or without BOM,
  then strict Windows-1252) + `normalize_markdown()`.
- **The bug it fixes.** Text pasted from Word, Notion or an AI has
  non-breaking spaces on "empty" lines and two trailing spaces. Pandoc then
  merges everything into one paragraph (`###` printed, tables as rows of bars),
  with **no error**.
- **What `normalize_markdown()` does:**
  - trims hard, figure, narrow and zero-width spaces at line ends;
  - repairs markers separated by a hard space (`###`, bullets, numbered lists,
    `>`).
- **Trailing double spaces** are both garbage and syntax (a line break).
  `has_disguised_spaces()` decides: if any line ends in a disguised space,
  everything is trimmed; otherwise line breaks are kept.
- **What it never touches:** hard spaces inside text, and fenced ``` / ~~~
  blocks.

## Finding the Markdown

`find_markdown()` tries, in order:

1. the path as given;
2. the path inside `input/`;
3. for a bare name, a recursive search under `input/`.

With a course profile, `locate_markdown()` searches `input/<folder>` first.

- **An existing path always wins.**
- **Two files with the same name are an error** listing their subfolders,
  never an arbitrary pick.

## raw_tex and validation

- **raw_tex is off by default.** Pandoc runs with `markdown-raw_tex` unless
  `--allow-latex` is given, so a path like `C:\Users\...` prints literally
  instead of breaking LaTeX. `$...$` math works in both modes.
- **Structure warnings.** `validate_markdown(markdown, headings)` warns about
  missing or out-of-order headings from the format's `format.toml` `headings`.
  Only `apa7` defines them; no headings means no warnings.
  `markdown_headings()` ignores fenced code.
