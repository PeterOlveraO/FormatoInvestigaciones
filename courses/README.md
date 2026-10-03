# Course profiles

Each `.toml` file here stores what repeats in every paper of a course, so you
do not type it each time:

| Field | Meaning | Required |
|---|---|---|
| `name` | Course name, as shown on the cover | Yes |
| `teacher` | Teacher | No |
| `group` | Group | No |
| `members` | Team members, separated by commas | No |
| `template` | Template from `templates/` (`apa`, `apa-simple`…) | No (`apa`) |
| `folder` | Subfolder of `input/` and `output/` for this course | No |

The file name is the profile's key: `ia.toml` is used with `-p ia`, and it
appears in the menu's profile list. Copy `example.toml` to start.

Precedence: what you type on the command line > the profile > `.env`.

This folder is ignored by git except for `example.toml` and this file, because
your teachers' names are your own data.
