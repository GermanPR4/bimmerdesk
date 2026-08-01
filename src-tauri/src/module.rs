//! Core module system: every functional area of the app (Vehicle Info, Live
//! Data, Diagnostics...) implements `Module` and is registered at compile
//! time in `ModuleRegistry` — no runtime plugin loading. See
//! PROJECT_PLAN.md sections 3.2 and 5, and PROJECT_PRINCIPLES.md for why
//! (Rust has no stable ABI, dynamic `.dll` loading is fragile).

use std::fmt;

/// Capability a module can declare it needs. The registry validates these at
/// startup so what a module can touch (ECU, network, filesystem...) is
/// explicit and auditable — not sandboxed in a separate process, just
/// declarative control (see PROJECT_PLAN.md 2, "Permisos entre módulos").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    Ecu,
    Sqlite,
    Network,
    Filesystem,
    Ai,
    Usb,
}

#[derive(Debug, Clone)]
pub struct ModuleManifest {
    pub id: &'static str,
    pub version: &'static str,
    pub capabilities: &'static [Capability],
}

#[derive(Debug)]
pub enum ModuleError {
    StartupFailed { module_id: &'static str, reason: String },
}

impl fmt::Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::StartupFailed { module_id, reason } => {
                write!(f, "module '{module_id}' failed to start: {reason}")
            }
        }
    }
}

impl std::error::Error for ModuleError {}

/// Shared context handed to every module at startup. Kept intentionally
/// minimal in Fase 0 — grows only when a real module needs something
/// (DB pool, telemetry handle...), never speculatively.
pub struct AppContext {}

pub trait Module: Send + Sync {
    fn manifest(&self) -> ModuleManifest;

    /// Registers this module's commands on the bus and any public API it
    /// exposes to other modules. A no-op body is valid for stub modules
    /// (Coding, ECU Flash in V1).
    fn register(&self, bus: &mut crate::command_bus::CommandBus);

    /// Called once at application startup, after registration.
    fn on_startup(&self, ctx: &AppContext) -> Result<(), ModuleError>;
}

/// Compile-time registry: modules are listed explicitly (in `lib.rs`), never
/// discovered at runtime.
#[derive(Default)]
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self { modules: Vec::new() }
    }

    pub fn register(&mut self, module: Box<dyn Module>) -> &mut Self {
        self.modules.push(module);
        self
    }

    pub fn manifests(&self) -> Vec<ModuleManifest> {
        self.modules.iter().map(|m| m.manifest()).collect()
    }

    /// Wires every registered module onto the bus and runs its startup hook.
    pub fn start_all(
        &self,
        bus: &mut crate::command_bus::CommandBus,
        ctx: &AppContext,
    ) -> Result<(), ModuleError> {
        for module in &self.modules {
            module.register(bus);
            module.on_startup(ctx)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_bus::CommandBus;

    struct StubModule;

    impl Module for StubModule {
        fn manifest(&self) -> ModuleManifest {
            ModuleManifest {
                id: "stub",
                version: "0.1.0",
                capabilities: &[Capability::Sqlite],
            }
        }

        fn register(&self, _bus: &mut CommandBus) {}

        fn on_startup(&self, _ctx: &AppContext) -> Result<(), ModuleError> {
            Ok(())
        }
    }

    #[test]
    fn registry_starts_all_registered_modules() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(StubModule));

        assert_eq!(registry.manifests().len(), 1);
        assert_eq!(registry.manifests()[0].id, "stub");

        let mut bus = CommandBus::new();
        let ctx = AppContext {};
        assert!(registry.start_all(&mut bus, &ctx).is_ok());
    }
}
