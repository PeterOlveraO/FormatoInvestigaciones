# Introducción

Un **árbol binario de búsqueda** (ABB) es una estructura de datos no lineal que
organiza un conjunto de claves de forma que buscar, insertar y eliminar tomen,
en el caso favorable, un tiempo proporcional al logaritmo del número de
elementos. Esa propiedad es la que explica su presencia en los índices de las
bases de datos, en los diccionarios ordenados de las bibliotecas estándar y en
los sistemas de archivos.

La idea que lo sostiene es una sola: **el orden está en la forma del árbol, no
en el recorrido**. Cada nodo divide el conjunto restante en dos mitades, las
menores a la izquierda y las mayores a la derecha, de modo que cada comparación
descarta una parte del problema en lugar de avanzar un elemento.

Este trabajo recorre cuatro aspectos del ABB: su definición y propiedades, las
operaciones básicas con su costo, los cuatro recorridos clásicos y el problema
del equilibrio, que es donde la estructura pasa de ser una idea elegante a una
herramienta confiable.

# Desarrollo

## Definición y propiedades

Un árbol binario es un conjunto finito de nodos que o bien está vacío, o bien
consta de una raíz y dos subárboles binarios disjuntos llamados izquierdo y
derecho. Se convierte en árbol **de búsqueda** cuando cumple, para todo nodo
$x$, la siguiente invariante:

$$\text{clave}(i) \leq \text{clave}(x) \leq \text{clave}(d)
\quad \forall\, i \in \text{izq}(x),\; \forall\, d \in \text{der}(x)$$

Es decir, todas las claves del subárbol izquierdo son menores o iguales que la
del nodo, y todas las del derecho son mayores o iguales.

::: aviso
La condición se aplica a los **subárboles completos**, no solo a los hijos
inmediatos. Un árbol donde cada nodo cumple la regla con sus dos hijos puede
seguir sin ser un ABB, y ese es el error más común al aprender la estructura.
:::

El siguiente árbol cumple la invariante:

```{.dot caption="Árbol binario de búsqueda con ocho claves"}
digraph {
  graph [ranksep=0.35, nodesep=0.30];
  node  [shape=circle, fixedsize=true, width=0.42, fontname="Helvetica", fontsize=11];
  edge  [arrowsize=0.6];

  50 -> 30; 50 -> 70;
  30 -> 20; 30 -> 40;
  70 -> 60; 70 -> 80;
  20 -> 10;
}
```

Conviene fijar tres términos que se usan en el resto del trabajo:

Altura
: Número de aristas del camino más largo de la raíz a una hoja. El árbol del
  ejemplo tiene altura 3.

Nodo hoja
: Nodo sin hijos. En el ejemplo son 10, 40, 60 y 80.

Factor de equilibrio
: Diferencia entre la altura del subárbol izquierdo y la del derecho.

La altura $h$ es la magnitud que gobierna el costo de todas las operaciones,
porque ninguna recorre más de un camino de la raíz a una hoja. Para un árbol de
$n$ nodos se cumple:

$$\lfloor \log_2 n \rfloor \leq h \leq n - 1$$

El extremo izquierdo corresponde al árbol perfectamente equilibrado y el derecho
al árbol degenerado, que es una lista enlazada disfrazada. ==Toda la diferencia
entre una estructura logarítmica y una lineal está en ese rango==, y por eso el
equilibrio no es un adorno sino el punto central del diseño.

## Operaciones básicas

### Búsqueda

La búsqueda compara la clave buscada con la del nodo actual y desciende hacia un
solo lado, descartando el otro subárbol completo:

```python
def buscar(nodo, clave):
    """Devuelve el nodo con la clave, o None si no está en el árbol."""
    if nodo is None or nodo.clave == clave:
        return nodo
    if clave < nodo.clave:
        return buscar(nodo.izquierdo, clave)
    return buscar(nodo.derecho, clave)
```

Buscar el 40 en el árbol anterior recorre tres nodos en lugar de los ocho que
exigiría una lista:

```
   50  ──►  clave < 50, bajar a la izquierda
    │
   30  ──►  clave > 30, bajar a la derecha
    │
   40  ──►  encontrada
```

### Inserción

La inserción busca la clave y, al llegar a un enlace vacío, coloca ahí el nodo
nuevo. Esto significa que **un ABB siempre inserta en una hoja** y que la forma
final depende por completo del orden de llegada de los datos. Insertar las
claves 10, 20, 30, 40 en ese orden produce un árbol degenerado:

```{.dot caption="El mismo conjunto insertado en orden: una lista disfrazada"}
digraph {
  graph [ranksep=0.30];
  node  [shape=circle, fixedsize=true, width=0.42, fontname="Helvetica", fontsize=11];
  edge  [arrowsize=0.6];

  10 -> 20 -> 30 -> 40;
}
```

### Eliminación

Eliminar es la única operación con casos, y son tres:

1. **El nodo es una hoja.** Se desengancha y ya está.
2. **El nodo tiene un solo hijo.** El hijo ocupa su lugar.
3. **El nodo tiene dos hijos.** Se sustituye por su *sucesor inorden* —la clave
   menor del subárbol derecho— y se elimina ese sucesor, que por construcción
   cae en alguno de los dos casos anteriores.

El tercer caso sobre el árbol del ejemplo, eliminando la raíz:

```{.dot caption="Eliminación de la raíz: la sustituye su sucesor inorden"}
digraph {
  graph [ranksep=0.35, nodesep=0.25];
  node  [shape=circle, fixedsize=true, width=0.42, fontname="Helvetica", fontsize=11];
  edge  [arrowsize=0.6];

  subgraph cluster_antes {
    label = "Antes";
    fontname = "Helvetica"; fontsize = 11; color = gray60;
    a50 [label="50"]; a30 [label="30"]; a70 [label="70"];
    a20 [label="20"]; a40 [label="40"]; a60 [label="60"]; a80 [label="80"];
    a50 -> a30; a50 -> a70;
    a30 -> a20; a30 -> a40;
    a70 -> a60; a70 -> a80;
  }

  subgraph cluster_despues {
    label = "Después";
    fontname = "Helvetica"; fontsize = 11; color = gray60;
    d60 [label="60"]; d30 [label="30"]; d70 [label="70"];
    d20 [label="20"]; d40 [label="40"]; d80 [label="80"];
    d60 -> d30; d60 -> d70;
    d30 -> d20; d30 -> d40;
    d70 -> d80;
  }
}
```

El 60 es el menor del subárbol derecho, así que ocupar la raíz con él conserva
la invariante sin tocar el resto del árbol.

### Costo de las operaciones

Las tres operaciones tienen el mismo perfil, porque las tres recorren un camino
de la raíz hacia abajo:

| Operación | Árbol equilibrado | Árbol degenerado | Espacio |
|---|:---:|:---:|:---:|
| Búsqueda | Θ(log n) | Θ(n) | Θ(1) |
| Inserción | Θ(log n) | Θ(n) | Θ(1) |
| Eliminación | Θ(log n) | Θ(n) | Θ(1) |
| Mínimo o máximo | Θ(log n) | Θ(n) | Θ(1) |
| Recorrido completo | Θ(n) | Θ(n) | Θ(h) |

Cormen et al. (2022) demuestran además que un ABB construido insertando $n$
claves distintas en orden aleatorio tiene altura esperada $O(\log n)$, lo que
explica que la estructura funcione razonablemente bien en la práctica aun sin
mecanismo de equilibrio[^aleatorio].

[^aleatorio]: El resultado depende de que el orden de llegada sea aleatorio. Con
datos ya ordenados —el caso más frecuente en un sistema real, porque las claves
suelen venir de un identificador incremental— el árbol degenera siempre.

## Los recorridos

Un recorrido visita todos los nodos siguiendo un criterio. Los tres primeros son
en profundidad y se distinguen solo por el momento en que se procesa la raíz:

| Recorrido | Orden | Resultado en el árbol de ejemplo |
|---|---|---|
| Inorden | izquierda, **raíz**, derecha | 10, 20, 30, 40, 50, 60, 70, 80 |
| Preorden | **raíz**, izquierda, derecha | 50, 30, 20, 10, 40, 70, 60, 80 |
| Postorden | izquierda, derecha, **raíz** | 10, 20, 40, 30, 60, 80, 70, 50 |
| Por niveles | de arriba abajo, de izquierda a derecha | 50, 30, 70, 20, 40, 60, 80, 10 |

El inorden es el importante: sobre un ABB **devuelve las claves ordenadas**, y
esa es la razón de que la estructura sirva para consultas por rango.

```python
def inorden(nodo, visitar):
    """Recorre el árbol de menor a mayor clave."""
    if nodo is None:
        return
    inorden(nodo.izquierdo, visitar)
    visitar(nodo.clave)
    inorden(nodo.derecho, visitar)
```

El recorrido por niveles es el único que no es recursivo por naturaleza: se
implementa con una cola, sacando un nodo y encolando sus hijos.

> Los recorridos en profundidad usan una pila —explícita o la de llamadas— y los
> recorridos en anchura usan una cola. Esa dualidad es la misma que separa a la
> búsqueda en profundidad de la búsqueda en anchura sobre grafos, de los que el
> árbol no es más que un caso particular sin ciclos.

## El problema del equilibrio

Un ABB sin control degenera con datos ordenados, y los datos reales llegan
ordenados más veces de las que uno quisiera. La solución son los **árboles
autoequilibrados**, que reacomodan los nodos después de cada modificación para
mantener la altura en $O(\log n)$.

### Árboles AVL

Propuestos por Adelson-Velsky y Landis (1962), fueron la primera estructura de
este tipo. Su invariante es estricta: el factor de equilibrio de cada nodo debe
valer −1, 0 o 1. Cuando una inserción lo rompe, el árbol se repara con una
rotación, que es un intercambio local de tres enlaces:

```{.dot caption="Rotación derecha: la altura baja sin alterar el recorrido inorden"}
digraph {
  graph [ranksep=0.32, nodesep=0.22];
  node  [shape=circle, fixedsize=true, width=0.40, fontname="Helvetica", fontsize=11];
  edge  [arrowsize=0.55];

  subgraph cluster_antes {
    label = "Desequilibrado";
    fontname = "Helvetica"; fontsize = 11; color = gray60;
    az [label="z"]; ay [label="y"]; ax [label="x"];
    a1 [label="T1", shape=box, width=0.36, height=0.30];
    a2 [label="T2", shape=box, width=0.36, height=0.30];
    a3 [label="T3", shape=box, width=0.36, height=0.30];
    a4 [label="T4", shape=box, width=0.36, height=0.30];
    az -> ay; az -> a4;
    ay -> ax; ay -> a3;
    ax -> a1; ax -> a2;
  }

  subgraph cluster_despues {
    label = "Tras la rotación";
    fontname = "Helvetica"; fontsize = 11; color = gray60;
    by [label="y"]; bx [label="x"]; bz [label="z"];
    b1 [label="T1", shape=box, width=0.36, height=0.30];
    b2 [label="T2", shape=box, width=0.36, height=0.30];
    b3 [label="T3", shape=box, width=0.36, height=0.30];
    b4 [label="T4", shape=box, width=0.36, height=0.30];
    by -> bx; by -> bz;
    bx -> b1; bx -> b2;
    bz -> b3; bz -> b4;
  }
}
```

La rotación conserva el recorrido inorden —y por lo tanto la invariante del
ABB— pero reduce la altura del subárbol en uno. Hay cuatro casos según dónde se
produjo el desequilibrio:

| Caso | Dónde se insertó | Reparación |
|---|---|---|
| Izquierda-izquierda | Subárbol izq. del hijo izq. | Rotación derecha |
| Derecha-derecha | Subárbol der. del hijo der. | Rotación izquierda |
| Izquierda-derecha | Subárbol der. del hijo izq. | Rotación izquierda y luego derecha |
| Derecha-izquierda | Subárbol izq. del hijo der. | Rotación derecha y luego izquierda |

### Árboles rojo-negro

Relajan la condición: en vez de exigir alturas casi iguales, colorean los nodos
y garantizan que ningún camino de la raíz a una hoja sea más del doble de largo
que otro. El árbol resultante queda menos equilibrado que un AVL, pero necesita
menos rotaciones al modificarse, y por eso es el que usan casi todas las
bibliotecas estándar.

```
   Comparación de estrategias

   ├── AVL ──────────► equilibrio estricto
   │                   búsquedas más rápidas
   │                   más rotaciones al escribir
   │
   └── Rojo-negro ───► equilibrio aproximado
                       escrituras más baratas
                       el de uso general
```

::: nota
La regla práctica: AVL cuando se lee mucho más de lo que se escribe, rojo-negro
cuando la carga está repartida. Si la estructura se construye una vez y después
solo se consulta, el AVL gana; si se modifica constantemente, el rojo-negro.
:::

### Dónde queda el ABB simple

| Estructura | Búsqueda | Mantiene orden | Complejidad de implementar |
|---|:---:|:---:|---|
| Lista enlazada | Θ(n) | Sí | Trivial |
| Tabla hash | Θ(1) promedio | ~~Sí~~ No | Media |
| ABB sin equilibrio | Θ(n) peor caso | Sí | Baja |
| AVL o rojo-negro | Θ(log n) garantizado | Sí | Alta |

La tabla hash gana en búsqueda por clave exacta, pero pierde el orden: no puede
responder «dame todas las claves entre 20 y 60» sin recorrerlo todo. Ahí es
donde el árbol sigue siendo insustituible.

### Lista de comprobación al elegir

- [x] ¿Se necesitan consultas por rango o recorrer en orden? → árbol
- [x] ¿Las claves llegan ordenadas o casi ordenadas? → árbol autoequilibrado
- [ ] ¿Solo se busca por clave exacta? → tabla hash
- [ ] ¿Los datos no caben en memoria? → árbol B, no binario

# Conclusión

El árbol binario de búsqueda convierte el orden de los datos en una forma, y esa
forma es lo que permite descartar la mitad del problema en cada comparación. Su
costo depende por completo de la altura, y la altura depende del orden en que
llegaron las claves: el mismo conjunto puede producir un árbol logarítmico o una
lista enlazada disfrazada.

De ahí que la versión útil en producción no sea el ABB básico sino su variante
autoequilibrada. Los AVL pagan rotaciones para conservar un equilibrio estricto
y los rojo-negro aceptan un árbol más desordenado a cambio de escrituras más
baratas; la elección entre ambos depende de la proporción entre lecturas y
escrituras, no de cuál sea «mejor».

Queda fuera de este trabajo el caso en que los datos no caben en memoria, donde
la unidad de costo deja de ser la comparación y pasa a ser el acceso a disco.
Ahí la respuesta son los árboles B, que agrupan muchas claves por nodo para
reducir el número de accesos; es la estructura que usan los índices de las bases
de datos relacionales.

# Referencias

- Adelson-Velsky, G. M., & Landis, E. M. (1962). An algorithm for the
  organization of information. *Soviet Mathematics Doklady, 3*, 1259-1263.

- Cormen, T. H., Leiserson, C. E., Rivest, R. L., & Stein, C. (2022).
  *Introduction to algorithms* (4a ed.). MIT Press.

- Knuth, D. E. (1998). *The art of computer programming: Vol. 3. Sorting and
  searching* (2a ed.). Addison-Wesley.

- Sedgewick, R., & Wayne, K. (2011). *Algorithms* (4a ed.). Addison-Wesley.

- Weiss, M. A. (2014). *Data structures and algorithm analysis in C++* (4a ed.).
  Pearson.
