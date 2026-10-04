# Course profiles

Each `.toml` file here keeps everything for one course: format, design and all
the cover data. The easiest way to create or edit one is the wizard (`p` in
the menu, or automatically the first time). You can also edit them by hand:

| Field | Meaning |
|---|---|
| `name` | Course name, as shown on the cover (required) |
| `university`, `faculty`, `student`, `semester` | General cover data |
| `members` | Team members, separated by commas (they replace the student) |
| `teacher`, `group` | Teacher and group |
| `format` | `apa7`, `harvard`, `mla` or one of yours |
| `design` | `geometric-cover`, `classic-cover`, `report`, `starter` or one of yours (the old key `template` still works) |
| `language` | Document language, `es` or `en` (default: the format's) |
| `folder` | Subfolder of `input/` and `output/` for this course |
| `[fields]` | The design's own fields, e.g. `SALON = "B-204"` |

The file name is the profile's key: `ia.toml` is used with `-p ia`.

Precedence: command-line option > profile. Every cover datum the design shows
comes from here (there is no `.env`); the wizard asks for all of them.

This folder is ignored by git except for `example.toml` and this file, because
the data in it (names, teachers) is your own.
