# Cover logos

Put your institution's logos here. This folder is ignored by git, because
institutional logos are rarely redistributable.

| File | Where it appears | Usually |
|---|---|---|
| `logo-universidad.png` | Top left | The university's coat of arms |
| `logo-facultad.png` | Top right | The faculty's or school's logo |

- **Optional.** Both files are optional: without them the cover is generated
  without logos.
- **Format.** Use PNG, ideally with a transparent background.
- **Color.** Use dark or colored logos, because a white logo is invisible on
  the white cover.
- **Margins.** The template scales the logos by height (1.6 cm and 0.88 cm), so
  a large transparent margin makes them look small.
- **Another folder.** To keep them outside the project, use `--logos <folder>`
  or set `LOGOS="<folder>"` in `.env`.
- **Position.** If they look misplaced, adjust the `LOGOS` block of
  `templates/designs/geometric-cover/template.ltx` (`xshift`, `yshift`, `height`).
