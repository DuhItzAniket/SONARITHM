// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The command line (11 §2), declared with clap.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Turn data into structured, navigable sound.
///
/// Exit codes: 0 success, 1 errors, 2 usage, 3 warnings denied. Every command accepts
/// --json for machine-readable output and --plain for linear text without colour or box
/// drawing (screen readers).
#[derive(Debug, Parser)]
#[command(
    name = "sonarithm",
    bin_name = "sonarithm",
    version,
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    /// Print machine-readable JSON on standard output, with stable field names.
    #[arg(long, global = true)]
    pub(crate) json: bool,
    /// Print linear text: no colour, no box drawing, progress as text lines.
    #[arg(long, global = true)]
    pub(crate) plain: bool,
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Render a spec offline to WAV or FLAC, with its render manifest and cue sidecar.
    Render(Render),
    /// Check a spec: syntax, shape, meaning, and the defaults it will get.
    Validate(Checked),
    /// Compile a spec; print the Score summary, a description and the diagnostics.
    Inspect(Inspect),
    /// Compile a spec and report its warnings with rule IDs (a stub until the perceptual
    /// lint arrives in C05).
    Lint(Checked),
    /// Sonify one data file with the default design: x along time, y as pitch.
    Quick(Quick),
    /// Print a JSON Schema.
    Schema(Schema),
    /// Work on spec files.
    #[command(subcommand)]
    Spec(SpecCommand),
}

/// A spec to check, with the warning policy.
#[derive(Debug, Args)]
pub(crate) struct Checked {
    /// The spec file, or `-` for standard input.
    pub(crate) spec: PathBuf,
    /// Exit with code 3 when there are warnings.
    #[arg(long)]
    pub(crate) deny_warnings: bool,
    /// Also list the applied defaults and automatic choices (Info diagnostics).
    #[arg(long, short)]
    pub(crate) verbose: bool,
}

/// Output settings shared by `render` and `quick -o`.
#[derive(Debug, Args)]
pub(crate) struct Output {
    /// The audio format; by default from the output file's extension (.wav or .flac).
    #[arg(long, value_enum)]
    pub(crate) format: Option<FormatArg>,
    /// The sample rate in hertz; by default the spec's (48000).
    #[arg(long, value_name = "HZ")]
    pub(crate) sample_rate: Option<u32>,
    /// Keep the rendered level instead of normalizing to the spec's loudness target.
    #[arg(long)]
    pub(crate) preserve_level: bool,
    /// Do not write the cue sidecar (.cues.vtt).
    #[arg(long)]
    pub(crate) no_cues: bool,
}

#[derive(Debug, Args)]
pub(crate) struct Render {
    /// The spec file, or `-` for standard input.
    pub(crate) spec: PathBuf,
    /// The audio file to write; the manifest and cues go next to it.
    #[arg(short, long, value_name = "FILE")]
    pub(crate) output: PathBuf,
    /// Read the data from this file instead of the spec's source.
    #[arg(long, value_name = "FILE")]
    pub(crate) data: Option<PathBuf>,
    /// The seed of every random stream, instead of the spec's.
    #[arg(long)]
    pub(crate) seed: Option<u64>,
    #[command(flatten)]
    pub(crate) audio: Output,
    /// Exit with code 3, writing nothing, when the compile has warnings.
    #[arg(long)]
    pub(crate) deny_warnings: bool,
    /// Also list the applied defaults and automatic choices.
    #[arg(long, short)]
    pub(crate) verbose: bool,
}

/// Audio formats.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum FormatArg {
    /// RIFF WAVE, 24-bit PCM.
    Wav,
    /// FLAC, 24-bit.
    Flac,
}

#[derive(Debug, Args)]
pub(crate) struct Inspect {
    #[command(flatten)]
    pub(crate) checked: Checked,
    /// Also print the L1/L2 text description.
    #[arg(long)]
    pub(crate) describe: bool,
    /// Also print every Score event, as an outline.
    #[arg(long)]
    pub(crate) tree: bool,
}

#[derive(Debug, Args)]
pub(crate) struct Quick {
    /// The data file: CSV, TSV, JSON, NDJSON, Parquet, Arrow IPC, XLSX or ODS.
    pub(crate) data: PathBuf,
    /// The field along time; by default the field with the x role, or the first number.
    #[arg(long)]
    pub(crate) x: Option<String>,
    /// The field mapped to pitch; by default the field with the y role, or the next number.
    #[arg(long)]
    pub(crate) y: Option<String>,
    /// The mark.
    #[arg(long, value_enum)]
    pub(crate) mark: Option<MarkArg>,
    /// The pitch range in hertz, as LOW,HIGH (default 400,1400: SDR-08).
    #[arg(long, value_name = "LOW,HIGH", value_parser = parse_range)]
    pub(crate) range: Option<(f64, f64)>,
    /// The y field's concept, such as `temperature`, which sets its polarity (SDR-12..14).
    #[arg(long)]
    pub(crate) concept: Option<String>,
    /// The spec's name; by default the data file's name without its extension.
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Write the generated spec to this file, to edit and reuse it.
    #[arg(long, value_name = "FILE")]
    pub(crate) spec_out: Option<PathBuf>,
    /// Render to this audio file (.wav or .flac), with its manifest and cues.
    #[arg(short, long, value_name = "FILE")]
    pub(crate) output: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) audio: Output,
    /// Exit with code 3 when there are warnings.
    #[arg(long)]
    pub(crate) deny_warnings: bool,
    /// Also list the applied defaults and automatic choices.
    #[arg(long, short)]
    pub(crate) verbose: bool,
}

/// Marks `quick` offers.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum MarkArg {
    /// One tone per data point.
    Tone,
    /// One continuous contour.
    Glide,
    /// Tones or a glide by density (experimental, SDR-20).
    Auto,
}

#[derive(Debug, Args)]
pub(crate) struct Schema {
    /// Which schema.
    #[arg(long, value_enum, default_value_t = SchemaArg::Spec)]
    pub(crate) kind: SchemaArg,
}

/// Schemas `schema` prints.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum SchemaArg {
    /// The spec v0.1 schema.
    Spec,
}

#[derive(Debug, Subcommand)]
pub(crate) enum SpecCommand {
    /// Migrate a spec to the current spec version.
    Migrate(Migrate),
}

#[derive(Debug, Args)]
pub(crate) struct Migrate {
    /// The spec file, or `-` for standard input.
    pub(crate) spec: PathBuf,
    /// The target spec version, such as `0.1`; by default the current one.
    #[arg(long, value_name = "VERSION")]
    pub(crate) to: Option<String>,
    /// Rewrite the file instead of printing the result.
    #[arg(long)]
    pub(crate) in_place: bool,
}

/// Parses `LOW,HIGH`.
fn parse_range(text: &str) -> Result<(f64, f64), String> {
    let (low, high) = text
        .split_once(',')
        .ok_or_else(|| "write the range as LOW,HIGH, for example 300,900".to_owned())?;
    let number = |part: &str| {
        part.trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| format!("`{}` is not a positive number of hertz", part.trim()))
    };
    let (low, high) = (number(low)?, number(high)?);
    if low >= high {
        return Err(format!(
            "the low end ({low}) must be below the high end ({high})"
        ));
    }
    Ok((low, high))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_parse_and_are_checked() {
        assert_eq!(parse_range("300,900"), Ok((300.0, 900.0)));
        assert_eq!(parse_range(" 300 , 900.5 "), Ok((300.0, 900.5)));
        assert!(parse_range("900,300").is_err());
        assert!(parse_range("300").is_err());
        assert!(parse_range("-1,5").is_err());
        assert!(parse_range("a,5").is_err());
    }

    #[test]
    fn the_command_line_is_consistent() {
        use clap::CommandFactory as _;
        Cli::command().debug_assert();
    }
}
