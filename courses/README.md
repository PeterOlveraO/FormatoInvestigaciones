# Course profiles

Each `.toml` file here keeps everything for one course: the design and all the
cover data. The easiest way to create or edit one is the wizard (`p` in
the menu, or automatically the first time). You can also edit them by hand:

| Field | Meaning |
|---|---|
| `name` | Course name, as shown on the cover (required) |
| `university`, `faculty`, `student` | General cover data |
| `members` | Team members, separated by commas (they replace the student) |
| `teacher`, `group` | Teacher, and group or term (e.g. `7-A`) |
| `design` | `geometric-cover` or one of yours in `my-templates/designs/` (the old key `template` still works) |
| `language` | Document language, `es` or `en` (default: Spanish) |
| `folder` | Subfolder of `input/` and `output/` for this course |
| `output` | Folder for the PDFs, instead of `output/<folder>` (relative to the project, `~` works) |
| `copies` | Extra folders for a copy of the PDF, e.g. `["~/Drive/IA"]` |
| `[fields]` | The design's own fields, e.g. `SALON = "B-204"` |

The wizard rewrites the whole file when it saves, so its comments and
commented-out lines are lost. To keep them, edit the file by hand instead of
with the wizard.

The file name is the profile's key: `ia.toml` is used with `-p ia`.

Old profiles still load. `semester` and `format` are read and ignored with a
one-line warning (put the term in `group`; the format is always APA 7), and a
retired design name (`classic-cover`, `report`, `starter`, `apa`,
`apa-simple`) uses `geometric-cover`, also with a warning. When the wizard
saves a profile, it leaves `semester` and `format` out.

Precedence: command-line option > profile. Every cover datum the design shows
comes from here (there is no `.env`); the wizard asks for all of them.

`example.toml` is not listed in the menu (so a new user gets the wizard), but
`-p example` works on the command line.

This folder is ignored by git except for `example.toml` and this file, because
the data in it (names, teachers) is your own.
