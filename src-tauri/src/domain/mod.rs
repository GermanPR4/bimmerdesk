//! Domain hierarchy: Manufacturer -> VehiclePlatform -> Vehicle -> Protocol.
//! See PROJECT_PLAN.md 3.3. The core only ever depends on these
//! abstractions; `bmw::Bmw` is a concrete implementation registered like any
//! other manufacturer, never special-cased in core logic — see
//! PROJECT_PRINCIPLES.md ("prohibido el condicional por identidad de
//! marca/módulo en lógica de negocio").

pub mod bmw;

/// A vehicle manufacturer supported by the app.
pub trait Manufacturer: Send + Sync {
    fn name(&self) -> &'static str;
    fn platforms(&self) -> &'static [&'static str];
}

/// A platform/generation within a manufacturer (e.g. "F-Series", "E-Series").
/// Determines which `Protocol` implementation applies to a given `Vehicle`.
#[derive(Debug, Clone)]
pub struct VehiclePlatform {
    pub manufacturer: &'static str,
    pub name: &'static str,
    pub protocol: &'static str,
}

/// A concrete vehicle instance. Identification-only in Fase 0 — the full
/// Vehicle Profile (history, maintenance, photos, notes...) is built module
/// by module from Fase 2 onward, owned by their respective modules
/// (see PROJECT_PLAN.md 3.4).
#[derive(Debug, Clone)]
pub struct Vehicle {
    pub vin: String,
    pub platform: VehiclePlatform,
}
