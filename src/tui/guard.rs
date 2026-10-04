//! Red de seguridad del menú: un pánico al atender una tecla, al dibujar o en
//! el hilo que genera el PDF se registra y se convierte en un mensaje, en vez
//! de cerrar el programa con la terminal a medias.

use std::any::Any;
use std::backtrace::Backtrace;
use std::cell::Cell;
use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::logging;

thread_local! {
    /// Si el hilo está dentro de `run`: ahí el pánico se atrapa después.
    static GUARDED: Cell<bool> = const { Cell::new(false) };
}

/// Ejecuta `work`; si entra en pánico devuelve el mensaje del pánico.
pub fn run<T>(work: impl FnOnce() -> T) -> Result<T, String> {
    let was_guarded = GUARDED.replace(true);
    let result = catch_unwind(AssertUnwindSafe(work));
    GUARDED.set(was_guarded);
    result.map_err(|payload| panic_message(payload.as_ref()))
}

/// El texto que trae un pánico (`panic!("…")` o un `String`).
pub fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "?".to_owned())
}

/// Va DESPUÉS de `ratatui::init()`. Dentro de `run` el pánico solo se
/// registra: la terminal sigue en uso y restaurarla rompería el menú. Fuera,
/// sigue la cadena de siempre (ratatui restaura, el registro lo anota y Rust
/// imprime el mensaje).
pub fn install_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if GUARDED.get() {
            let thread = std::thread::current();
            let name = thread.name().unwrap_or("?");
            let backtrace = Backtrace::force_capture();
            logging::error(logging::describe_panic(info.payload(), info.location(), name, &backtrace));
        } else {
            previous(info);
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panic_becomes_its_message_and_the_flag_is_restored() {
        assert_eq!(run(|| 2 + 2), Ok(4));
        let error = run(|| -> u8 { panic!("se rompió {}", 7) }).unwrap_err();
        assert_eq!(error, "se rompió 7");
        assert!(!GUARDED.get());
        // Anidado: el de afuera sigue protegido después del de adentro.
        let outer = run(|| {
            let _ = run(|| panic!("adentro"));
            assert!(GUARDED.get());
        });
        assert!(outer.is_ok());
    }
}
