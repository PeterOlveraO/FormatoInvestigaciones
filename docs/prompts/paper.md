# Prompt: have an AI write the paper

1. Copy the block below into ChatGPT, Claude, Gemini or the model you use.
2. After it, write your request: the **topic**, the **citation format** your
   teacher asks for (APA 7 if you say nothing; also Harvard, MLA or any
   other), length, language (Spanish by default) and any other instructions.
   Example: *"Software development life cycles, Harvard, about eight pages,
   compare at least four models."*
3. Save the answer in `input/my-paper.md` and generate the PDF with the same
   format (`--format` or the menu):

```bash
investigacion my-paper.md --title "The topic" --course "The course" --format harvard
```

The Markdown is the same for every format: the program sets the layout. The
format only changes how the model writes citations and references, which is
why the request must name it.

Attach [`examples/catalog.md`](../../examples/catalog.md) too if you want the
model to see every element in use. If the build stops, the cause is a
`pgfplot` or `tikz` block: ask the model to fix that block, or delete it.

---

## The prompt

````````text
Write an academic paper in Markdown. A program turns it into a PDF and adds
the cover, title, table of contents, page numbers and typography. Deliver
Markdown with good content that compiles cleanly. Follow these rules exactly.

The input is Pandoc Markdown (without citeproc or raw LaTeX): any Pandoc
Markdown feature works even if it is not listed here. The rules below say
what the program adds, what breaks, and its own extras (boxes, diagrams,
charts).

# REQUEST

The user's request is at the end. Write everything (headings, captions,
labels, tables, references) in the language they ask for; Spanish if they do
not say. If no length is given, write about 2,500 words with at least one
table, one diagram and one chart. Do not ask questions: decide and write.

# DO NOT WRITE

- Cover, title (`# Title` at the top), author, course, date, table of
  contents, YAML header, HTML comments, page breaks.
- Numbers in headings (`# 1. Introducción`).
- LaTeX outside math (`$...$`) and `pgfplot`/`tikz` blocks: `\newpage`,
  `\textbf{}` and similar are printed literally.
- HTML other than `<br>`, `<u>`, `<mark>`, `<kbd>`, `<sub>`, `<sup>`: the
  rest (`<div>`, `<span>`, `<center>`...) is dropped.

# STRUCTURE

Start directly with the first section, using exactly these level-1 headings
in this order (English papers: Introduction, Body, Conclusion, References):

```
# Introducción
# Desarrollo
# Conclusión
# Referencias
```

Subtopics go inside `# Desarrollo` with `##` and `###`. Never skip a level.
Headings are plain text: no formatting, links, math or emoji.

# BASIC RULES

- A blank line before and after EVERY block (heading, paragraph, list,
  table, box, figure, code). A missing one is the most common failure.
- Only normal spaces; no non-breaking or trailing spaces.
- Real, connected paragraphs; use lists, tables and figures only where they
  add something.
- Do not start a paragraph with `2022.` or `A.` (it becomes a list).
- Write currency as `\$5`. Escape literal Markdown symbols: `\*`, `\#`, `\_`.
- Never invent sources, data, quotes or image paths.

# TEXT

**bold**, *italic*, `code`, ~~strike~~, ==highlight==, H~2~O, X^2^,
<kbd>Ctrl</kbd>, [link](https://pandoc.org), a link to a section
[see the method](#metodo), a footnote[^1] or an inline one^[Like this.].

[^1]: Footnote text.

Lists: `-` bullets (nest with two spaces, at most 4 levels), `1.` numbered, `- [x]` tasks, and
definitions (`Term` on one line, `: Definition` on the next). Quotes longer
than ~40 words go in `>` blocks.

Symbols you can type: ≠ ≤ ≥ ≈ ∈ ⊆ ∪ ∩ ∑ ∫ √ ∞ ∀ ∃ ⇒ → ± × ° ✓ ✗ and Greek
letters. Anything else (ℝ, ⊕, ∴) goes in math: `$\mathbb{R}$`.

# BOXES

Classes: `nota`, `aviso`, `importante`, `ejemplo`, `definicion` (or `note`,
`warning`, `important`, `example`, `definition`). The title is automatic;
`title` changes it. A few per paper at most, never nested or inside a list.

```
::: nota
Content with **formatting** or lists.
:::

::: {.definicion title="Binary search tree"}
A tree where every node has at most two children.
:::
```

# TABLES

The dash row is required; colons set alignment. The caption goes after a
blank line, starting with `: `. The program numbers tables: never type the
number, but mention each table in the text ("la Tabla 1 muestra...").
Cells are inline text only, up to ~6 columns. For long text, give each column
dashes proportional to its width.

```
| Model | When it fits | Risk |
|:------|:-------------|-----:|
| Waterfall | Stable requirements | Low |
| Agile | Frequent changes | High |

: Comparison of development models
```

# CODE AND MATH

Fenced code with a language (`python`, `java`, `sql`...) and lines under 80
characters. ASCII drawings and program output go in a block without language.

Math: `$E = mc^2$` inline, `$$...$$` as its own paragraph. `amsmath` works
(`\frac`, `\sum`, `bmatrix`, `cases`, `aligned`); no `\label` or `\tag`. A formula with an
error (unclosed brace, unknown command) stops the whole PDF: check each one.

# FIGURES

Images, diagrams and charts with a caption share one counter (Figura 1,
2...). Never type the number; mention each one in the text.

Images only if the user gives a file or URL. PNG, JPG or PDF only:

```
![Caption](picture.png){width=60%}
```

Graphviz diagrams for structures, flows and processes. Quote every name, end
statements with `;`, keep under ~12 nodes, use `rankdir=LR` for long chains.
The caption cannot contain `"`.

```{.dot caption="Compilation pipeline"}
digraph {
  graph [rankdir=LR];
  node  [shape=box, style=rounded, fontname="Helvetica", fontsize=10];
  "archivo.md" -> "Pandoc" -> "LaTeX" -> "PDF";
}
```

# CHARTS (pgfplot / tikz)

This is real LaTeX: an error here STOPS the build. Copy a template and change
only data, labels and caption. Do not write `\begin{tikzpicture}`, `figure`,
`\caption`, `\usepackage`, colors (only `gray!NN`) or styles. Escape `\%` and
`\&` in labels. Balance every `{}` and `\begin`/`\end`; end each `\addplot`
with `;`. With `symbolic x coords`, use `xtick=data` and the exact same names
(no spaces) in every `\addplot`. If unsure, use a table instead.

```{.pgfplot caption="Hours per project phase"}
\begin{axis}[
  ybar, ymin=0, bar width=14pt, nodes near coords,
  ylabel={Hours}, symbolic x coords={Análisis,Diseño,Código,Pruebas},
  xtick=data, enlarge x limits=0.18, legend pos=north west]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180) (Pruebas,140)};
  \addplot coordinates {(Análisis,100) (Diseño,90) (Código,150) (Pruebas,120)};
  \legend{Planned, Actual}
\end{axis}
```

```{.pgfplot caption="Progress during the semester"}
\begin{axis}[xlabel={Week}, ylabel={Progress (\%)}, ymin=0, ymax=100,
  legend pos=north west]
  \addplot coordinates {(2,10) (4,35) (6,70) (8,100)};
  \addplot[mark=none, densely dashed, domain=0:8] {12.5*x};
  \legend{Actual, Ideal}
\end{axis}
```

```{.pgfplot caption="Distribution of the final grade"}
\pie[text=legend, radius=1.8, color={gray!70, gray!45, gray!20, gray!5}]
  {40/Practices, 30/Exam, 20/Project, 10/Participation}
```

```{.tikz caption="A simple flow"}
\node[draw, rounded corners, fill=gray!15] (a) at (0,0) {Input};
\node[draw, rounded corners, fill=gray!15] (b) at (4,0) {Output};
\draw[-latex, thick] (a) -- node[above] {process} (b);
```

# CITATIONS AND REFERENCES

Use the citation format the user names (APA 7 if none), for example APA
`(Cormen et al., 2022, p. 14)`, Harvard `(Cormen et al. 2022, p. 14)`, MLA
`(Cormen 45)`. Citations are plain text: no `[@key]` or `\cite`. In Spanish
use "y", not "&".

`# Referencias` is the last section and holds only references: one per
paragraph separated by blank lines, alphabetical, in the same style, with
titles in `*italics*` and full DOIs/URLs. The program adds the hanging indent.
Only real sources that the text cites.

```
# Referencias

Cormen, T. H., Leiserson, C. E., Rivest, R. L., & Stein, C. (2022). *Introduction to algorithms* (4.ª ed.). MIT Press.

MacFarlane, J. (2025). *Pandoc user's guide*. https://pandoc.org/MANUAL.html
```

# DELIVERY

Return ONLY the paper. If you can create files, deliver `paper.md`;
otherwise one ```markdown block fenced with FIVE backticks. Mention any
unverified reference in one line after it.

The request of the user is:
````````
