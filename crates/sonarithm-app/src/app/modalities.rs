// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The explorer's modality and verbosity toggles (B03-T09; keyboard-and-navigation §5.5;
//! A11Y-10, A11Y-12).
//!
//! A toggle flips the *effective* value, the one every output acts on, by writing the
//! explicit-UI-action layer (12 §3, ADR-0020 §3) on top of the profile. That layer is saved
//! in the settings, so the choice survives a restart and no profile overrides it later;
//! `choices_reset` returns to the profile's values. The outputs follow at once: speech off
//! logs announcements without speaking them, sonification off silences the master (cues
//! too), braille off empties the braille line, visual off hides the highlight (the UI reads
//! the effective view), and the verbosity is the composer's.
//!
//! Each change is announced with `E-MODE`, before speech goes off and after it comes back,
//! so it is always heard. Speech, sonification and braille are never all off at once: a
//! blind listener must not silence the app by accident (`SNR-APP-094`; INFER, OD-30).

use sonarithm_audaxis::announce::Announcement;
use sonarithm_audaxis::earcons::EarconId;
use sonarithm_i18n::Args;

use super::App;
use crate::codes;
use crate::ipc::{IpcError, Reply, ToggleTarget};
use crate::settings::{UiChoices, Verbosity};

impl App {
    /// `modality_toggle`: flips one effective modality, or the verbosity.
    ///
    /// # Errors
    ///
    /// `SNR-APP-094` when the change would leave no non-visual output, and the settings
    /// store's errors when the choice cannot be saved.
    pub(super) fn modality_toggle(&mut self, target: ToggleTarget) -> Result<Reply, IpcError> {
        let current = self.interaction();
        let [sonification, speech, braille, visual] = current.modalities;
        let mut choices = self.settings.choices;
        let (key, on) = match target {
            ToggleTarget::Speech => {
                choices.speech = Some(!speech);
                ("speech", !speech)
            }
            ToggleTarget::Sonification => {
                choices.sonification = Some(!sonification);
                ("sonification", !sonification)
            }
            ToggleTarget::Braille => {
                choices.braille = Some(!braille);
                ("braille", !braille)
            }
            ToggleTarget::Visual => {
                choices.visual = Some(!visual);
                ("visual", !visual)
            }
            ToggleTarget::Verbosity => {
                let next = if current.verbosity == Verbosity::Verbose {
                    Verbosity::Terse
                } else {
                    Verbosity::Verbose
                };
                choices.verbosity = Some(next);
                ("verbosity", next == Verbosity::Verbose)
            }
            _ => return Err(IpcError::new(&codes::BAD_REQUEST, "Unknown toggle.")),
        };
        // No lockout: at least one of speech, sonification and braille stays on.
        let after = |chosen: Option<bool>, now: bool| chosen.unwrap_or(now);
        let audible = after(choices.speech, speech)
            || after(choices.sonification, sonification)
            || after(choices.braille, braille);
        if !audible {
            // Spoken as an error, so it is heard whatever is muted.
            if let Some(text) = self.toggle_text("announce-modality-lockout", &Args::new()) {
                self.transport.announce(Announcement::error(text.clone(), text));
            }
            return Err(IpcError::new(
                &codes::MODALITY_LOCKOUT,
                "Speech, sonification and braille cannot all be off. Nothing was changed.",
            ));
        }
        let said = if target == ToggleTarget::Verbosity {
            self.toggle_text(
                "announce-verbosity-changed",
                &Args::new().text("verbosity", if on { "verbose" } else { "terse" }),
            )
        } else {
            self.toggle_text(
                "announce-modality-changed",
                &Args::new()
                    .text("modality", key)
                    .text("on", if on { "yes" } else { "no" }),
            )
        };
        // Speech going off: say so while it is still on. Everything else: apply, then say.
        let speech_off = target == ToggleTarget::Speech && !on;
        if speech_off {
            self.announce_mode(said.as_deref());
        }
        self.apply_choices(choices)?;
        if !speech_off {
            self.announce_mode(said.as_deref());
        }
        self.settings_effective()
    }

    /// `choices_reset`: clears the toggles' choices, back to the profile's values.
    ///
    /// # Errors
    ///
    /// The settings store's errors when the settings cannot be saved.
    pub(super) fn choices_reset(&mut self) -> Result<Reply, IpcError> {
        self.apply_choices(UiChoices::default())?;
        let said = self.toggle_text("announce-choices-reset", &Args::new());
        self.announce_mode(said.as_deref());
        self.settings_effective()
    }

    /// Saves `choices`, makes them the explicit-UI-action layer and reconfigures the
    /// transport's outputs.
    fn apply_choices(&mut self, choices: UiChoices) -> Result<(), IpcError> {
        let mut settings = self.settings.clone();
        settings.choices = choices;
        self.store.save(&settings)?;
        self.settings = settings;
        self.session_prefs = crate::profile::choices_layer(&self.settings.choices);
        self.transport.configure(self.output_config())?;
        Ok(())
    }

    fn announce_mode(&self, text: Option<&str>) {
        if let Some(text) = text {
            let mut announcement = Announcement::status(text, text);
            announcement.earcon = Some(EarconId::Mode);
            self.transport.announce(announcement);
        }
    }

    fn toggle_text(&self, id: &str, args: &Args) -> Option<String> {
        self.localizer.as_ref()?.format(id, args).ok()
    }
}
