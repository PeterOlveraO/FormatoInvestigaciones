# Prompt: have an AI write the paper

This file holds a ready-to-paste prompt.

1. Copy everything inside the big block below and paste it into ChatGPT,
   Claude, Gemini or the model you use.
2. After it, write your request: topic, length, language (Spanish by default),
   format (APA 7 by default, Harvard or MLA 9) and your teacher's
   instructions. Example: *"Software development life cycles, about eight
   pages, APA 7, compare at least four models."*
3. Save the answer as-is in `input/my-paper.md` (a `.md` file).
4. Generate the PDF and read the warnings, if any:

```bash
investigacion my-paper.md --title "The topic" --course "The course"
# another format:  --format harvard   or   --format mla
```

Tips:

- For the model to see a real example of every element, also attach
  [`examples/catalog.md`](../../examples/catalog.md).
- If the paper comes out with only paragraphs, ask: *"Include at least two
  Graphviz diagrams and one chart with real data about the topic."*
- The only element that can stop the build is a badly written `pgfplot` or
  `tikz` block (real LaTeX). The error message gives the line: ask the model
  to fix that block, or delete it.

---

## The prompt

````````text
You are going to write an academic paper in Markdown. A program called
`investigacion` turns that Markdown into a PDF (Pandoc, then LaTeX) in APA 7,
Harvard or MLA 9 format. The program adds the cover, the table of contents,
the page numbers and all the typography. Your job is to deliver Markdown that
(a) has good academic content and (b) compiles without errors or warnings.
Follow every rule below exactly: they come from what the program really
supports and was tested.

# 1. THE REQUEST

The user's request comes at the end of this message. From it, decide:

- **Language.** Write the whole paper (headings, captions, diagram labels,
  chart labels, table contents, references) in the language the user asks for.
  If they do not say, write in Spanish.
- **Format.** APA 7 if the user does not say otherwise; also Harvard or MLA 9.
  The format changes the headings (section 3) and the citation style
  (section 12). The Markdown syntax is the same for all three.
- **Length and depth.** Follow the requested length. If none is given, write
  about 8 pages (about 2,500 words) with a table, a diagram and a chart.
- **Missing details.** Do not ask questions: assume reasonable choices and
  write the paper.

# 2. WHAT NOT TO WRITE

The program adds all of this from its own settings. If you write it, it
appears twice or breaks the layout:

- No cover page, no paper title (no `# Title` at the top), no author, student,
  teacher, course, university, faculty or date.
- No table of contents or index.
- No YAML header (`---` block with `title:`, `author:`...) and no HTML
  comments.
- No page numbers, headers or footers, and no page breaks (they do not exist;
  the program paginates).
- No LaTeX outside the three places where it is allowed: math between `$`
  signs, and the `pgfplot` and `tikz` blocks (sections 8 and 9). Anywhere
  else a backslash command such as `\newpage`, `\textbf{x}` or `\section{x}`
  is printed literally in the PDF.
- No numbers in headings (`# 1. Introducción`): Harvard numbers them by
  itself and APA/MLA do not number.

# 3. STRUCTURE

The document starts directly with the first section. Use level-1 headings
(`#`) for the main sections and `##`, `###` for the subtopics.

- **APA 7:** use exactly these four level-1 headings, in this order, spelled
  like this (the program warns if one is missing or out of order):

  ```
  # Introducción
  # Desarrollo
  # Conclusión
  # Referencias
  ```

  Put the subtopics inside `# Desarrollo` with `##` and `###`. If the paper is
  in English, use `# Introduction`, `# Body`, `# Conclusion` and
  `# References`; the program may then print harmless warnings about the
  Spanish names, and the PDF is still correct.
- **Harvard and MLA 9:** no heading is required, but use the same four
  sections (with `# Referencias` / `# References` last). Do not write numbers:
  Harvard adds `1`, `1.1`, `1.1.1` automatically.
- **Heading levels:** `#` level 1, `##` level 2, `###` level 3, `####`
  level 4, `#####` level 5. Never skip a level (no `###` right after `#`).
  Prefer to stop at level 3. Levels 4 and 5 become a run-in heading: they
  end with a period and the paragraph continues on the same line, so write
  text right after them and never put a list, table or figure directly
  under them.
- **Headings are plain text.** No bold, italics, code, links, footnotes,
  formulas, emoji or the characters `#` and `\` inside a heading.
- **The references section is always the last one.**

Minimal skeleton (APA 7):

``````markdown skeleton
# Introducción

Presentation of the topic, why it matters and what the paper covers.

# Desarrollo

## First subtopic

Paragraphs.

## Second subtopic

Paragraphs.

# Conclusión

Closing paragraphs.

# Referencias

Surname, N. (2022). *Title of the book*. Publisher.
``````

# 4. RULES THAT ARE NOT NEGOTIABLE

- **Separate EVERY block with a real blank line:** before and after each
  heading, paragraph, list, table, quote, box, image, diagram, chart and code
  block. Almost every broken PDF comes from a missing blank line.
- **Only normal spaces.** Never use non-breaking spaces (U+00A0) or other
  invisible spaces, and do not leave spaces at the end of lines. The only
  exception is the two-space trailing line break, which you should not use
  anyway (use `<br>`, see below).
- **Write real paragraphs.** Complete, well-connected paragraphs; do not turn
  the paper into bullet lists. Use each element only when it adds something:
  a table to compare, a diagram for a structure or process, a chart for
  numbers, a box for an important definition.
- **Do not start a paragraph with a number and a period, or with a single
  capital letter and a period** (`2022. In that year...`, `A. Smith said...`):
  Markdown turns it into a numbered list. Rephrase the sentence.
- **Dollar signs.** Two `$` in the same paragraph can be read as a formula.
  Write currency as `\$5` (a backslash before the dollar sign).
- **Special characters.** To print a Markdown symbol literally, put a
  backslash before it: `\*`, `\_`, `\#`, `\[`, `\<`. `&`, `%` and `_` are safe
  in normal text. A path such as `C:\Users\ana` is printed as written.
- **Quotes and dashes** are typeset automatically: `"text"` becomes curly
  quotes, `--` an en dash, `---` an em dash, `...` an ellipsis.
- **Do not invent sources, data, quotations or figures.** If you are not sure
  of a reference, say so in your answer instead of making it up. The numbers
  in tables and charts must be real or clearly marked as an illustrative
  example in the text.

# 5. TEXT

``````markdown
Inside a paragraph: **bold**, *italic*, ***both***, `inline code`,
~~strikethrough~~ and ==highlight==. Chemistry and math notation: H~2~O
(subscript) and X^2^ (superscript). Underline with <u>this</u>, a
<mark>marked</mark> word, a key such as <kbd>Ctrl</kbd>, and a forced line
break<br>in the middle of a paragraph.

Links: [with its text](https://pandoc.org), a bare address such as
https://www.markdownguide.org, or <https://www.latex-project.org>. A link to
another section uses the automatic identifier of its heading (lowercase,
without accents, hyphens for spaces): [see the method](#metodo).

A footnote goes in the text[^one], and its content on another line, anywhere
in the document. There is also an inline form^[Written right here, inside the
sentence.].

[^one]: The content of the footnote. It can have *formatting*.
``````

Use bold sparingly (a term defined for the first time), italics for foreign
words, titles and emphasis, and highlight almost never. HTML tags other than
the ones above (`<div>`, `<span>`, `<p>`, `<details>`, `<center>`) are NOT
supported and are dropped without warning, so never use them. Supported
inline HTML: `<br>`, `<mark>`, `<sub>`, `<sup>`, `<u>`, `<ins>`, `<kbd>`,
`<samp>`, `<code>`, `<em>`, `<i>`, `<strong>`, `<b>`, `<del>`, `<s>`,
`<strike>`, `<img src="..." alt="...">` and `<hr>` on its own line.

# 6. LISTS, QUOTES AND BOXES

``````markdown
- A bullet list uses a hyphen
- Nested items use two spaces
  - Second level
    - Third level
- Back to the first level

1. A numbered list
2. Second step
   1. Sub-step with three spaces
3. Third step

- [x] A finished task
- [ ] A pending task

Glossary term
: Its definition, after a colon at the start of the next line.

Another term
: Another definition.

> A block quote starts with the greater-than sign. Use it for textual quotes
> of more than about 40 words, and cite the source after it.
>
> It can have several paragraphs.
``````

Highlighted boxes (a thin black frame with a bold title; they do not float).
The valid classes are `nota`, `aviso`, `importante`, `ejemplo` and
`definicion`; the English names `note`, `warning`, `important`, `example` and
`definition` also work. The automatic title is in the language of the
document (Nota, Aviso, Importante, Ejemplo, Definición). To change the title,
use the `title` attribute. Use at most a few boxes per paper.

``````markdown
::: nota
The content of the box. It can have **formatting**, lists and `code`.
:::

::: {.definicion title="Binary search tree"}
Data structure where every node has at most two children.
:::

::: {.aviso title="Before submitting"}
A warning with its own title.
:::
``````

The opening line has exactly three colons. The closing line is only `:::`.
Do not put a box inside another box or inside a list.

# 7. TABLES, CODE, MATH AND SYMBOLS

Tables use the pipe syntax. The row of dashes under the header is required;
the colons in it set the alignment of each column (`:---` left, `:---:`
center, `---:` right). The caption goes AFTER the table, separated by a blank
line, on a line that starts with a colon and a space. The program numbers the
tables itself (Tabla 1, Tabla 2...) in order. Do not type the number in the
caption.

``````markdown
| Model | When it fits | Risk |
|:------|:------------:|-----:|
| Waterfall | Stable requirements | Low |
| **Spiral** | Large `projects` | 3.5 |
| Agile | Frequent changes[^table] | 12 % |

: Comparison of development models

[^table]: A footnote also works inside a table cell.
``````

Table rules:

- Always write the caption. Mention the table in the text ("la Tabla 1
  muestra...") using the number it will have by counting the order.
- Cells hold inline text only (no lists, paragraphs or merged cells). Write a
  literal bar as `\|`.
- Use up to about 6 columns. If a cell has a lot of text, make the dashes of
  the separator row proportional to the width each column should have
  (a long column gets many dashes, a short one few); otherwise the columns
  are split evenly:

``````markdown
| Concept | Detailed description of the concept | Example |
|--------|--------------------------------------------------|-------|
| Alpha | A long text that wraps over several lines in the column | One |
| Beta | Another equally long text that is split by the width | Two |

: Table with long text
``````

- Long tables break across pages by themselves and repeat the header.

Code goes in a fenced block (three backticks). Always write the language
after them so that it gets syntax colors (`python`, `java`, `c`, `cpp`,
`javascript`, `sql`, `bash`, `json`, `html`, `rust`...). Keep every line under
about 80 characters, because long lines run off the page and are not wrapped.
A block without language is monospaced and uncolored, and it is the right
place for an ASCII drawing or program output:

``````markdown
```python
def binary_search(items, target):
    """Return the position of target, or -1 if it is missing."""
    low, high = 0, len(items) - 1
    while low <= high:
        mid = (low + high) // 2
        if items[mid] == target:
            return mid
        if items[mid] < target:
            low = mid + 1
        else:
            high = mid - 1
    return -1
```

```
Input:   [5, 3, 8, 1]
Output:  [1, 3, 5, 8]
```
``````

Math is LaTeX between dollar signs: `$...$` inline and `$$...$$` on its own
paragraph. Everything from `amsmath`/`amssymb` works (`\frac`, `\sum`, `\int`,
`\sqrt`, `\mathbb{R}`, `\mathbf`, `\vec`, `\hat`, `\overline`, `\mathcal`,
`\operatorname`, `\text{...}`, `bmatrix`, `cases`, `aligned`). Do not use
`\label` or `\tag` and do not nest `$` inside `\text`.

``````markdown
Inline math such as $E = mc^2$ or $\alpha + \beta \leq \gamma$ is written in
the sentence, and the set of real numbers is $\mathbb{R}$.

$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$$

$$A = \begin{bmatrix} 1 & 2 \\ 3 & 4 \end{bmatrix}$$

$$f(x) = \begin{cases} x^2 & \text{if } x \geq 0 \\ -x & \text{if } x < 0 \end{cases}$$

$$\begin{aligned} a &= b + c \\ d &= e \end{aligned}$$
``````

Symbols can be typed directly in the text: ≠ ≤ ≥ ≈ ≡ ∈ ∉ ⊆ ∪ ∩ ∑ ∏ ∫ ∂ √ ∞
∀ ∃ ⇒ ⇔ → ← ± × ÷ ° ✓ ✗ and the Greek letters α β γ δ ε θ λ μ π σ φ ω Δ Σ Ω.
Other symbols (for example ℝ ℕ ≪ ⊕ ∴ ♥) are NOT available as plain text and
appear as `[?]`: write them as math (`$\mathbb{R}$`, `$\oplus$`,
`$\therefore$`). Emoji work (🚀 💡 ✅, or `:joy:`) but do not use them in an
academic paper.

# 8. IMAGES

Only use an image if the user gives you a file name or an address. Never
invent a path or a URL. The caption is the text between the brackets: the
program numbers the figure itself (Figura 1, Figura 2...) in order, together
with the diagrams and charts that have a caption. Do not type the number.

``````markdown
![Caption of the figure](picture.png){width=60%}

![An image from the web](https://example.com/figure.png){width=50%}
``````

- Allowed formats: PNG, JPG and PDF only. SVG, WEBP, GIF and BMP produce a
  warning and are replaced by their caption.
- A local file is searched next to the Markdown file; a web address is
  downloaded the first time. If it cannot be found, the PDF is still
  generated, with the caption instead of the image.
- `{width=60%}` sets the width as a percentage of the text width (the image is
  never larger than the page). Without it the image is shown at its natural
  size, reduced if it does not fit.
- The caption position depends on the format (APA above, Harvard and MLA
  below): do not worry about it.

# 9. DIAGRAMS WITH GRAPHVIZ

For a hierarchy, flow, process, graph, automaton or data model use a `dot`
block. It is drawn as a vector image and numbered as a figure when it has a
`caption`. Follow these rules:

- The block opens with ```` ```{.dot caption="Caption of the figure"} ```` and
  closes with three backticks. The caption goes inside double quotes, so it
  must not contain a double quote (use «» or single quotes). The caption is
  optional; without it the diagram has no number.
- Write every node name and label in double quotes: `"Diseño"`,
  `"Base de datos"`. End each statement with `;`.
- `digraph` with `->` for directed graphs; `graph` with `--` for undirected
  ones (`layout=neato` for networks).
- Add `fontname="Helvetica", fontsize=10` to `node` so it matches the document.
- Page width is about 16 cm: use `rankdir=LR` for long chains and keep a
  diagram under about 12 nodes.
- If the Graphviz code has an error, the block stays as plain code with a
  warning: the PDF is still produced, but the diagram is lost. Check the
  braces and the semicolons.

``````markdown
```{.dot caption="Class hierarchy of a figure system"}
digraph {
  graph [ranksep=0.45];
  node  [shape=box, style=rounded, fontname="Helvetica", fontsize=10];
  edge  [arrowsize=0.7];

  "Figura" -> "Círculo";
  "Figura" -> "Polígono";
  "Polígono" -> "Triángulo";
  "Polígono" -> "Cuadrilátero";
}
```

```{.dot caption="Compilation pipeline"}
digraph {
  graph [rankdir=LR, ranksep=0.5];
  node  [shape=box, style=rounded, fontname="Helvetica", fontsize=10];
  edge  [arrowsize=0.7];

  "archivo.md" -> "Pandoc" -> "LaTeX" -> "PDF";
}
```

```{.dot caption="Search in a binary tree"}
digraph {
  graph [ranksep=0.35];
  node  [fontname="Helvetica", fontsize=10];
  edge  [arrowsize=0.7, fontname="Helvetica", fontsize=9];

  inicio  [shape=ellipse, label="Inicio"];
  vacio   [shape=diamond, label="¿Nodo vacío?"];
  hallado [shape=ellipse, label="Encontrada"];
  izq     [shape=box, label="Bajar a la izquierda"];

  inicio -> vacio;
  vacio -> hallado [label="sí"];
  vacio -> izq     [label="no"];
  izq -> vacio;
}
```

```{.dot caption="Three-tier architecture"}
digraph {
  graph [ranksep=0.4];
  node  [shape=box, style=rounded, fontname="Helvetica", fontsize=10];
  edge  [arrowsize=0.7];

  subgraph cluster_presentacion {
    label="Presentación"; fontname="Helvetica"; fontsize=10; color=gray60;
    "Web"; "Móvil";
  }
  subgraph cluster_datos {
    label="Datos"; fontname="Helvetica"; fontsize=10; color=gray60;
    "Base de datos";
  }

  "Web" -> "Base de datos";
  "Móvil" -> "Base de datos";
}
```
``````

Other shapes you can use: `ellipse`, `diamond`, `circle`, `doublecircle`,
`record` (fields separated by `|`, for data models) and `point`. For a very
simple scheme you may also draw with box characters (`└ ─ ┼ │ ►`), but ALWAYS
inside a code block without a language; outside it the drawing falls apart.

# 10. DATA CHARTS (pgfplots) AND FREE DRAWINGS (TikZ)

This is real LaTeX, and it is the only place where a mistake STOPS the whole
build. Be conservative: copy one of the templates below and change only the
data, the labels and the caption.

How it works: the block content is placed inside a `tikzpicture` (do not write
that environment yourself), and with a `caption` it becomes a numbered figure.
Chart blocks use the class `pgfplot`, free drawings the class `tikz`.

Safety rules for every `pgfplot` and `tikz` block:

1. Arrows such as `->`, `<->` or `-latex` are fine in any language.
2. Do not write `\begin{figure}`, `\caption`, `\label`, `\centering`,
   `\usepackage` or `\pgfplotsset`. The program adds the figure and the
   packages.
3. Do not set colors or global styles: the format already defines a sober
   grayscale palette. The only allowed colors are shades of gray
   (`gray!30`). Each series is distinguished by its marker or its dashes.
4. Escape special characters inside labels and text: `\%`, `\&`, `\#`, `\_`.
   Write decimals with a point (`3.5`); the PDF shows them with the right
   separator.
5. Every `{` must be closed and every `\begin{axis}` must have its
   `\end{axis}`. Every `\addplot` ends with `;`.
6. Names after `symbolic x coords` (or `symbolic y coords`) must be written
   exactly the same in every `\addplot`, with no spaces inside a name, and
   `xtick=data` is required with them.
7. If you need a TikZ library, write `\usetikzlibrary{positioning,
   arrows.meta, shapes.geometric}` as the first line of a `tikz` block (this
   was tested). Without it only basic TikZ is available: `rectangle`,
   `circle`, `\node`, `\draw`, `\fill`, `\foreach`, coordinates `(x,y)` and
   `at (x,y)`.
8. Use `ymin=0` for bar charts, and keep at most 4 series.
9. If you are not sure that a block is correct, replace it with a table or
   a Graphviz diagram. A wrong block loses the entire paper, a missing one
   does not.

Chart templates (all tested). Change the data and the labels only.

``````markdown
```{.pgfplot caption="Hours spent per project phase"}
\begin{axis}[
  ybar, ymin=0, bar width=18pt, nodes near coords,
  ylabel={Hours}, symbolic x coords={Análisis,Diseño,Código,Pruebas},
  xtick=data, enlarge x limits=0.18]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180) (Pruebas,140)};
\end{axis}
```

```{.pgfplot caption="Defects found per phase and team"}
\begin{axis}[
  ybar, ymin=0, bar width=10pt,
  ylabel={Defects}, symbolic x coords={Análisis,Diseño,Código,Pruebas},
  xtick=data, enlarge x limits=0.18, legend pos=north west]
  \addplot coordinates {(Análisis,12) (Diseño,18) (Código,45) (Pruebas,30)};
  \addplot coordinates {(Análisis,8) (Diseño,14) (Código,38) (Pruebas,22)};
  \legend{Team A, Team B}
\end{axis}
```

```{.pgfplot caption="Languages used in the projects"}
\begin{axis}[
  xbar, xmin=0, bar width=12pt, nodes near coords,
  xlabel={Projects}, symbolic y coords={Java,Python,JavaScript},
  ytick=data, enlarge y limits=0.2, width=0.75\linewidth, height=5cm]
  \addplot coordinates {(14,Java) (31,Python) (22,JavaScript)};
\end{axis}
```

```{.pgfplot caption="Distribution of the effort per semester"}
\begin{axis}[
  ybar stacked, ymin=0, bar width=20pt,
  ylabel={Hours}, symbolic x coords={2024-1,2024-2,2025-1},
  xtick=data, legend pos=outer north east]
  \addplot coordinates {(2024-1,40) (2024-2,55) (2025-1,60)};
  \addplot coordinates {(2024-1,30) (2024-2,25) (2025-1,35)};
  \legend{Theory, Practice}
\end{axis}
```

```{.pgfplot caption="Project progress during the semester"}
\begin{axis}[
  xlabel={Week}, ylabel={Progress (\%)}, ymin=0, ymax=100,
  legend pos=north west, xtick={2,4,6,8}]
  \addplot coordinates {(2,10) (4,35) (6,70) (8,100)};
  \addplot coordinates {(2,6) (4,25) (6,60) (8,95)};
  \legend{Planned, Actual}
\end{axis}
```

```{.pgfplot caption="Study hours against grade, with a trend line"}
\begin{axis}[xlabel={Study hours}, ylabel={Grade}, legend pos=south east]
  \addplot[only marks] table[x=hours, y=grade] {
    hours grade
    2 62
    4 71
    6 80
    8 91
  };
  \addplot[no marks, thick, black] table[x=hours, y={create col/linear regression={y=grade}}] {
    hours grade
    2 62
    4 71
    6 80
    8 91
  };
  \legend{Students, Trend}
\end{axis}
```

```{.pgfplot caption="Growth of three functions"}
\begin{axis}[
  xlabel={$n$}, ylabel={Operations}, domain=1:50, samples=80,
  legend pos=north west, ymax=250]
  \addplot[mark=none, thick] {x^2/10};
  \addplot[mark=none, densely dashed, thick] {x};
  \legend{$\Theta(n^2)$, $\Theta(n)$}
\end{axis}
```

```{.pgfplot caption="Distribution of the final grade"}
\pie[text=legend, radius=1.8, color={gray!70, gray!45, gray!20, gray!5}]
  {40/Practices, 30/Exam, 20/Project, 10/Participation}
```

```{.tikz caption="A simple flow drawn with TikZ"}
\node[draw, rounded corners, fill=gray!15, minimum width=2cm, minimum height=0.8cm] (a) at (0,0) {Input};
\node[draw, rounded corners, fill=gray!15, minimum width=2cm, minimum height=0.8cm] (b) at (4,0) {Process};
\node[draw, rounded corners, fill=gray!15, minimum width=2cm, minimum height=0.8cm] (c) at (8,0) {Output};
\draw[-latex, thick] (a) -- (b);
\draw[-latex, thick] (b) -- node[above, font=\small] {result} (c);
```
``````

The code `\pie` needs no `axis`; the other charts need exactly one `axis`
environment. The same data can also be shown as a table; prefer the chart
when the point is the comparison or the trend.

# 11. FIGURE AND TABLE NUMBERING

Figures (images, `dot` diagrams and `pgfplot`/`tikz` blocks with a caption)
share one counter: Figura 1, Figura 2... in order of appearance. Tables have
their own counter. In the text, refer to them by that number ("como muestra
la Figura 2"), counting in order. Introduce every figure and table in a
sentence before or after it; never leave one without mentioning it.

# 12. REFERENCES AND CITATIONS

**Citations in the text** are plain text. Do not use citation keys such as
`[@cormen2022]` (they are printed literally) and no `\cite`. Use the style of
the chosen format and cite only sources that appear in the reference list:

| Format | In the text | Example |
|---|---|---|
| APA 7 | (Author, year), (Author & Author, year), (Author et al., year); with a page: (Author, year, p. 14) | (Cormen et al., 2022) |
| Harvard | (Author year), with a page: (Author year, p. 14) | (Cormen et al. 2022, p. 14) |
| MLA 9 | (Author page), no year | (Cormen 45) |

In Spanish write "y" instead of "&" inside the citation in APA ("Pérez y
López, 2020") and use "p." or "pp.".

**The reference section.**

- Its heading must be one of these (any level, any case, with or without
  accent): `Referencias`, `Bibliografía`, `Referencias bibliográficas`,
  `Lista de referencias`, `Obras citadas`, `Fuentes consultadas`, `Fuentes`,
  `References`, `Reference list`, `Bibliography` or `Works Cited`. Use
  `# Referencias` (in English `# References`; for MLA `# Obras citadas` or
  `# Works Cited`). Any other name loses the hanging indent.
- The program applies the hanging indent (first line at the margin, the rest
  indented) to everything under that heading until the next heading. So put
  nothing else there.
- Write one reference per paragraph, separated by a BLANK line (consecutive
  lines without a blank line would merge into a single paragraph), or as a
  hyphen list; both give the same result.
- Sort the list alphabetically by the first author's surname. Use italics
  (`*...*`) for the title of a book, report, journal or web page, as the
  style requires.
- Write DOIs and URLs as plain text, in full (`https://doi.org/10.xxxx/yyyy`);
  long addresses are broken automatically.
- Include only real, verifiable sources the paper actually cites. If the user
  gives sources, use those. If you cannot be certain, write fewer references
  and say so in your answer.

``````markdown references
# Referencias

Cormen, T. H., Leiserson, C. E., Rivest, R. L., & Stein, C. (2022). *Introduction to algorithms* (4.ª ed.). MIT Press.

MacFarlane, J. (2025). *Pandoc user's guide*. https://pandoc.org/MANUAL.html
``````

Reference style per format (adapt the punctuation to the language):

- **APA 7:** `Surname, N. N., & Surname, N. (year). *Title in italics*. Publisher. URL`
  Article: `Surname, N. (year). Title of the article. *Journal, vol*(issue), pages. https://doi.org/...`
- **Harvard:** `Surname, N. (year) *Title in italics*. Edition. City: Publisher.`
  Article: `Surname, N. (year) 'Title of the article', *Journal*, vol(issue), pp. 1-10.`
- **MLA 9:** `Surname, Name. *Title in Italics*. Publisher, year.`
  Article: `Surname, Name. "Title of the Article." *Journal*, vol. 3, no. 2, year, pp. 1-10.`

# 13. THINGS THAT BREAK OR GET LOST (summary)

- A missing blank line (the most common failure: headings, tables or lists
  appear as plain text with `#`, `|` or `-` characters).
- Invisible or non-breaking spaces copied from other tools.
- Raw LaTeX outside math, `pgfplot` and `tikz` (printed literally), and any
  syntax error inside `pgfplot` or `tikz` (build stops).
- `<` or `>` characters inside a `pgfplot` or `tikz` block (build stops).
- Unsupported HTML (`<div>`, `<span>`, `<center>`...): its content is lost.
- SVG, WEBP or GIF images, and invented image paths.
- Characters outside the list in section 7 (`ℝ`, `ℕ`, `♥`...): they show up
  as `[?]`.
- A table without the row of dashes under the header: it is printed as bars.
- Code lines longer than about 80 characters: they run off the page.
- ASCII drawings outside a code block: the alignment is lost.
- A reference list with a heading the program does not recognize, or with
  references glued together without blank lines.
- Citation keys (`[@key]`), YAML headers, `\newpage` and manual numbers in
  headings.

# 14. FINAL CHECKLIST (verify before answering)

- [ ] The paper is in the language the user asked for (Spanish by default).
- [ ] It starts directly with the first section: no cover, title, author,
      date or table of contents.
- [ ] APA 7: the four headings exactly as in section 3. Harvard and MLA: the
      same four sections, no numbers in headings.
- [ ] Heading levels are not skipped and have no formatting in them.
- [ ] A blank line before and after every block.
- [ ] No raw LaTeX outside math, `pgfplot` and `tikz` blocks.
- [ ] Every `pgfplot` and `tikz` block comes from a template, has balanced
      braces and contains no `<` or `>`.
- [ ] Every table and figure has a caption and is mentioned in the text.
- [ ] Code blocks have a language and short lines.
- [ ] Citations and references follow the requested format; the reference
      list is alphabetical, last, and every source is real.
- [ ] No emoji, HTML other than the supported tags, YAML or HTML comments.

# 15. DELIVERY

Return ONLY the Markdown paper, with no explanation before or after it:

- If you can create a file, deliver it as `paper.md`.
- Otherwise, put the whole paper in ONE single code block with the language
  `markdown`, fenced with FIVE backticks (not three), because the paper
  itself contains blocks fenced with three backticks that would close a
  shorter fence.

If you must warn about something (for example, a reference you could not
verify), do it in one short line after the code block.

The request of the user is:
````````

---

## If the build fails

1. Read the error: it gives the line of the temporary `.tex` file and the
   last LaTeX lines. Almost always the cause is a `pgfplot` or `tikz` block.
2. Ask the model: *"This block fails with this error: ... Fix it using only the
   templates of the prompt, without `<` or `>`."* Or delete the block.
3. Warnings (for example a missing image or a symbol shown as `[?]`) do not
   stop the PDF; they tell you what to fix.
