# Perfiles de materia

Cada archivo `.toml` de esta carpeta guarda los datos que se repiten en todos
los trabajos de una materia, para no escribirlos cada vez:

| Campo      | Qué es                                                | ¿Obligatorio? |
|------------|-------------------------------------------------------|---------------|
| `subject`  | Nombre de la materia (sale en la portada)             | Sí            |
| `teacher`  | Docente                                               | No            |
| `group`    | Grupo                                                 | No            |
| `members`  | Integrantes del equipo, separados por comas           | No            |
| `template` | Plantilla de `templates/` (`apa`, `apa-simple`…)      | No (`apa`)    |
| `folder`   | Subcarpeta de `input/` y de `output/` de la materia   | No            |

El nombre del archivo es la clave del perfil: `ia.toml` se usa con
`-p ia` (o `--subject-profile ia`), y en el menú interactivo aparece en la
lista de perfiles.

Prioridad de los datos: lo que se escriba en el comando gana sobre el perfil, y
el perfil gana sobre el `.env`.

Esta carpeta está en `.gitignore` salvo `example.toml` y este archivo: los
nombres de tus docentes son datos tuyos y no se suben al repositorio.
