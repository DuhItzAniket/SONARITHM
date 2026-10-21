// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The profile commands of the app core (ADR-0020; B03-T08).

use std::path::{Path, PathBuf};

use sonarithm::Mark;
use sonarithm_audaxis::announce::{Part, Settings as ComposerSettings};

use super::App;
use crate::codes;
use crate::ipc::{IpcError, ProfileInfo, Reply};
use crate::profile::{
    self, EffectiveSetting, Layer, Layers, MarkPreference, PRESET_PREFIX, PROJECT_ID, PartName,
    Profile,
};

/// The project profile's member in a `.sonarithm` bundle (RFC-0002 §2, ADR-0020 §2).
pub(super) const PROJECT_MEMBER: &str = "profiles/project-profile.json";
/// User profiles live here, under the app-config folder.
const PROFILES_DIR: &str = "profiles";

fn unknown(id: &str) -> IpcError {
    IpcError::new(
        &codes::UNKNOWN_PROFILE,
        format!("No profile has the ID {id}, or it cannot be changed."),
    )
}

fn user_id_ok(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id != PROJECT_ID
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A file the user chose to import: a regular `.json` file (ADR-0017 §7).
fn import_file(path: &str) -> Result<PathBuf, IpcError> {
    let path = Path::new(path);
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let refuse = |why: &str| IpcError::new(&codes::FILE_REFUSED, format!("`{name}` {why}"));
    if !name.to_ascii_lowercase().ends_with(".json") {
        return Err(refuse("is not a profile file (.json)."));
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| refuse("cannot be found."))?;
    if !canonical.is_file() {
        return Err(refuse("is not a file."));
    }
    Ok(canonical)
}

/// A file to export to: an absolute `.json` path in an existing folder.
fn export_file(path: &str) -> Result<PathBuf, IpcError> {
    let path = Path::new(path);
    let name = path.file_name().filter(|_| path.is_absolute());
    let folder = path.parent().and_then(|f| f.canonicalize().ok());
    let refuse = || {
        IpcError::new(
            &codes::EXPORT_PATH,
            "Export a profile to a .json file in an existing folder.",
        )
    };
    let (Some(name), Some(folder)) = (name, folder) else {
        return Err(refuse());
    };
    if !name
        .to_string_lossy()
        .to_ascii_lowercase()
        .ends_with(".json")
    {
        return Err(refuse());
    }
    let target = folder.join(name);
    if target.is_dir() {
        return Err(refuse());
    }
    Ok(target)
}

fn part(name: PartName) -> Part {
    match name {
        PartName::Series => Part::Series,
        PartName::X => Part::X,
        PartName::Value => Part::Value,
        PartName::Position => Part::Position,
        PartName::Level => Part::Level,
        PartName::Tags => Part::Tags,
        PartName::Provenance => Part::Provenance,
        PartName::Hint => Part::Hint,
    }
}

impl App {
    fn profiles_dir(&self) -> PathBuf {
        self.config_dir.join(PROFILES_DIR)
    }

    /// The user's profiles that read cleanly, by ID.
    fn user_profiles(&self) -> Vec<(String, Profile)> {
        let Ok(entries) = std::fs::read_dir(self.profiles_dir()) else {
            return Vec::new();
        };
        let mut found: Vec<(String, Profile)> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let id = name.strip_suffix(".json")?.to_owned();
                if !user_id_ok(&id) {
                    return None;
                }
                let bytes = std::fs::read(entry.path()).ok()?;
                let (profile, _) = profile::parse(&bytes).ok()?;
                Some((id, profile))
            })
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));
        found
    }

    /// The profile with `id`: a preset, a user profile, or the project's.
    fn find_profile(&self, id: &str) -> Option<Profile> {
        if id == PROJECT_ID {
            return self.project_profile();
        }
        if id.starts_with(PRESET_PREFIX) {
            return profile::presets()
                .into_iter()
                .find(|(preset, _)| preset == id)
                .map(|(_, p)| p);
        }
        if !user_id_ok(id) {
            return None;
        }
        let bytes = std::fs::read(self.profiles_dir().join(format!("{id}.json"))).ok()?;
        profile::parse(&bytes).ok().map(|(p, _)| p)
    }

    /// The active profile, if one is set and readable.
    pub(super) fn active_profile(&self) -> Option<Profile> {
        self.settings
            .active_profile
            .as_deref()
            .and_then(|id| self.find_profile(id))
    }

    /// The project's profile, if the project carries a valid one.
    pub(super) fn project_profile(&self) -> Option<Profile> {
        let bytes = self.project.reserved.get(PROJECT_MEMBER)?;
        profile::parse(bytes).ok().map(|(p, _)| p)
    }

    /// The effective interaction preferences (12 §3).
    pub(super) fn interaction(&self) -> profile::Interaction {
        let active = self.active_profile();
        let project = self.project_profile();
        profile::effective(&Layers {
            settings: &self.settings,
            profile: active.as_ref(),
            project: project.as_ref(),
            session: &self.session_prefs,
        })
    }

    /// The composer settings of the effective verbosity and part order (A11Y-12).
    pub(super) fn composer_settings(&self) -> ComposerSettings {
        let interaction = self.interaction();
        let mut settings = ComposerSettings::default();
        settings.verbosity = match interaction.verbosity {
            crate::settings::Verbosity::Verbose => sonarithm_audaxis::announce::Verbosity::Verbose,
            _ => sonarithm_audaxis::announce::Verbosity::Terse,
        };
        if let Some(order) = interaction.part_order {
            settings.terse = order.terse.into_iter().map(part).collect();
            settings.verbose = order.verbose.into_iter().map(part).collect();
        }
        settings
    }

    /// The spec to compile: `text` with the profile and project layers filled in.
    pub(super) fn effective_spec(
        &self,
        text: &str,
    ) -> Result<(String, Vec<EffectiveSetting>), IpcError> {
        let active = self.active_profile();
        let project = self.project_profile();
        let layers = profile::sound(active.as_ref(), project.as_ref());
        let data = self
            .datasets
            .values()
            .map(|open| (open.summary.id.clone(), open.bundle.clone()))
            .collect();
        profile::effective_spec(text, &layers, &data)
    }

    /// The marks the profile layers prefer, if any.
    fn preferred_marks(&self) -> Option<MarkPreference> {
        let active = self.active_profile();
        let project = self.project_profile();
        profile::sound(active.as_ref(), project.as_ref())
            .into_iter()
            .filter_map(|(_, sound)| sound.marks)
            .next_back()
    }

    pub(super) fn profile_list(&self, warnings: &[IpcError]) -> Reply {
        let active = self.settings.active_profile.as_deref();
        let mut profiles: Vec<ProfileInfo> = profile::presets()
            .into_iter()
            .chain(self.user_profiles())
            .map(|(id, p)| ProfileInfo {
                active: Some(id.as_str()) == active,
                preset: id.starts_with(PRESET_PREFIX),
                id,
                name: p.name,
            })
            .collect();
        if let Some(project) = self.project_profile() {
            profiles.push(ProfileInfo {
                id: PROJECT_ID.to_owned(),
                name: project.name,
                preset: false,
                active: false,
            });
        }
        Reply::Profiles {
            profiles,
            warnings: warnings.iter().map(super::projects::warning_info).collect(),
        }
    }

    pub(super) fn profile_get(&self, id: &str) -> Result<Reply, IpcError> {
        let profile = self.find_profile(id).ok_or_else(|| unknown(id))?;
        Ok(Reply::Profile {
            id: id.to_owned(),
            profile,
        })
    }

    pub(super) fn profile_activate(&mut self, id: Option<String>) -> Result<Reply, IpcError> {
        if let Some(id) = &id
            && (id == PROJECT_ID || self.find_profile(id).is_none())
        {
            return Err(unknown(id));
        }
        let mut settings = self.settings.clone();
        settings.active_profile = id;
        self.store.save(&settings)?;
        self.settings = settings;
        self.transport.configure(self.output_config())?;
        Ok(self.profile_list(&[]))
    }

    /// A new user ID for `name`: its slug, made unique.
    fn new_id(&self, name: &str) -> String {
        let mut slug: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        slug = slug
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        slug.truncate(48);
        if slug.is_empty() || slug == PROJECT_ID {
            "profile".clone_into(&mut slug);
        }
        let taken: Vec<String> = self.user_profiles().into_iter().map(|(id, _)| id).collect();
        let mut id = slug.clone();
        let mut n = 2;
        while taken.contains(&id) {
            id = format!("{slug}-{n}");
            n += 1;
        }
        id
    }

    /// Writes a user profile, crash-safely.
    fn write_user(&self, id: &str, profile: &Profile) -> Result<(), IpcError> {
        let bytes = profile::write(profile)?;
        let dir = self.profiles_dir();
        std::fs::create_dir_all(&dir).map_err(|err| {
            IpcError::new(
                &codes::SETTINGS_WRITE,
                format!("The profiles folder could not be created: {}.", err.kind()),
            )
        })?;
        crate::settings::write_atomically(&dir.join(format!("{id}.json")), &bytes, None).map_err(
            |detail| {
                IpcError::new(
                    &codes::SETTINGS_WRITE,
                    format!("The profile could not be saved: {detail}."),
                )
            },
        )
    }

    pub(super) fn profile_save(
        &mut self,
        id: Option<&str>,
        profile: &Profile,
    ) -> Result<Reply, IpcError> {
        profile::validate(profile)?;
        match id {
            Some(PROJECT_ID) => {
                let bytes = profile::write(profile)?;
                self.project
                    .reserved
                    .insert(PROJECT_MEMBER.to_owned(), bytes);
                self.changed();
            }
            Some(id) if id.starts_with(PRESET_PREFIX) || !user_id_ok(id) => {
                return Err(unknown(id));
            }
            Some(id) => self.write_user(id, profile)?,
            None => {
                let id = self.new_id(&profile.name);
                self.write_user(&id, profile)?;
            }
        }
        self.transport.configure(self.output_config())?;
        Ok(self.profile_list(&[]))
    }

    pub(super) fn profile_import(&mut self, path: &str) -> Result<Reply, IpcError> {
        let file = import_file(path)?;
        let size = std::fs::metadata(&file).map_or(0, |m| m.len());
        if size > profile::MAX_BYTES as u64 {
            return Err(IpcError::new(
                &codes::PROFILE_INVALID,
                "The profile is not valid: it is larger than 1 MiB.",
            ));
        }
        let bytes = std::fs::read(&file).map_err(|err| {
            IpcError::new(
                &codes::FILE_REFUSED,
                format!("The profile could not be read: {}.", err.kind()),
            )
        })?;
        let (imported, warnings) = profile::parse(&bytes)?;
        let id = self.new_id(&imported.name);
        self.write_user(&id, &imported)?;
        Ok(self.profile_list(&warnings))
    }

    pub(super) fn profile_export(&self, id: &str, path: &str) -> Result<Reply, IpcError> {
        let profile = self.find_profile(id).ok_or_else(|| unknown(id))?;
        let target = export_file(path)?;
        let bytes = profile::write(&profile)?;
        crate::settings::write_atomically(&target, &bytes, None).map_err(|detail| {
            IpcError::new(
                &codes::EXPORT_FAILED,
                format!("The profile could not be exported: {detail}."),
            )
        })?;
        Ok(self.profile_list(&[]))
    }

    /// The one-key flip (SDR-15): toggles `field` in the active user profile (a copy of a
    /// preset, or a new profile, when needed), saves it, and recompiles.
    pub(super) fn profile_flip_polarity(&mut self, field: &str) -> Result<Reply, IpcError> {
        if self.spec.is_none() {
            return Err(IpcError::new(
                &codes::NO_SPEC,
                "There is no spec whose polarity to flip.",
            ));
        }
        let active = self.settings.active_profile.clone();
        let (id, mut profile) = match active {
            Some(id) if user_id_ok(&id) => {
                let profile = self.find_profile(&id).ok_or_else(|| unknown(&id))?;
                (id, profile)
            }
            Some(id) => {
                let mut copy = self.find_profile(&id).ok_or_else(|| unknown(&id))?;
                copy.name = format!("{} (mine)", copy.name);
                (self.new_id(&copy.name), copy)
            }
            None => {
                let profile = Profile::new("My profile");
                (self.new_id(&profile.name), profile)
            }
        };
        let polarity = profile.sound.polarity.get_or_insert_with(Default::default);
        if !polarity.flipped_fields.remove(field) {
            polarity.flipped_fields.insert(field.to_owned());
        }
        self.write_user(&id, &profile)?;
        if self.settings.active_profile.as_deref() != Some(id.as_str()) {
            let mut settings = self.settings.clone();
            settings.active_profile = Some(id);
            self.store.save(&settings)?;
            self.settings = settings;
        }
        self.compile()
    }

    pub(super) fn settings_effective(&self) -> Result<Reply, IpcError> {
        let mut settings = self.interaction().view;
        if let Some(text) = self.spec.as_deref() {
            let (_, sound) = self.effective_spec(text)?;
            settings.extend(sound);
        }
        settings.retain(|s| s.layer != Layer::Default || !s.value.is_null());
        Ok(Reply::Effective { settings })
    }

    /// A quick spec over an open dataset with the preferred marks (SDR-38), set as an
    /// undoable edit.
    pub(super) fn spec_quick(
        &mut self,
        dataset: &str,
        x: Option<String>,
        y: String,
    ) -> Result<Reply, IpcError> {
        let open = self
            .datasets
            .values()
            .find(|open| open.summary.id == dataset)
            .ok_or_else(|| super::unknown_dataset(dataset))?;
        let mut quick = self.snr.quick().data(open.bundle.clone()).y(y);
        if let Some(x) = x {
            quick = quick.x(x);
        }
        match self.preferred_marks() {
            Some(MarkPreference::Discrete) => quick = quick.mark(Mark::Tone),
            Some(MarkPreference::Continuous) => quick = quick.mark(Mark::Glide),
            None => {}
        }
        let spec = quick.spec().map_err(|err| super::first_error(&err))?;
        let mut value: serde_json::Value = serde_json::from_str(spec.text()).map_err(|err| {
            IpcError::new(
                &codes::BAD_REQUEST,
                format!("The quick spec is not JSON: {err}."),
            )
        })?;
        value["data"]["source"] = serde_json::json!({ "kind": "dataset", "id": dataset });
        let text = serde_json::to_string_pretty(&value).unwrap_or_default();
        let reply = self.spec_set(text)?;
        self.changed();
        Ok(reply)
    }
}
