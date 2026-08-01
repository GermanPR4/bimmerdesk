//! Typed command bus: modules register a handler for a concrete command
//! type; callers dispatch that concrete type and get its typed output back.
//! No stringly-typed routing — see PROJECT_PLAN.md 2 ("Comunicación entre
//! módulos": tipado, no genérico por strings) and PROJECT_PRINCIPLES.md.
//!
//! Internally this uses `TypeId`/`Any` to store heterogeneous handlers in one
//! map, but that's an implementation detail: every call site is fully typed
//! (`bus.dispatch::<ScanDtcsCommand>(cmd)` returns `ScanDtcsCommand::Output`,
//! checked by the compiler, not by a string key).

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Marker for anything that can travel through the bus. `Output` is the
/// concrete response type for that specific command.
pub trait Command: Any + Send + 'static {
    type Output: Send + 'static;
}

#[derive(Debug, PartialEq, Eq)]
pub enum BusError {
    NoHandler { command: &'static str },
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusError::NoHandler { command } => {
                write!(f, "no handler registered for command '{command}'")
            }
        }
    }
}

impl std::error::Error for BusError {}

type Handler = Box<dyn Fn(Box<dyn Any + Send>) -> Box<dyn Any + Send> + Send + Sync>;

#[derive(Default)]
pub struct CommandBus {
    handlers: HashMap<TypeId, Handler>,
}

impl CommandBus {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// Registers the handler a module exposes for `C`. Re-registering the
    /// same command type overwrites the previous handler — that should only
    /// ever happen deliberately, never as two modules silently colliding.
    pub fn register<C, F>(&mut self, handler: F)
    where
        C: Command,
        F: Fn(C) -> C::Output + Send + Sync + 'static,
    {
        let boxed: Handler = Box::new(move |cmd: Box<dyn Any + Send>| {
            let cmd = *cmd
                .downcast::<C>()
                .expect("command type mismatch on dispatch — bus bug, not caller error");
            Box::new(handler(cmd))
        });
        self.handlers.insert(TypeId::of::<C>(), boxed);
    }

    pub fn dispatch<C: Command>(&self, command: C) -> Result<C::Output, BusError> {
        let handler = self.handlers.get(&TypeId::of::<C>()).ok_or(BusError::NoHandler {
            command: std::any::type_name::<C>(),
        })?;
        let boxed_output = handler(Box::new(command));
        Ok(*boxed_output
            .downcast::<C::Output>()
            .expect("command output type mismatch — bus bug, not caller error"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Ping(u32);

    impl Command for Ping {
        type Output = u32;
    }

    #[test]
    fn dispatches_to_registered_handler() {
        let mut bus = CommandBus::new();
        bus.register::<Ping, _>(|cmd| cmd.0 + 1);

        let result = bus.dispatch(Ping(41)).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn returns_error_when_no_handler_registered() {
        let bus = CommandBus::new();
        let result = bus.dispatch(Ping(1));
        assert!(matches!(result, Err(BusError::NoHandler { .. })));
    }
}
