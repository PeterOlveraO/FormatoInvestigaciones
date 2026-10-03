# Prompt para pedirle el trabajo a una IA

Este archivo contiene un prompt listo para copiar. Pégalo en ChatGPT, Claude,
Gemini o el modelo que uses, añade abajo el tema y las indicaciones de tu
profesor, y lo que te devuelva se guarda tal cual en un `.md` dentro de `input/`
y se genera con:

```bash
investigacion mi-trabajo.md --title "El tema" --subject "La materia"
```

El prompt le explica al modelo **todo** lo que el generador sabe componer, para
que aproveche las tablas, los diagramas y las gráficas en vez de entregar
párrafos sueltos.

> Si quieres que además vea ejemplos reales de cada elemento, adjunta también
> [`ejemplo/catalogo.md`](ejemplo/catalogo.md).

---

## El prompt

````text
Vas a redactar un trabajo académico en Markdown. Ese Markdown lo procesa un
generador que lo convierte en un PDF con formato APA 7, así que el formato
importa tanto como el contenido. Sigue estas reglas al pie de la letra.

# ESTRUCTURA

- Empieza directamente en "# Introducción". NO escribas portada, índice, título
  del trabajo, nombre del alumno, materia, fecha ni encabezados de página: todo
  eso lo añade el generador.
- Usa exactamente estos cuatro encabezados de nivel 1, en este orden:
  # Introducción
  # Desarrollo
  # Conclusión
  # Referencias
- Dentro de "# Desarrollo" organiza los subtemas con ## y ###.
- Los niveles de encabezado equivalen a los de APA: # es nivel 1 (centrado y en
  negrita), ## nivel 2, ### nivel 3, #### nivel 4 y ##### nivel 5. No te saltes
  niveles.

# REGLAS DE FORMATO QUE NO SE NEGOCIAN

- Separa TODO bloque con una línea en blanco real: antes y después de cada
  encabezado, lista, tabla, cita, imagen, diagrama y bloque de código.
- No uses espacios duros (U+00A0) ni otros espacios invisibles. Solo el espacio
  normal. No dejes espacios al final de los renglones.
- Para cortar un renglón dentro de un párrafo usa <br>.
- Escribe párrafos completos y bien hilados. No conviertas todo el trabajo en
  listas de viñetas.
- No escribas comandos de LaTeX sueltos en el texto ni uses la contrabarra
  fuera de las fórmulas y de los bloques que se indican más abajo.

# SINTAXIS DISPONIBLE

Texto: **negrita**, *cursiva*, ***ambas***, `código`, ~~tachado~~,
==resaltado==, subíndice H~2~O, superíndice X^2^.

Enlaces: [texto](https://ejemplo.com) o una URL suelta, que se convierte sola.

Notas al pie: una marca[^1] en el texto y, en otra línea, [^1]: el contenido.

Listas:
- con viñetas, anidando con dos espacios
1. numeradas
- [x] de tareas, con [x] hecha y [ ] pendiente
Término
: definición, para un glosario

Citas en bloque:
> El texto citado, con el signo de mayor que.

Cajas destacadas, para una definición o una advertencia:
::: nota
El contenido de la caja.
:::
Tipos: nota, aviso, importante, ejemplo, definicion. El título se puede cambiar
con ::: {.aviso title="Antes de entregar"}

Tablas (la fila de guiones es obligatoria; los dos puntos alinean):
| Columna | Centrada | A la derecha |
|---------|:--------:|-------------:|
| texto   | texto    | 3.14         |

Código, indicando el lenguaje para que salga con colores:
```python
def ejemplo():
    return True
```

Fórmulas: en línea con $E = mc^2$ y en bloque con $$ ... $$ en su propio
renglón. Puedes usar \sum, \frac, \int, \begin{bmatrix}, \begin{cases} y demás.

Símbolos: escribe ≠ ≤ ≥ ≈ ∈ ∑ √ ∞ π Δ α β directamente en el texto.

Imágenes, si el usuario te da una ruta o una dirección:
![Texto alternativo](imagen.png){width=60%}

# DIAGRAMAS

Para un esquema —jerarquías, flujos, procesos, grafos, autómatas, modelos de
datos— usa un bloque de Graphviz. Se dibuja como imagen vectorial:

```{.dot caption="Pie de la figura"}
digraph {
  graph [rankdir=LR];
  node  [shape=box, style=rounded, fontname="Helvetica", fontsize=10];

  "Requisitos" -> "Diseño" -> "Pruebas";
}
```

Usa "digraph" con -> para grafos dirigidos y "graph" con -- para no dirigidos.
Dispones de shape=box/ellipse/diamond/circle/doublecircle/record, subgraph
cluster_X para agrupar, layout=neato para redes y etiquetas en las aristas con
[label="texto"].

Si el esquema es muy simple, también puedes dibujarlo con caracteres de caja
(└ ─ ┼ │ ►), pero SIEMPRE dentro de un bloque de código sin lenguaje, porque
fuera de él se desalinea.

# GRÁFICAS DE DATOS

Cuando haya cifras que comparar, usa una gráfica de pgfplots. Ojo: esto es
LaTeX de verdad y un error de sintaxis detiene la compilación, así que
compruébala mentalmente antes de escribirla.

```{.pgfplot caption="Pie de la figura"}
\begin{axis}[
  ybar, ymin=0, ylabel={Horas},
  symbolic x coords={Análisis,Diseño,Código}, xtick=data]
  \addplot coordinates {(Análisis,120) (Diseño,95) (Código,180)};
\end{axis}
```

Tipos que puedes usar, cambiando las opciones del entorno axis:
- ybar (barras verticales), xbar (horizontales), ybar stacked (apiladas)
- varias series con \addplot repetido y \legend{Serie A, Serie B}
- líneas: sin ybar, con \addplot coordinates {...}
- dispersión: \addplot[only marks] table[x=col, y=col] { ... }
- regresión lineal: y={create col/linear regression={y=col}}
- histograma: \addplot[hist={bins=6}] table[y index=0] { ... }
- caja y bigotes: boxplot/draw direction=y y boxplot prepared={...}
- barras de error: error bars/.cd, y dir=both, y explicit
- funciones: domain=1:50, samples=80 y \addplot[mark=none] {x^2}
- escala logarítmica: usa semilogyaxis en vez de axis
- pastel: \pie[text=legend]{40/Parte A, 60/Parte B}

No pongas colores: la plantilla ya usa una paleta sobria en escala de grises,
adecuada para APA.

# REFERENCIAS

En "# Referencias" escribe las fuentes en formato APA 7, una por línea, como
lista con guiones. El generador les da la sangría francesa. Cita en el texto con
el formato autor-fecha, por ejemplo (Cormen et al., 2022).

No inventes fuentes. Si no tienes certeza de una referencia, dilo en vez de
fabricarla.

# CRITERIO

Usa cada elemento cuando aporte algo: una tabla para comparar, un diagrama para
una estructura o un proceso, una gráfica para cifras, una caja para una
definición importante. No metas elementos de adorno ni emoji en un trabajo
académico.

# ENTREGA

Devuelve únicamente el contenido del archivo .md, sin explicaciones antes ni
después y sin envolverlo en un bloque de código.

El tema del trabajo es:
````

---

## Cómo usarlo

1. Copia el bloque completo.
2. Añade al final el tema, la extensión que necesitas y las indicaciones de tu
   profesor. Por ejemplo: *«Los ciclos de vida del desarrollo de software, unas
   ocho páginas, con una comparación de al menos cuatro modelos»*.
3. Guarda la respuesta en `input/mi-trabajo.md`.
4. Genera el PDF y revisa las advertencias, si las hay.

## Si el resultado no usa diagramas ni gráficas

Los modelos tienden a escribir solo párrafos. Pídelo explícitamente:

> Incluye al menos dos diagramas de Graphviz y una gráfica de pgfplots con
> datos reales del tema.

## Si la compilación falla

El único elemento que puede detener la compilación es una gráfica `pgfplot` mal
escrita; el comando te dirá en qué línea. Copia el error y pídele al modelo que
corrija ese bloque, o bórralo: el resto del trabajo se genera igual.
