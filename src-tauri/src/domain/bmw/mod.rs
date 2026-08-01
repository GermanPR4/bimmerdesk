//! Concrete BMW implementation of the domain abstractions. Registered like
//! any other `Manufacturer` — the core never special-cases "BMW" by name
//! outside this module (PROJECT_PRINCIPLES.md, prueba del "dentro de cinco
//! años": añadir Audi/Mercedes debe significar "escribir otro módulo así",
//! nunca "tocar un `if` en el núcleo").

use super::Manufacturer;

pub struct Bmw;

impl Manufacturer for Bmw {
    fn name(&self) -> &'static str {
        "BMW"
    }

    fn platforms(&self) -> &'static [&'static str] {
        // E-Series listed for domain completeness (KWP2000, post-Fase-0);
        // V1 only implements the F-Series path end to end.
        &["F-Series", "E-Series"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bmw_reports_expected_platforms() {
        let bmw = Bmw;
        assert_eq!(bmw.name(), "BMW");
        assert!(bmw.platforms().contains(&"F-Series"));
    }
}
