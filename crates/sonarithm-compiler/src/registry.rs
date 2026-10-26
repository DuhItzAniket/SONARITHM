// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Builds the engine registry and reports conflicts as `SNR-COMP` diagnostics (05 §3).

use sonarithm_core::diagnostic::Diagnostic;
use sonarithm_core::engine::{Engine, Registry, RegistryError};

use crate::codes;

/// Registers `engines` in order. Order matters only for the default encoder of a mark
/// that no engine claims explicitly: the first engine to register it wins.
///
/// # Errors
///
/// The first registration conflict, as an `SNR-COMP-011` … `015` diagnostic.
pub fn build_registry<D>(engines: &[&dyn Engine<D>]) -> Result<Registry<D>, Box<Diagnostic>> {
    let mut registry = Registry::new();
    for engine in engines {
        registry
            .add_engine(*engine)
            .map_err(|err| Box::new(registry_diagnostic(&err)))?;
    }
    Ok(registry)
}

/// The diagnostic for a registration conflict.
#[must_use]
pub fn registry_diagnostic(error: &RegistryError) -> Diagnostic {
    let code = match error {
        RegistryError::DuplicateEngine { .. } => &codes::DUPLICATE_ENGINE,
        RegistryError::DuplicateSource { .. } => &codes::DUPLICATE_SOURCE,
        RegistryError::DuplicateAnalyzer { .. } => &codes::DUPLICATE_ANALYZER,
        RegistryError::DuplicateShaper { .. } => &codes::DUPLICATE_SHAPER,
        RegistryError::DuplicateTemplate { .. } => &codes::DUPLICATE_TEMPLATE,
        _ => &codes::CONFLICTING_ENCODER,
    };
    Diagnostic::new(code, format!("{error}."))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sonarithm_core::engine::{
        CompileContext, CompileError, EngineManifest, Registrar, Source, SourceKind, SpecFragment,
        TemplateExpansion, TemplateProvider,
    };
    use sonarithm_core::ids::EngineId;
    use sonarithm_core::score::EngineVersion;

    use super::*;

    struct NoSource;

    impl Source<()> for NoSource {
        fn ingest(&self, _: &SpecFragment, _: &mut CompileContext) -> Result<(), CompileError> {
            Ok(())
        }
    }

    struct FunctionSource(&'static str);

    impl Engine<()> for FunctionSource {
        fn manifest(&self) -> EngineManifest {
            EngineManifest::new(
                EngineId::from_static(self.0),
                self.0,
                EngineVersion("0.0.0".to_owned()),
            )
        }

        fn register(&self, registrar: &mut Registrar<'_, ()>) -> Result<(), RegistryError> {
            registrar.source(SourceKind::Function, Arc::new(NoSource))
        }
    }

    /// A03-T04 acceptance: a duplicate registration fails with an `SNR-COMP` code.
    #[test]
    fn duplicate_registration_fails_with_an_snr_comp_code() {
        let curvasona = FunctionSource("curvasona");
        let duplicate = build_registry(&[&curvasona, &curvasona]).err().unwrap();
        assert_eq!(duplicate.code.as_str(), "SNR-COMP-011");

        let plotsona = FunctionSource("plotsona");
        let conflict = build_registry(&[&curvasona, &plotsona]).err().unwrap();
        assert_eq!(conflict.code.as_str(), "SNR-COMP-012");
        assert_eq!(
            conflict.message,
            "engines `curvasona` and `plotsona` both register the `function` source."
        );
        assert!(build_registry(&[&curvasona]).is_ok());
    }

    struct NoTemplate;

    impl TemplateProvider<()> for NoTemplate {
        fn expand(
            &self,
            _: &SpecFragment,
            (): &(),
            _: &mut CompileContext,
        ) -> Result<TemplateExpansion, CompileError> {
            Err(CompileError::Cancelled)
        }
    }

    struct Templates(&'static str);

    impl Engine<()> for Templates {
        fn manifest(&self) -> EngineManifest {
            EngineManifest::new(
                EngineId::from_static(self.0),
                self.0,
                EngineVersion("0.0.0".to_owned()),
            )
        }

        fn register(&self, registrar: &mut Registrar<'_, ()>) -> Result<(), RegistryError> {
            registrar.template("chart-line", Arc::new(NoTemplate))
        }
    }

    /// RFC-0004 §2: a template ID belongs to one engine (`SNR-COMP-016`), and the registry
    /// lists the IDs with their engines.
    #[test]
    fn a_template_id_registered_twice_is_snr_comp_016() {
        let first = Templates("numisona");
        let second = Templates("plotsona");
        let conflict = build_registry(&[&first, &second]).err().unwrap();
        assert_eq!(conflict.code.as_str(), "SNR-COMP-016");
        let registry = build_registry(&[&first]).unwrap();
        let listed: Vec<(&str, &str)> = registry
            .templates()
            .map(|(id, engine)| (id, engine.as_str()))
            .collect();
        assert_eq!(listed, [("chart-line", "numisona")]);
        assert!(registry.template("chart-line").is_some());
        assert!(registry.template("chart-pie").is_none());
    }
}
