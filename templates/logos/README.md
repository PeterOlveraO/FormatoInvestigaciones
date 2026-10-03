# Logos de la portada

Esta carpeta está en `.gitignore`: los logos de una institución rara vez son
redistribuibles, así que **no se suben al repositorio**. Cada quien pone aquí
los suyos.

## Qué archivos poner

Dos imágenes, con estos nombres exactos:

| Archivo | Dónde sale | Qué suele ser |
|---|---|---|
| `logo-universidad.png` | Arriba a la izquierda | El escudo de la universidad |
| `logo-facultad.png` | Arriba a la derecha | El logotipo de la facultad o escuela |

Los dos son **opcionales**: si falta uno, o los dos, la portada se compila igual
y simplemente no aparecen.

## Cómo deben ser

- **PNG**, preferiblemente con fondo transparente.
- **Oscuros o de color.** La portada es blanca, así que un logo blanco sería
  invisible; si el tuyo es blanco, conviértelo antes a una versión oscura.
- Sin margen de más: la plantilla los escala por altura (1.6 cm el de la
  universidad y 0.88 cm el de la facultad), y un margen transparente grande los
  hace verse pequeños y descolocados.

## Si prefieres tenerlos en otro sitio

No hace falta copiarlos aquí. Puedes guardarlos donde quieras e indicarlo al
generar:

```bash
investigacion trabajo.md --titulo "Tema" --materia "M" --docente "D" \
  --logos ~/Documentos/logos-de-mi-universidad
```

O dejarlo fijo en el `.env` para no repetirlo en cada trabajo:

```dotenv
LOGOS="/home/usuario/Documentos/logos-de-mi-universidad"
```

## Si quedan descolocados

Las posiciones y alturas están en el bloque `LOGOS` de `templates/apa/template.ltx`, dentro
de la portada. Son dos `\node` de TikZ con su `xshift`, `yshift` y `height`.
