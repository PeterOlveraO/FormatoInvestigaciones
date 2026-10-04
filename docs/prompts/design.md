# Prompt: have an AI create a cover design

This file holds a ready-to-paste prompt. It makes an AI produce a LaTeX
**design** (the cover and the look of the first pages) that fits the
program: two files, `template.ltx` and `template.toml`.

1. Copy everything inside the big block below and paste it into ChatGPT,
   Claude, Gemini or the model you use.
2. After it, describe the cover you want, or attach an image or a PDF of it.
   Say which data it must show (title, student or team, course, teacher,
   university, faculty, group, semester, date, and any extra such as the
   room) and the colors, if you have them (hex codes are best).
3. Create the folder `my-templates/designs/<name>/` in the project (git
   ignores it) and save the two files the AI gives you inside it.
4. Validate it. This builds the example catalog with the design in every
   format it accepts:

```bash
investigacion --check-template <name>
```

5. Use it: `investigacion paper.md -p course --design <name>`, or put
   `design = "<name>"` in the course profile. It also appears in the menu.

If something fails, paste the output of `--check-template` (or the
`last-error.log` that is saved next to the PDF when you build a real paper)
back to the AI and ask it to fix the design.

---

## The prompt

````````text
You are going to create a LaTeX "design" for a program called `investigacion`,
which turns a paper written in Markdown into a PDF (Pandoc, then pdflatex).
A design is the cover (or the title block) and the look of the first pages.
It is made of TWO files, which you must deliver:

- `template.ltx`: a LaTeX document with placeholders.
- `template.toml`: a small manifest (what the design is called, which formats
  it accepts, which extra data it asks for).

The user will describe the cover they want (in words, or with an image or a
PDF). Reproduce it as faithfully as possible within the rules below. Do not
ask questions: if something is missing, decide it and mention the decision in
one line after the files. Write the short comments inside the files in
Spanish (or in the language the user writes in).

# 1. HOW THE SYSTEM IS BUILT

There are three layers and the design is only the last one:

1. BASE (`investigacion-base.sty` and `investigacion-final.sty`): universal.
   It loads everything the paper's content needs (tables, code, math,
   diagrams, charts, symbols, emoji, hyperlinks). The design must load it.
2. FORMAT (APA 7, Harvard or MLA 9, chosen by the user): the norm. It sets the
   paper size, margins, fonts, line spacing, headings, captions, page
   numbers and the references style.
3. DESIGN (your work): the cover or title block, its colors and its layout,
   plus its own data fields.

The program fills the placeholders `%%NAME%%` of `template.ltx` with the
user's data (already escaped for LaTeX), inserts the language and the format
where `%%FORMAT%%` is, and puts the converted paper where
`%%CONTENIDO_MARKDOWN%%` is. Then it runs pdflatex (several passes, so
TikZ overlays and the table of contents work).

# 2. THE CONTRACT OF `template.ltx` (mandatory, in this order)

```latex
\documentclass[%%CLASS_OPTIONS%%]{article}
\usepackage{investigacion-base}
%%FORMAT%%

% ... your \newcommand data, colors and helper commands ...

\usepackage{investigacion-final}

\begin{document}

% ... your cover or title block ...

\FormatBodySetup
% ... optionally \tableofcontents and \newpage ...

%%CONTENIDO_MARKDOWN%%

\end{document}
```

Rules, all of them checked or required:

1. `\documentclass[%%CLASS_OPTIONS%%]{article}` exactly. The format fills
   the font size and the paper (for example `12pt, letterpaper`). Never
   write another class or fixed options.
2. `\usepackage{investigacion-base}` is the first package line.
3. `%%FORMAT%%` goes alone on its own line, right after the base. The program
   replaces it with two lines (`\usepackage[<babel options>]{babel}` and
   `\usepackage{investigacion-format}`). **Never write `%%FORMAT%%` (or any
   other marker you do not mean to use) inside a comment**: the replacement
   is made on the whole text, and the second inserted line would no longer be
   commented out.
4. Your data commands (`\newcommand{\X}{%%MARKER%%}`) and any extra
   `\usepackage` come AFTER `%%FORMAT%%`.
5. `\usepackage{investigacion-final}` is the LAST thing of the preamble,
   right before `\begin{document}`. It loads hyperref, footnotes in tables and
   the PDF metadata. Nothing may be loaded after it.
6. After the cover call `\FormatBodySetup` (the format sets the alignment and
   indentation of the body there).
7. `%%CONTENIDO_MARKDOWN%%` appears exactly once, alone on its line, inside
   the document, after the cover. Never inside a comment.
8. The file is UTF-8. Accents and `ñ` can be typed directly.

# 3. THE MARKERS

A marker is `%%NAME%%`: only capital letters A-Z and underscores (no digits,
no accents, no lowercase). The program replaces it with the value, already
escaped (`&`, `%`, `_`, `#`, `$`, `{`, `}`, `~`, `^`, `\` are safe): never
escape it again, and do not use a value inside `\url`, `\hypersetup` or
`\pdfbookmark`.

| Marker | Meaning | Notes |
|---|---|---|
| `%%TITULO%%` | Title of the paper | Required |
| `%%UNIVERSIDAD%%` | University | Required (unless listed in `optional`) |
| `%%FACULTAD%%` | Faculty or school | Required (unless in `optional`) |
| `%%MATERIA%%` | Course name | Required (unless in `optional`) |
| `%%SEMESTRE%%` | Term, for example `2026-2` | Required (unless in `optional`) |
| `%%ALUMNO%%` | Student name | May be empty |
| `%%INTEGRANTES%%` | Team members, one name per line, joined by `\\` | May be empty |
| `%%DOCENTE%%` | Teacher | May be empty |
| `%%GRUPO%%` | Group, for example `7-A` | May be empty |
| `%%FECHA_ENTREGA%%` | Delivery date, already in the document language | Always filled |
| `%%CLASS_OPTIONS%%` | See section 2 | Only in `\documentclass` |
| `%%FORMAT%%` | See section 2 | Once, alone on its line |
| `%%CONTENIDO_MARKDOWN%%` | The paper | Once |

Important behavior:

- The program asks the user only for the data whose marker appears in your
  file. A standard marker you do not write is simply not requested, so do not
  write markers you will not show.
- `TITULO`, `UNIVERSIDAD`, `FACULTAD`, `MATERIA` and `SEMESTRE` are required
  when used. If the design must work without one of them, list it in the
  `optional` array of `template.toml` AND hide its line when empty.
- The others (`ALUMNO`, `INTEGRANTES`, `DOCENTE`, `GRUPO`) can arrive empty
  and are never required: always guard them.
- Guard every optional value with `\ifdefempty{\Command}{}{...}` after
  storing it in a command (`etoolbox` is already loaded). A value that is
  empty must not leave a blank line or a stray label.
- Team or student: the members replace the student. Use
  `\ifdefempty{\ListaIntegrantes}{ student line }{ members block }`. Put the
  members in `\begin{tabular}[t]{@{}l@{}}\ListaIntegrantes\end{tabular}` (or
  directly in a TikZ `node` with `align=left`) so each `\\` starts a new
  line.
- Order of the data in the built-in covers: student or team, course,
  teacher, semester, group. Follow it unless the user's cover says otherwise.

## Custom fields

Any other `%%NAME%%` (for example `%%SALON%%`, `%%LINEA_DE_INVESTIGACION%%`)
is a custom field. The program asks for it in the menu, saves it in the
course profile (`[fields]`) or takes it from `--set NAME=value`. It arrives
empty if nobody gave it. Describe each custom field in `template.toml`, and
always store it in a command and guard it with `\ifdefempty`:

```latex
\newcommand{\Salon}{%%SALON%%}
...
\ifdefempty{\Salon}{}{\textbf{Salón:} \Salon\par}
```

A name that looks like a standard one but is slightly different (`DOCENTES`,
`TITLO`) produces a warning suggesting the standard marker.

# 4. THE MANIFEST `template.toml`

All keys are optional, but write the file. Unknown keys are an error. Texts
can be a plain string or bilingual `{ es = "…", en = "…" }`; always write
both languages.

```toml
description = { es = "Portada con banda de color", en = "Cover with a color band" }
# Formats it works with; the first one is the default. Built-in ones:
# "apa7", "harvard", "mla". Empty = any format. Use all three unless the design
# cannot work with one of them.
formats = ["apa7", "harvard", "mla"]
# Standard markers that the design uses but does not require.
optional = ["FACULTAD"]

# One block per custom field used in template.ltx (%%SALON%% -> SALON).
[fields.SALON]
label = { es = "Salón", en = "Room" }
help = { es = "Aula donde se entrega el trabajo", en = "Room where the paper is handed in" }
required = false
```

- `required = true` makes the program refuse to build until the field is
  filled. Use it only when the cover is meaningless without it.
- Every `[fields.X]` must have a matching `%%X%%` in the file, and the other
  way round (otherwise `--check-template` warns).
- `optional` may only mention markers that the file really uses.

# 5. WHAT THE DESIGN OWNS AND WHAT IT MUST NOT TOUCH

The design owns: the cover or title block, its colors, its decorations, the
position of every item, and whether there is a table of contents.

The design must NOT change (the format owns them, and the same design has to
look right with APA 7, Harvard and MLA):

- Fonts of the body, font packages (`newtxtext`, `lmodern`, `helvet`,
  `fontspec`...), line spacing (`setspace`), paragraph indentation.
- Page geometry and margins (`geometry`, `\newgeometry`), paper size.
- Heading styles (`titlesec`), numbering, captions (`caption`), page style and
  page numbers in the body (`fancyhdr`, `\pagestyle`), the references list.
- Never load the `float` package (it conflicts with `floatrow`, which the base
  loads). Never load `hyperref`, `bookmark`, `xurl`, `footnote` or
  `footnotehyper` (the final package does it), and never load `babel`
  yourself (the format block does it, and a second load is an option clash).
- Never remove or redefine what the base provides for Pandoc
  (`\tightlist`, `CajaMarcada`, `ReferenceList`, `Shaded`, `longtable`,
  `\pandocbounded`...).

Inside the cover you are free to use font sizes (`\Huge`, `\large`,
`\fontsize{..}{..}\selectfont`), families (`\sffamily`, `\ttfamily`),
`\bfseries`, `\itshape`, `\scshape`, `\textcolor`, `\colorbox`, tables, TikZ.
Open the font of the cover only with groups `{ ... }` so nothing leaks into
the body.

## Packages already loaded (do not load again, do not pass options to them)

- From the base: `inputenc` (utf8), `fontenc` (T1), `textcomp`, `setspace`,
  `amsmath`, `amssymb`, `xcolor`, `tikz` (with NO libraries), `pgfplots`,
  `pgfplotstable`, `pgf-pie`, `graphicx`, `longtable`, `booktabs`, `array`,
  `calc`, `caption`, `etoolbox`, `fancyvrb`, `framed`, `soul`, `ulem`,
  `upquote`, `pmboxdraw`, `twemojis`, `newunicodechar`, `floatrow`.
- From the format: `babel`, `geometry`, `titlesec`, `ragged2e`, `fancyhdr`,
  `newtxtext`, `newtxmath` (APA also `indentfirst`).
- From the final package: `hyperref`, `bookmark`, `xurl`, `footnotehyper`.

Safe extras (put them after `%%FORMAT%%`, before `investigacion-final`):
`multicol`, `tcolorbox` (with its own options, for example `[most]`),
`lipsum`, `eso-pic`, `tabularx`, `makecell`, `multirow` (all tested). For extra colors do not reload `xcolor`:
use `\definecolor{Name}{HTML}{1F3A5F}`. For TikZ libraries use
`\usetikzlibrary{...}` in the preamble (tested: `positioning`, `calc`,
`shapes.geometric`, `arrows.meta`, `decorations.pathmorphing`). The base already
loads the `babel` library, so arrows such as `->` work in every language.

# 6. HOW TO BUILD A ROBUST COVER

- **Cover page.** Use `\begin{titlepage} ... \end{titlepage}`. The
  environment restarts the page counter, and the built-in designs count the
  cover as page 1, so right after it write `\setcounter{page}{2}`. Use
  `\thispagestyle{empty}` for no page number on the cover (or
  `\thispagestyle{fancy}` to show the format's one). Inside the cover use
  `\singlespacing` (the format may double-space the document).
- **A design without a cover** (a compact title block, like the built-in
  `report`) is also fine: skip `titlepage`, print the block, then
  `\FormatBodySetup` and the content.
- **Table of contents.** Optional: `\tableofcontents` followed by `\newpage`
  after `\FormatBodySetup`. The format translates its title.
- **Full-page decoration** (bands, frames, corners, backgrounds): draw it with
  TikZ in a `\begin{tikzpicture}[remember picture, overlay]` and position it
  relative to `current page.north west`, `north east`, `south west`,
  `south east` or `center`. Use `\paperwidth` and `\paperheight` instead of
  fixed sizes (the paper is Letter, but the format may change it). The overlay
  works without any library; pdflatex runs several passes.
- **Putting text below a decoration.** The tikzpicture that opens a paragraph
  makes the following `\vspace` misbehave: put `\vspace*{..}` BEFORE the
  tikzpicture, as the first line of the cover, so what follows lands under
  the decoration. A `\vspace` needs a `\par` before it if the previous
  line is text or an image.
- **Titles.** They can be long (up to about 150 characters) and wrap on 3-4
  lines. Put the title in a TikZ `node` with `text width=...` and
  `align=left` or in a `\parbox`, never on a single line that can overflow.
  Prefer sizes `\LARGE`-`\Huge` with `\raggedright` or `\centering`.
- **Hyphenation.** To avoid breaking words in a big title use
  `\hyphenpenalty=10000\exhyphenpenalty=10000` inside the group.
- **Language of the labels.** The labels ("Alumno", "Materia"...) are text in
  your file. To follow the document language (es or en) define:

  ```latex
  \newcommand{\Etiqueta}[2]{\ifcsname captionsspanish\endcsname #1\else #2\fi}
  ```

  and write `\Etiqueta{Materia}{Course}`. Do NOT use `\iflanguage{spanish}`:
  when the document is in English babel does not know Spanish and the build
  fails.
- **Logos.** The user's logos are `logo-universidad.png` and
  `logo-facultad.png`. They are optional, so always wrap them:

  ```latex
  \IfFileExists{logo-universidad.png}{\includegraphics[height=1.4cm]{logo-universidad}}{}%
  \IfFileExists{logo-facultad.png}{\includegraphics[height=1.0cm]{logo-facultad}}{}%
  ```

  Scale by HEIGHT (a crest is almost square and a faculty logo is a long
  strip), without extension in `\includegraphics`, and place them on a white
  or light area (they are made for white paper). The cover must look right
  without them. Do not rely on any other image: draw the decoration with
  TikZ. (If you really need another image file, it must live in a
  `logos/` folder next to the design, but then the user's shared logos are
  not used for this design.)
- **Dates, numbers.** `%%FECHA_ENTREGA%%` is already formatted and in the
  document language; print it as is.
- **Colors.** Define them once at the top with `\definecolor{..}{HTML}{RRGGBB}`
  and use them by name. If the user gives an image, estimate the main colors.
  Keep enough contrast for the text on every background.
- **Test the extremes mentally:** one-word title and a four-line title; one
  student and six team members; no teacher, no group, no faculty; English and
  Spanish. Nothing may overlap, overflow the page or leave an empty label.
- **Do not break the paper.** After `\FormatBodySetup` nothing of the design
  may change the text: no `\pagestyle`, no `\linespread`, no `\parindent`
  changes outside a group.

# 7. LATEX TRAPS WITH THIS SYSTEM

- `\ifdefempty` works on macros (`\ifdefempty{\Salon}{...}{...}`), not on the
  marker text itself.
- A `\\` inside `\ListaIntegrantes` is a line break: it needs a `tabular`, a
  `node` with `align=left`, or a `\parbox`.
- `\vspace` does nothing at the top of a page unless it is `\vspace*`.
- Keep the cover on ONE page: the sum of `\vspace`, `\vfill` and text must fit
  Letter (21.6 x 27.9 cm) with the 2.5 cm margins of the formats. Prefer
  `\vfill` to push the date to the bottom.
- In a TikZ `node`, long text needs `text width=` or it runs off the page.
- Never write a literal `%%SOMETHING%%` in a comment, and never start a
  comment line with `%%` (the program would read it as a marker).
- Do not use `\input` or `\include` of other files: only files next to the
  design with the extension `.sty` are copied (you may add your own
  `.sty`, but a single `.ltx` file is better).
- Do not use `fontspec`, `xelatex` or `lualatex` features: the build is
  pdflatex.

# 8. HOW TO WORK

1. Study the description or the image: zones of the page, colors, text sizes,
   decorations, what data it shows, and whether it has a table of contents.
2. Choose a short design name in lowercase with hyphens (`banner`,
   `blue-frame`). It must NOT be any of: `geometric-cover`, `classic-cover`,
   `report`, `starter`, `apa`, `apa-simple` (built-in names win over yours).
   The folder name is the design name.
3. Decide the data: which standard markers it shows, which of them can be
   missing (`optional`), which custom fields it needs.
4. Write `template.ltx` following sections 2 to 7, and `template.toml`.
5. Go through the checklist in section 10 line by line.

# 9. REFERENCE EXAMPLE (tested: passes `--check-template` with apa7, harvard and mla)

A cover with a dark band and an accent stripe at the top (the title in white
over the band), logos below, the data underneath with labels in the document
language, the delivery date at the bottom, and a custom field `SALON`.

`my-templates/designs/banner/template.toml`:

```toml
description = { es = "Portada con banda de color y etiquetas en el idioma del documento", en = "Cover with a color banner and labels in the document language" }
formats = ["apa7", "harvard", "mla"]
optional = ["FACULTAD"]

[fields.SALON]
label = { es = "Salón", en = "Room" }
help = { es = "Aula donde se entrega el trabajo", en = "Room where the paper is handed in" }
required = false
```

`my-templates/designs/banner/template.ltx`:

```latex
% Diseño «banner»: banda de color arriba con el título en blanco, los datos
% debajo y la fecha al pie. Etiquetas en el idioma del documento.
\documentclass[%%CLASS_OPTIONS%%]{article}
\usepackage{investigacion-base}
%%FORMAT%%

% Datos
\newcommand{\TituloTrabajo}{%%TITULO%%}
\newcommand{\NombreUniversidad}{%%UNIVERSIDAD%%}
\newcommand{\NombreFacultad}{%%FACULTAD%%}
\newcommand{\NombreAlumno}{%%ALUMNO%%}
\newcommand{\ListaIntegrantes}{%%INTEGRANTES%%}
\newcommand{\NombreMateria}{%%MATERIA%%}
\newcommand{\NombreMaestro}{%%DOCENTE%%}
\newcommand{\GrupoMateria}{%%GRUPO%%}
\newcommand{\SemestreActual}{%%SEMESTRE%%}
\newcommand{\FechaEntrega}{%%FECHA_ENTREGA%%}
\newcommand{\Salon}{%%SALON%%}

% Colores propios (xcolor ya lo carga la base).
\definecolor{BannerDark}{HTML}{1F3A5F}
\definecolor{BannerAccent}{HTML}{E8A33D}

% Etiqueta en el idioma del documento: \Etiqueta{español}{inglés}. (Con babel
% solo en inglés \iflanguage{spanish} falla; \captionsspanish existe solo si
% el español está cargado.)
\newcommand{\Etiqueta}[2]{\ifcsname captionsspanish\endcsname #1\else #2\fi}

% Una línea «Etiqueta: valor» que se omite si el valor está vacío.
\newcommand{\DatoPortada}[2]{\ifdefempty{#2}{}{\par\noindent\textbf{#1:} #2\par\vspace{0.2cm}}}

\usepackage{investigacion-final}

\begin{document}

\begin{titlepage}
  \thispagestyle{empty}
  \singlespacing
  \raggedright
  % El espacio va ANTES del dibujo: un \vspace dentro de un párrafo (el que abre
  % tikzpicture) no se aplica donde se espera. Así lo que sigue queda bajo la banda.
  \vspace*{8.9cm}
  % La banda va en una capa que ocupa toda la página (necesita dos pasadas).
  \begin{tikzpicture}[remember picture, overlay]
    \fill[BannerDark] (current page.north west) rectangle ([yshift=-8cm]current page.north east);
    \fill[BannerAccent] ([yshift=-8cm]current page.north west) rectangle ([yshift=-8.4cm]current page.north east);
    \node[anchor=north west, text=white, align=left, text width=0.78\paperwidth]
      at ([xshift=2.5cm, yshift=-3cm]current page.north west)
      {{\sffamily\bfseries\Huge \TituloTrabajo\par}};
  \end{tikzpicture}
  % Debajo de la banda: logos (opcionales) y datos.
  \IfFileExists{logo-universidad.png}{\includegraphics[height=1.4cm]{logo-universidad}}{}%
  \IfFileExists{logo-facultad.png}{\hspace{1cm}\includegraphics[height=1.0cm]{logo-facultad}}{}%
  \par\vspace{0.6cm}
  {\sffamily\large\bfseries \NombreUniversidad\par}
  \ifdefempty{\NombreFacultad}{}{{\sffamily \NombreFacultad\par}}
  \vspace{1cm}
  % Los integrantes sustituyen al alumno.
  \ifdefempty{\ListaIntegrantes}{%
    \DatoPortada{\Etiqueta{Alumno}{Student}}{\NombreAlumno}%
  }{%
    \par\noindent\textbf{\Etiqueta{Integrantes}{Members}:}\par
    \begin{tabular}[t]{@{}l@{}}\ListaIntegrantes\end{tabular}\par\vspace{0.2cm}%
  }
  \DatoPortada{\Etiqueta{Materia}{Course}}{\NombreMateria}
  \DatoPortada{\Etiqueta{Docente}{Teacher}}{\NombreMaestro}
  \DatoPortada{\Etiqueta{Semestre}{Term}}{\SemestreActual}
  \DatoPortada{\Etiqueta{Grupo}{Group}}{\GrupoMateria}
  \DatoPortada{\Etiqueta{Salón}{Room}}{\Salon}
  \vfill
  {\sffamily\bfseries \Etiqueta{Fecha de entrega}{Due date}: \FechaEntrega\par}
\end{titlepage}

% La portada cuenta como página 1.
\setcounter{page}{2}
\FormatBodySetup
\tableofcontents
\newpage

%%CONTENIDO_MARKDOWN%%

\end{document}
```

# 10. CHECKLIST (verify before answering)

- [ ] `\documentclass[%%CLASS_OPTIONS%%]{article}` and
      `\usepackage{investigacion-base}` first; `%%FORMAT%%` alone on its line,
      not in a comment.
- [ ] `\usepackage{investigacion-final}` last in the preamble; nothing loaded
      after it.
- [ ] `\FormatBodySetup` after the cover and `%%CONTENIDO_MARKDOWN%%` once,
      alone on its line, not in a comment.
- [ ] No font, geometry, spacing, caption, titlesec, fancyhdr, babel, hyperref
      or `float` package; no package reloaded with options.
- [ ] Every marker is `%%CAPITAL_LETTERS%%`; every custom field is stored in
      a command, guarded with `\ifdefempty`, and described in `template.toml`
      (and every description has its marker).
- [ ] Standard markers that may be empty are guarded; the ones the design can
      live without are in `optional`.
- [ ] Members replace the student (`\ifdefempty{\ListaIntegrantes}`).
- [ ] Logos wrapped in `\IfFileExists`, scaled by height, with the file name
      only, and the cover works without them.
- [ ] Labels follow the document language (`\ifcsname captionsspanish\endcsname`),
      not `\iflanguage`.
- [ ] The cover fits on one page for a long title and a long member list.
- [ ] `template.toml` has the description and `formats` in both languages
      and only the allowed keys: `description`, `formats`, `optional`,
      `fields`.

# 11. DELIVERY

Return exactly this, with no long explanations:

1. `template.ltx` in one code block (language `latex`).
2. `template.toml` in one code block (language `toml`).
3. One short line with the design name and any decision you took.
4. These commands, with the real design name (a Linux/macOS shell; on Windows
   create the folder and the files by hand):

```bash
mkdir -p my-templates/designs/<name>
# save template.ltx and template.toml inside that folder, then:
investigacion --check-template <name>
investigacion paper.md -p <course profile> --design <name> --format apa7
```

If `--check-template` prints errors, send them back to be fixed.

The description of the design the user wants is:
````````

---

## Notes for the maintainer

- The reference example above lives in this file only; it was checked with
  `investigacion --check-template` in a copy of the project (all three
  formats) and built with `--doc-lang es` and `--doc-lang en`.
- Keep this prompt in sync with the design contract: `.claude/skills/latex-templates/SKILL.md`,
  `src/template.rs` (markers, manifest keys) and `src/check.rs` (what is
  checked).
