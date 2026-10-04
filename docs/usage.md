# Using investigacion

A short, friendly guide to turn a Markdown paper into a PDF, with the menu or
with one command. Installation is in [INSTALL.md](../INSTALL.md). Every
option is in [cli.md](cli.md); formats, designs and the Markdown syntax are
in [reference.md](reference.md).

## The idea

1. You write your paper in a Markdown file (`.md`). Put it in `input/`, or
   anywhere you like.
2. You tell the program the title and the course.
3. You get a PDF with a cover, in APA 7, Harvard or MLA 9 format.

## The menu

Run the program with no arguments:

```bash
investigacion
```

### First run

1. **Language screen.** Pick Spanish or English with ↑↓ and press Enter. The
   language of your system is already marked. The choice is saved, so it is
   asked only once. You can change it later in Options (`l`).
2. **Profile wizard.** If you have no course profiles yet, a wizard opens. A
   profile saves the data of one course (university, teacher, group, format,
   design…) so you do not type it every time. The wizard asks, one step at a
   time:
   - a short profile name, such as `ia`;
   - the format and the design;
   - the data that design shows (a `*` marks what is required);
   - the folder inside `input/` where that course's papers live (optional).

   Enter goes on, Shift+Tab goes back, Esc cancels. The profile is saved in
   `courses/<name>.toml` and chosen for you in the form.

### Home view

The home view has a short description, a status panel and four entries:

| Entry | What it does |
|---|---|
| Generate a PDF | Opens the form. |
| New profile | Opens the wizard. |
| Options | Profiles, folders, language and log. |
| Quit | Leaves the menu. |

The status panel lists your profiles, the last PDF of this session, and
whether Pandoc and pdflatex (required) and Graphviz (optional, for diagrams)
are installed. A missing required tool is shown in red.

Keys: ↑↓ move, Enter chooses, `s` or `q` quits.

### The form

Choose **Generate a PDF**. The fields are:

| Field | What to put |
|---|---|
| Course profile | Pick one: it fills the course, teacher, group, format, design and output folder. Optional. |
| Markdown file | **Required.** Browse with the arrows; typing filters the list. |
| Title | **Required.** Shown on the cover; it does not name the file. |
| PDF file name | Optional. By default, the name of the Markdown file. |
| Course, Teacher, Team members, Group | Cover data. Empty means "use the profile's". |
| Format | `apa7`, `harvard`, `mla`… Empty: the first one the design accepts. |
| Design | The cover and look. Empty: `geometric-cover`. |
| Output folder | Empty: `output/` (or the profile's folder inside it). |
| Extra copies | Folders for a copy of the PDF, separated by `;`. |

If the chosen design has its own fields (for example, a classroom), they
appear at the end. Whether Course is required depends on the design. A field
marked `[missing]` must be filled before you can generate.

Keys in the form:

| Key | Action |
|---|---|
| ↑ ↓ | Move between fields (`1`–`9` jump straight to one). |
| Enter | Edit the field, or open its list. |
| Del | Clear the field. |
| `g` | Generate the PDF (F5 also works). |
| `v` | Open the last PDF. |
| `o` | Open the options. |
| `s` or `q` | Quit. |
| Esc | Back to the home view. |

While you type a value: Enter saves, Esc cancels, Ctrl+U clears. In a list:
↑↓ move, Enter or → opens or chooses, ← or Backspace goes up a folder, and
Esc cancels.

### Options

Press `o`. Choose a row with ↑↓ and Enter, or press its letter:

| Key | Action |
|---|---|
| `p` | Edit the chosen profile (or create one if none is chosen). |
| `n` | New profile. |
| `c` (or `f`) | Open a project folder: `input`, `output`, `courses`… |
| `l` | Switch the menu between Spanish and English. |
| `r` | Open the log (what happened, and errors). |
| `i` (or `h`) | Back to the home view. |

## Generating a PDF

1. In the form, pick the Markdown file and write the title (and pick a
   profile if you have one).
2. Press `g`. The result panel says "Generating the PDF…". It can take a few
   seconds. Keys are ignored until it finishes.
3. When it ends you see **Done**. Press `v` to open the PDF.

Warnings (a missing image, an unknown symbol, a missing recommended heading)
show in yellow and do not stop the PDF.

## Where the PDF ends up

- In `output/`, or in `output/<folder>` if your profile has a folder. Press
  `o` then `c` to open it.
- Named after the Markdown file: `Tarea1.md` becomes `tarea1.pdf`. Use "PDF
  file name" to change it. Generating again replaces the file.
- Extra copies go to the folders you listed.

## Without the menu: the 5 commands you will use

Replace `Tarea1.md` and the title with yours. `Tarea1.md` can be a bare name:
it is also searched inside `input/`.

```bash
# 1. With a course profile (the easiest)
investigacion Tarea1.md -p ia --title "Búsqueda heurística"

# 2. Without a profile: give the cover data
investigacion Tarea1.md --title "Tema" --course "Cálculo" \
  --set UNIVERSIDAD="Nombre de la universidad" --set FACULTAD="Nombre de la facultad" \
  --set SEMESTRE="2026-2"

# 3. Another format and design
investigacion Tarea1.md -p ia --title "Tema" --format mla --design classic-cover

# 4. A team paper, with a copy in another folder
investigacion Tarea1.md -p ia --title "Tema" --members "Ana Ruiz, Luis Paz" --copy ~/Drive/Entregas

# 5. See every option
investigacion --help
```

Values with spaces go in quotes. The PDF path is printed when it is ready.
All options, their old Spanish names, exit codes and environment variables
are in [cli.md](cli.md).

## When something fails

- **In the menu**, the result panel shows a red line starting with `✗`. It
  says what is missing or what went wrong (for example "Cannot generate yet.
  Missing: Title." or "Missing data used by the design…"). Fix the field and
  press `g` again.
- **In the terminal**, the error is printed as `Error: …` and the command
  ends with exit code 1 (or 2 for a mistake in the options).
- **A LaTeX error**: the message shows the real error line. `last-error.tex`
  and `last-error.log` are saved next to the PDF.
- **The log**: every run, from the menu or the terminal, is recorded in
  `cache/logs/investigacion.log`. In the menu press `o` then `r` to open it.
  Look at the end of the file, or attach it when you ask for help. It holds
  paths and titles, so read it before sharing it.
- **More detail**: run with `INVESTIGACION_LOG=debug` to record the full
  commands.
- **A menu bug** ("Internal error: …") does not close the menu: the details
  are in the log and you can keep working.
- **Your own design fails**: run `investigacion --check-template <design>`.

More causes and fixes, and the Markdown syntax, are in
[reference.md](reference.md).
