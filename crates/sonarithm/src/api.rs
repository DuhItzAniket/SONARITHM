// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! [`Sonarithm`], its [`Builder`] and the host [`Datasets`].

use std::collections::BTreeMap;

use sonarithm_compiler::{Compiler, Failure, Platform};
use sonarithm_core::engine::{CancelToken, Engine, EngineManifest};
use sonarithm_data::DataBundle;
use sonarithm_numisona::Numisona;
use sonarithm_numisona::polarity::PolarityProfile;

use crate::compiled::Compiled;
use crate::error::Error;
use crate::quick::Quick;
use crate::spec::Spec;

/// The SONARITHM compiler with its engines: the entry point of the facade.
///
/// It holds no data and no state between calls, so one instance can compile any number of
/// specs, from any thread.
///
/// ```
/// use sonarithm::{Sonarithm, Spec};
///
/// let snr = Sonarithm::builder().with_default_engines().build()?;
/// let spec = Spec::from_json_str(r#"{ "specVersion": "0.1",
///     "data": { "source": { "kind": "inline", "values": [
///         { "day": 1, "temp": 8.5 }, { "day": 2, "temp": 9.25 }, { "day": 3, "temp": 7.0 } ] } },
///     "substrate": { "time": { "field": "day" } },
///     "tracks": [{ "id": "temps", "mark": "tone", "channels": { "pitch": { "field": "temp" } } }] }"#)?;
/// let compiled = snr.compile(&spec)?;
/// assert_eq!(compiled.score().tracks[0].events.len(), 3);
/// for d in compiled.diagnostics().iter() {
///     println!("{d}"); // every applied default, with its rule ID
/// }
/// # Ok::<(), sonarithm::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Sonarithm {
    numisona: Option<Numisona>,
}

impl Default for Sonarithm {
    /// [`Sonarithm::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Sonarithm {
    /// SONARITHM with the default engines (the data platform and NUMISONA).
    #[must_use]
    pub fn new() -> Self {
        Self {
            numisona: Some(Numisona::new()),
        }
    }

    /// A builder, to choose the engines and the listener's polarity profile.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// The engines a compile uses, in registration order: the data platform first.
    #[must_use]
    pub fn engines(&self) -> Vec<EngineManifest> {
        let mut engines = vec![Platform::new().manifest()];
        if let Some(numisona) = &self.numisona {
            engines.push(Engine::<DataBundle>::manifest(numisona));
        }
        engines
    }

    /// Compiles a spec whose data comes from its own source (`inline` records, or `file`
    /// sources relative to [`Spec::base_dir`]).
    ///
    /// # Errors
    ///
    /// [`Error::Spec`] with every diagnostic when the spec or an engine reports an error.
    pub fn compile(&self, spec: &Spec) -> Result<Compiled, Error> {
        self.compile_with(spec, &Datasets::new())
    }

    /// Compiles a spec whose `dataset` sources name data the caller already holds, such as
    /// a table built in memory: `{"kind": "dataset", "id": "main"}` reads
    /// `datasets.get("main")`.
    ///
    /// # Errors
    ///
    /// As [`Sonarithm::compile`]; an unknown dataset ID is `SNR-COMP-026`.
    pub fn compile_with(&self, spec: &Spec, datasets: &Datasets) -> Result<Compiled, Error> {
        self.compile_cancellable(spec, datasets, &CancelToken::new())
    }

    /// [`Sonarithm::compile_with`] that stops between stages once `cancel` is cancelled.
    ///
    /// # Errors
    ///
    /// As [`Sonarithm::compile_with`], and [`Error::Cancelled`].
    pub fn compile_cancellable(
        &self,
        spec: &Spec,
        datasets: &Datasets,
        cancel: &CancelToken,
    ) -> Result<Compiled, Error> {
        let mut platform = spec
            .base_dir()
            .map_or_else(Platform::new, Platform::with_base_dir);
        for (id, bundle) in &datasets.0 {
            platform = platform.with_dataset(id.clone(), bundle.clone());
        }
        let mut engines: Vec<&dyn Engine<DataBundle>> = vec![&platform];
        if let Some(numisona) = &self.numisona {
            engines.push(numisona);
        }
        let compiler = Compiler::new(&engines).map_err(|diagnostic| {
            let mut one = sonarithm_core::Diagnostics::new();
            one.push(*diagnostic);
            Error::Spec(one)
        })?;
        match compiler.compile(spec.text(), cancel) {
            Ok(output) => Ok(Compiled::new(output, spec.name())),
            Err(Failure::Invalid(diagnostics)) => Err(Error::Spec(diagnostics)),
            Err(Failure::Cancelled) => Err(Error::Cancelled),
            Err(_) => Err(Error::Spec(sonarithm_core::Diagnostics::new())),
        }
    }

    /// A quick sonification of one dataset with the `line-overview` template: x along
    /// time, y as pitch, every default from the evidence (11 §1).
    pub fn quick(&self) -> Quick {
        Quick::new(self.clone())
    }
}

/// Builds a [`Sonarithm`].
#[derive(Clone, Debug, Default)]
#[must_use]
pub struct Builder {
    default_engines: bool,
    profile: Option<PolarityProfile>,
}

impl Builder {
    /// Registers the default engines: NUMISONA (parameter mapping). The data platform
    /// (`inline`, `file` and `dataset` sources) is always registered.
    pub fn with_default_engines(mut self) -> Self {
        self.default_engines = true;
        self
    }

    /// Uses a listener's polarity profile in NUMISONA (SDR-15).
    pub fn with_polarity_profile(mut self, profile: PolarityProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    /// The configured [`Sonarithm`].
    ///
    /// # Errors
    ///
    /// [`Error::Spec`] with the registration conflict (`SNR-COMP-011` … `015`), which
    /// only a misconfigured build can cause.
    pub fn build(self) -> Result<Sonarithm, Error> {
        let numisona = self.default_engines.then(|| match self.profile {
            Some(profile) => Numisona::with_profile(profile),
            None => Numisona::new(),
        });
        let snr = Sonarithm { numisona };
        // Registering once here surfaces conflicts at build time, not at the first compile.
        let platform = Platform::new();
        let mut engines: Vec<&dyn Engine<DataBundle>> = vec![&platform];
        if let Some(numisona) = &snr.numisona {
            engines.push(numisona);
        }
        Compiler::new(&engines).map_err(|diagnostic| {
            let mut one = sonarithm_core::Diagnostics::new();
            one.push(*diagnostic);
            Error::Spec(one)
        })?;
        Ok(snr)
    }
}

/// Data the caller holds, by ID, for `dataset` sources.
///
/// ```
/// use sonarithm::{Datasets, Format, ReadOptions, Sonarithm, Spec};
///
/// let bundle = sonarithm::read_bytes(b"x,y\n1,2\n2,4\n", Format::Csv, "frame.csv", &ReadOptions::new())?
///     .bundle;
/// let spec = Spec::from_json_str(r#"{ "specVersion": "0.1",
///     "data": { "source": { "kind": "dataset", "id": "main" } },
///     "tracks": [{ "id": "y", "mark": "tone", "channels": { "pitch": { "field": "y" } } }] }"#)?;
/// let compiled = Sonarithm::new().compile_with(&spec, &Datasets::new().with("main", bundle))?;
/// assert_eq!(compiled.score().tracks[0].events.len(), 2);
/// # Ok::<(), sonarithm::Error>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct Datasets(BTreeMap<String, DataBundle>);

impl Datasets {
    /// No datasets.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same datasets with `bundle` under `id`; an existing ID is replaced.
    #[must_use]
    pub fn with(mut self, id: impl Into<String>, bundle: DataBundle) -> Self {
        self.insert(id, bundle);
        self
    }

    /// Adds `bundle` under `id`; an existing ID is replaced.
    pub fn insert(&mut self, id: impl Into<String>, bundle: DataBundle) {
        self.0.insert(id.into(), bundle);
    }

    /// The bundle with `id`.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&DataBundle> {
        self.0.get(id)
    }

    /// The IDs, in order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(String::as_str)
    }
}
