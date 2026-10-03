# Prompt for asking an AI to write the paper

This file holds a ready-to-copy prompt.

1. Paste it into ChatGPT, Claude, Gemini or the model you use, and add your
   topic and your teacher's instructions at the end.
2. Save the answer as-is in a `.md` file inside `input/`.
3. Generate the PDF:

```bash
investigacion my-paper.md --title "The topic" --course "The course"
```

The prompt tells the model **everything** the generator can typeset, so it
uses tables, diagrams and charts instead of plain paragraphs. The paper comes
out in Spanish, because the template produces a Spanish APA document.

> For the model to see a real example of each element, also attach
> [`examples/catalog.md`](examples/catalog.md).

---

## The prompt

````text
You are going to write an academic paper in Markdown. A generator turns that
Markdown into a PDF in APA 7 format, so the format matters as much as the
content. Follow these rules exactly.

# LANGUAGE

- Write the whole paper in Spanish, including headings, captions and
  references.

# STRUCTURE

- Start directly with "# Introducción". Do NOT write a cover page, table of
  contents, paper title, student name, course, date or page headers: the
  generator adds all of that.
- Use exactly these four level-1 headings, in this order:
  # Introducción
  # Desarrollo
  # Conclusión
  # Referencias
- Inside "# Desarrollo", organize the subtopics with ## and ###.
- Heading levels map to APA levels: # is level 1 (centered, bold), ## level 2,
  ### level 3, #### level 4 and ##### level 5. Do not skip levels.

# FORMATTING RULES THAT ARE NOT NEGOTIABLE

- Separate EVERY block with a real blank line: before and after each heading,
  list, table, quote, image, diagram and code block.
- Do not use non-breaking spaces (U+00A0) or other invisible spaces. Only the
  normal space. Do not leave spaces at the end of lines.
- To break a line inside a paragraph, use <br>.
- Write complete, well-connected paragraphs. Do not turn the whole paper into
  bullet lists.
- Do not write stray LaTeX commands in the text, and do not use the backslash
  outside formulas and the blocks described below.

# AVAILABLE SYNTAX

Text: **bold**, *italic*, ***both***, `code`, ~~strikethrough~~,
==highlight==, subscript H~2~O, superscript X^2^.

Links: [text](https://example.com) or a bare URL, which becomes a link.

Footnotes: a mark[^1] in the text and, on another line, [^1]: the content.

Lists:
- bullets, nested with two spaces
1. numbered
- [x] tasks, with [x] done and [ ] pending
Term
: definition, for a glossary

Block quotes:
> The quoted text, with the greater-than sign.

Note boxes, for a definition or a warning:
::: nota
The content of the box.
:::
Types: nota, aviso, importante, ejemplo, definicion. The title can be changed
with ::: {.aviso title="Antes de entregar"}

Tables (the row of dashes is required; colons set the alignment):
| Columna | Centrada | A la derecha |
|---------|:--------:|-------------:|
| texto   | texto    | 3.14         |

Code, with the language so it gets syntax colors:
```python
def ejemplo():
    return True
```

Formulas: inline with $E = mc^2$ and as a block with $$ ... $$ on its own
line. You can use \sum, \frac, \int, \begin{bmatrix}, \begin{cases} and so on.

Symbols: write ≠ ≤ ≥ ≈ ∈ ∑ √ ∞ π Δ α β directly in the text.

Images, if the user gives you a path or an address:
![Texto alternativo](imagen.png){width=60%}

# DIAGRAMS

For a schema (hierarchies, flows, processes, graphs, automata, data models),
use a Graphviz block. It is drawn as a vector image:

```{.dot caption="Pie de la figura"}
digraph {
  graph [rankdir=LR];
  node  [shape=box, style=rounded, fontname="Helvetica", fontsize=10];

  "Requisitos" -> "Diseño" -> "Pruebas";
}
```

Use "digraph" with -> for directed graphs and "graph" with -- for undirected
ones. You have shape=box/ellipse/diamond/circle/doublecircle/record, subgraph
cluster_X to group nodes, layout=neato for networks and edge labels with
[label="texto"].

For a very simple schema you may also draw with box characters (└ ─ ┼ │ ►),
but ALWAYS inside a code block without a language, because outside it the
drawing falls apart.

# DATA CHARTS

When there are figures to compare, use a pgfplots chart. Careful: this is real
LaTeX and a syntax error stops the build, so check it mentally before writing
it.

```{.pgfplot caption="Pie de la figura"}
\begin{axis}[
  ybar, ymin=0, ylabel={Horas},
  symbolic x coords={Análisis,Diseño,Código}, xtick=data]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180)};
\end{axis}
```

Types you can use, by changing the options of the axis environment:
- ybar (vertical bars), xbar (horizontal), ybar stacked (stacked)
- several series with repeated \addplot and \legend{Serie A, Serie B}
- lines: without ybar, with \addplot coordinates {...}
- scatter: \addplot[only marks] table[x=col, y=col] { ... }
- linear regression: y={create col/linear regression={y=col}}
- histogram: \addplot[hist={bins=6}] table[y index=0] { ... }
- box plot: boxplot/draw direction=y and boxplot prepared={...}
- error bars: error bars/.cd, y dir=both, y explicit
- functions: domain=1:50, samples=80 and \addplot[mark=none] {x^2}
- log scale: use semilogyaxis instead of axis
- pie: \pie[text=legend]{40/Parte A, 60/Parte B}

Do not set colors: the template already uses a sober grayscale palette suitable
for APA.

# REFERENCES

Under "# Referencias", write the sources in APA 7 format, one per line, as a
dash list. The generator adds the hanging indent. Cite in the text with the
author-date format, for example (Cormen et al., 2022).

Do not invent sources. If you are not sure about a reference, say so instead
of making it up.

# JUDGMENT

Use each element when it adds something: a table to compare, a diagram for a
structure or a process, a chart for figures, a box for an important
definition. Do not add decorative elements or emoji to an academic paper.

# DELIVERY

Return only the content of the .md file, with no explanation before or after
and without wrapping it in a code block.

The topic of the paper is:
````

---

## How to use it

1. Copy the whole block.
2. Add the topic, the length you need and your teacher's instructions at the
   end. For example: *"Software development life cycles, about eight pages,
   comparing at least four models."*
3. Save the answer in `input/my-paper.md`.
4. Generate the PDF and read the warnings, if any.

## If the result has no diagrams or charts

Models tend to write only paragraphs. Ask for them explicitly:

> Include at least two Graphviz diagrams and one pgfplots chart with real data
> about the topic.

## If the build fails

The only element that can stop the build is a badly written `pgfplot` chart.
The error message gives the line. Copy the error and ask the model to fix that
block, or delete it; the rest of the paper is generated anyway.
