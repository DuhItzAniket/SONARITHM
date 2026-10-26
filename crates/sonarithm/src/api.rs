// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! [`Sonarithm`], its [`Builder`] and the host [`Datasets`].

use std::collections::BTreeMap;

use sonarithm_audaxis::Audaxis;
use sonarithm_compiler::{Compiler, Failure, Platform};
use sonarithm_core::engine::{CancelToken, Engine, EngineManifest};
use sonarithm_data::{DataBundle, Snapshot};
use sonarithm_numisona::Numisona;
use sonarithm_numisona::polarity::PolarityProfile;
use sonarithm_plotsona::Plotsona;
use sonarithm_sonotrace::Sonotrace;

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
    audaxis: Option<Audaxis>,
}

impl Default for Sonarithm {
    /// [`Sonarithm::new`].
    fn default() -> Self {
        Self::new()
    }
}

impl Sonarithm {
    /// SONARITHM with the default engines (the data platform, NUMISONA and PLOTSONA).
    #[must_use]
    pub fn new() -> Self {
        Self {
            numisona: Some(Numisona::new()),
            audaxis: None,
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
            engines.push(Plotsona::manifest());
        }
        if let Some(audaxis) = &self.audaxis {
            engines.push(Engine::<DataBundle>::manifest(audaxis));
        }
        engines
    }

    /// Compiles a spec whose data comes from its own source (`inline` records, or `file`
    /// and `chart` sources relative to [`Spec::base_dir`]).
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
        self.compile_engines(spec, datasets, cancel, None)
    }

    /// [`Sonarithm::compile_with`] for a spec whose source is a live stream
    /// (`{"kind": "stream", …}`): SONOTRACE reads `window`, a snapshot of the stream's rows
    /// ([`StreamTable::snapshot`](sonarithm_data::StreamTable::snapshot)), so the stream keeps receiving while this compiles
    /// (B06-T07). Each call compiles one window; live monitoring calls it as rows arrive.
    ///
    /// # Errors
    ///
    /// As [`Sonarithm::compile_with`]; `SNR-ENG-200` never, since the window is given.
    pub fn compile_stream(
        &self,
        spec: &Spec,
        datasets: &Datasets,
        window: &Snapshot,
    ) -> Result<Compiled, Error> {
        let sonotrace = Sonotrace::with_snapshot(window).map_err(|err| {
            let mut one = sonarithm_core::Diagnostics::new();
            one.push(err.to_diagnostic());
            Error::Spec(one)
        })?;
        self.compile_engines(spec, datasets, &CancelToken::new(), Some(&sonotrace))
    }

    fn compile_engines(
        &self,
        spec: &Spec,
        datasets: &Datasets,
        cancel: &CancelToken,
        stream: Option<&Sonotrace>,
    ) -> Result<Compiled, Error> {
        let mut platform = spec
            .base_dir()
            .map_or_else(Platform::new, Platform::with_base_dir);
        for (id, bundle) in &datasets.0 {
            platform = platform.with_dataset(id.clone(), bundle.clone());
        }
        let plotsona = spec
            .base_dir()
            .map_or_else(Plotsona::new, Plotsona::with_base_dir);
        let mut engines: Vec<&dyn Engine<DataBundle>> = vec![&platform];
        if let Some(numisona) = &self.numisona {
            engines.push(numisona);
            engines.push(&plotsona);
        }
        if let Some(audaxis) = &self.audaxis {
            engines.push(audaxis);
        }
        if let Some(sonotrace) = stream {
            engines.push(sonotrace);
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
    navigation: bool,
    profile: Option<PolarityProfile>,
}

impl Builder {
    /// Registers the default engines: NUMISONA (parameter mapping and its templates) and
    /// PLOTSONA (`chart` sources and chart-type templates, C01). The data platform
    /// (`inline`, `file` and `dataset` sources) is always registered.
    pub fn with_default_engines(mut self) -> Self {
        self.default_engines = true;
        self
    }

    /// Registers AUDAXIS (B03): compiles also build the navigation tree beside the Score
    /// ([`Compiled::nav`], 05 §8), AUDAXIS shapes the Score (stereo placement, A11Y-14) and
    /// records itself in the header, and the spec's `context` cues come from it. The app
    /// core uses this; the CLI and Python bindings do not yet, so their Scores differ from
    /// the app's by AUDAXIS's shaping until B03 makes it a default engine.
    pub fn with_navigation(mut self) -> Self {
        self.navigation = true;
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
        let audaxis = self.navigation.then(Audaxis::new);
        let snr = Sonarithm { numisona, audaxis };
        // Registering once here surfaces conflicts at build time, not at the first compile.
        let platform = Platform::new();
        let plotsona = Plotsona::new();
        let mut engines: Vec<&dyn Engine<DataBundle>> = vec![&platform];
        if let Some(numisona) = &snr.numisona {
            engines.push(numisona);
            engines.push(&plotsona);
        }
        if let Some(audaxis) = &snr.audaxis {
            engines.push(audaxis);
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
