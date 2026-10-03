---
name: regenerate-catalog
description: Rebuild the versioned examples/catalog.pdf and the README cover screenshot without personal data or logos. Use after changing templates, filters or examples/catalog.md.
disable-model-invocation: true
---

# Regenerate the catalog PDF and screenshots

`examples/catalog.pdf` and `docs/images/*.png` are the only generated files in
git. They **must not contain personal data or institutional logos**:

- `--env-file .env.example` makes the cover show the same placeholders a new
  clone sees ("Nombre de la universidad"…);
- an empty `--logos` folder keeps any institution's marks out.

1. Build and generate:

   ```bash
   cargo build --release
   mkdir -p /tmp/nologos
   target/release/investigacion examples/catalog.md \
     --title "Catalogo de elementos" --course "Nombre de la materia" \
     --teacher "Nombre del docente" --group "7-A" \
     --env-file .env.example --logos /tmp/nologos --output /tmp/pub --lang es
   ```

2. Compare the text with the current version. Only the intended changes and
   the delivery date should differ:

   ```bash
   pdftotext examples/catalog.pdf /tmp/old.txt
   pdftotext /tmp/pub/catalog.pdf /tmp/new.txt
   diff /tmp/old.txt /tmp/new.txt
   ```

3. Look at the pages you changed (`pdftoppm -r 45 -png -f N -l N …`), then
   copy:

   ```bash
   cp /tmp/pub/catalog.pdf examples/catalog.pdf
   pdftoppm -r 110 -png -f 1 -l 1 examples/catalog.pdf /tmp/cover
   cp /tmp/cover-01.png docs/images/cover.png
   ```

4. `diagram.png` and `charts.png` are pages of the catalog. Regenerate them
   only if those pages changed (find the page with `pdftotext -f N -l N`).
