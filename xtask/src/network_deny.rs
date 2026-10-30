// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `cargo xtask network-deny`: runs the app-core test executables under an OS mechanism
//! that records every outbound network connection attempt, and fails if there is any
//! (NFR-04, ADR-0012, AGENTS.md rule 9; B01-T06).
//!
//! | OS | Mechanism | Needs |
//! |---|---|---|
//! | Linux | `strace -f` traces `connect`, `sendto`, `sendmsg` and `sendmmsg`; an `AF_INET` or `AF_INET6` address is an attempt | `strace` |
//! | macOS | `sandbox-exec` with a profile that denies `network-outbound` to any IP address; the kernel's `deny(1) network-outbound` log lines for the process are the attempts (`log show`) | nothing extra |
//! | Windows | Windows Filtering Platform connection auditing (`auditpol`, events 5156 and 5157 in the Security log); an outbound event for the process is an attempt | an elevated shell (CI runners are) |
//!
//! `--self-test` runs a probe that connects on purpose (TCP and UDP to 192.0.2.1, a
//! documentation address that routes nowhere) and fails unless the mechanism sees it, so
//! a mechanism that stopped working cannot pass silently. testing-strategy §10 documents
//! the mechanisms; B01-T09 runs the same checks against the packaged app.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result, bail};
use cargo_metadata::Message;

/// The crate whose test executables run under the harness.
const PACKAGE: &str = "sonarithm-app";
/// The probe test (in `crates/sonarithm-app/tests/network_probe.rs`).
const PROBE: &str = "network_probe_connects";

/// One test executable's result.
struct Run {
    name: String,
    /// The process ID, for the log-based mechanisms.
    pid: Option<u32>,
    attempts: Vec<String>,
    /// What the tests printed, when the mechanism captures it (sandbox-exec).
    output: String,
}

/// The waits, in seconds, before each read of the OS log by the self-test: about a minute
/// in all. The unified log (macOS) and the Security log (Windows) persist entries
/// asynchronously, sometimes tens of seconds late on a busy runner, so the self-test reads
/// until it sees the probe.
const SELF_TEST_WAITS: [u64; 7] = [2, 3, 5, 8, 12, 15, 15];

/// The wait before the one read of the OS log after an ordinary run, which must not miss
/// a late entry.
const SETTLE: u64 = 10;

pub(crate) fn run(root: &Path, rest: &[String]) -> Result<()> {
    let self_test = match rest {
        [] => false,
        [flag] if flag == "--self-test" => true,
        [other, ..] => bail!("unexpected option `{other}`; expected `--self-test` or nothing"),
    };
    let mechanism = Mechanism::detect()?;
    mechanism.prepare(root)?;
    let executables = build(root)?;
    let mut runs = Vec::new();
    let started = Instant::now();
    for exe in &executables {
        let name = exe
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let is_probe = name.starts_with("network_probe");
        if self_test != is_probe {
            continue;
        }
        let args: Vec<&str> = if self_test {
            vec!["--ignored", "--exact", PROBE, "--nocapture"]
        } else {
            vec![]
        };
        println!("network-deny: running {name} under {}", mechanism.name());
        let (pid, attempts, output) = mechanism.run(root, exe, &args)?;
        runs.push(Run {
            name,
            pid,
            attempts,
            output,
        });
    }
    if runs.is_empty() {
        bail!("no test executables of {PACKAGE} were found");
    }
    if mechanism.logged() {
        let waits: &[u64] = if self_test {
            &SELF_TEST_WAITS
        } else {
            &[SETTLE]
        };
        for wait in waits {
            std::thread::sleep(Duration::from_secs(*wait));
            let log = mechanism.read_log(started)?;
            for run in &mut runs {
                if let Some(pid) = run.pid {
                    run.attempts = mechanism.lines(&log, pid, &run.name);
                }
            }
            if runs.iter().all(|run| !run.attempts.is_empty()) {
                break;
            }
        }
    }
    if self_test && matches!(mechanism, Mechanism::Sandbox) {
        accept_sandbox_refusals(&mut runs);
    }
    let total: usize = runs.iter().map(|run| run.attempts.len()).sum();
    for run in &runs {
        println!(
            "network-deny: {} made {} connection attempt(s)",
            run.name,
            run.attempts.len()
        );
        for line in run.attempts.iter().take(5) {
            println!("    {line}");
        }
    }
    match (self_test, total) {
        (false, 0) => {
            println!(
                "network-deny: no connection attempts ({})",
                mechanism.name()
            );
            Ok(())
        }
        (false, n) => {
            bail!("{n} network connection attempt(s) without a consent gate (NFR-04, ADR-0012)")
        }
        (true, 0) => bail!(
            "self-test: the probe's deliberate connections were not seen, so {} does not \
             work on this machine",
            mechanism.name()
        ),
        (true, n) => {
            println!("network-deny: self-test saw the probe's {n} attempt(s); the harness works");
            Ok(())
        }
    }
}

/// Builds the package's test executables and returns their paths.
fn build(root: &Path) -> Result<Vec<PathBuf>> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .current_dir(root)
        .args([
            "test",
            "--locked",
            "--no-run",
            "--message-format=json",
            "-p",
            PACKAGE,
        ])
        .stderr(Stdio::inherit())
        .output()
        .context("running `cargo test --no-run`")?;
    if !output.status.success() {
        bail!("building the {PACKAGE} tests failed");
    }
    let mut executables = Vec::new();
    for message in Message::parse_stream(output.stdout.as_slice()) {
        if let Message::CompilerArtifact(artifact) = message.context("reading cargo's output")?
            && artifact.profile.test
            && let Some(exe) = artifact.executable
        {
            executables.push(exe.into_std_path_buf());
        }
    }
    executables.sort();
    Ok(executables)
}

#[derive(Clone, Copy, Debug)]
enum Mechanism {
    Strace,
    Sandbox,
    Wfp,
}

impl Mechanism {
    fn detect() -> Result<Self> {
        if cfg!(target_os = "linux") {
            Ok(Self::Strace)
        } else if cfg!(target_os = "macos") {
            Ok(Self::Sandbox)
        } else if cfg!(windows) {
            Ok(Self::Wfp)
        } else {
            bail!("network-deny has no mechanism for this OS")
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Strace => "strace",
            Self::Sandbox => "sandbox-exec",
            Self::Wfp => "Windows Filtering Platform auditing",
        }
    }

    fn prepare(self, root: &Path) -> Result<()> {
        match self {
            Self::Strace => {
                let ok = Command::new("strace")
                    .arg("-V")
                    .stdout(Stdio::null())
                    .status()
                    .is_ok_and(|status| status.success());
                if !ok {
                    bail!("strace is not installed (`sudo apt-get install strace`)");
                }
                Ok(())
            }
            Self::Sandbox => {
                let dir = work_dir(root)?;
                std::fs::write(
                    dir.join("deny.sb"),
                    "(version 1)\n(allow default)\n(deny network-outbound (remote ip \"*:*\"))\n",
                )
                .context("writing the sandbox profile")
            }
            Self::Wfp => {
                // The GUID is "Filtering Platform Connection", so the call works in any
                // display language.
                let status = Command::new("auditpol")
                    .args([
                        "/set",
                        "/subcategory:{0CCE9226-69AE-11D9-BED3-505054503030}",
                        "/success:enable",
                        "/failure:enable",
                    ])
                    .stdout(Stdio::null())
                    .status()
                    .context("running auditpol")?;
                if !status.success() {
                    bail!(
                        "auditpol could not enable connection auditing; run the harness from \
                         an elevated shell"
                    );
                }
                Ok(())
            }
        }
    }

    /// Whether attempts come from an OS log read after the tests.
    const fn logged(self) -> bool {
        matches!(self, Self::Sandbox | Self::Wfp)
    }

    /// Runs `exe` under the mechanism. strace returns its attempts at once; the log-based
    /// mechanisms return the process ID for [`Mechanism::lines`].
    fn run(
        self,
        root: &Path,
        exe: &Path,
        args: &[&str],
    ) -> Result<(Option<u32>, Vec<String>, String)> {
        match self {
            Self::Strace => {
                let log = work_dir(root)?.join("strace.log");
                let status = Command::new("strace")
                    .args([
                        "-f",
                        "-qq",
                        "-e",
                        "trace=connect,sendto,sendmsg,sendmmsg",
                        "-o",
                    ])
                    .arg(&log)
                    .arg(exe)
                    .args(args)
                    .env("SONARITHM_BACKEND", "null")
                    .status()
                    .context("running strace")?;
                check(exe, status.success())?;
                let text = std::fs::read_to_string(&log).context("reading the strace log")?;
                Ok((None, inet_lines(&text), String::new()))
            }
            Self::Sandbox => {
                let profile = work_dir(root)?.join("deny.sb");
                let child = Command::new("sandbox-exec")
                    .arg("-f")
                    .arg(&profile)
                    .arg(exe)
                    .args(args)
                    .env("SONARITHM_BACKEND", "null")
                    .stdout(Stdio::piped())
                    .spawn()
                    .context("running sandbox-exec")?;
                let pid = child.id();
                let finished = child.wait_with_output().context("waiting for the tests")?;
                let output = String::from_utf8_lossy(&finished.stdout).into_owned();
                print!("{output}");
                check(exe, finished.status.success())?;
                Ok((Some(pid), Vec::new(), output))
            }
            Self::Wfp => {
                let mut child = Command::new(exe)
                    .args(args)
                    .env("SONARITHM_BACKEND", "null")
                    .spawn()
                    .context("running the tests")?;
                let pid = child.id();
                let status = child.wait().context("waiting for the tests")?;
                check(exe, status.success())?;
                Ok((Some(pid), Vec::new(), String::new()))
            }
        }
    }

    /// The OS log since `started` (with a margin), for [`Mechanism::lines`].
    fn read_log(self, started: Instant) -> Result<String> {
        let output = match self {
            Self::Strace => return Ok(String::new()),
            Self::Sandbox => {
                let minutes = started.elapsed().as_secs() / 60 + 2;
                Command::new("log")
                    .args(["show", "--style", "compact", "--last"])
                    .arg(format!("{minutes}m"))
                    .args(["--predicate", "eventMessage CONTAINS \"network-outbound\""])
                    .output()
                    .context("running `log show`")?
            }
            Self::Wfp => {
                let seconds = started.elapsed().as_secs() + 60;
                let script = format!(
                    "Get-WinEvent -FilterHashtable @{{LogName='Security'; Id=5156,5157; \
                     StartTime=(Get-Date).AddSeconds(-{seconds})}} -ErrorAction SilentlyContinue | \
                     ForEach-Object {{ $d = @{{}}; ([xml]$_.ToXml()).Event.EventData.Data | \
                     ForEach-Object {{ if ($_.Name) {{ $d[$_.Name] = $_.'#text' }} }}; \
                     \"$($_.Id)`t$($d.ProcessID)`t$($d.Direction)`t$($d.DestAddress):$($d.DestPort)`t$($d.Application)\" }}"
                );
                Command::new("powershell")
                    .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                    .output()
                    .context("reading the Security event log")?
            }
        };
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// The attempts of process `pid` in `log`.
    fn lines(self, log: &str, pid: u32, exe: &str) -> Vec<String> {
        match self {
            Self::Strace => Vec::new(),
            Self::Sandbox => sandbox_lines(log, pid),
            Self::Wfp => wfp_lines(log, pid, exe),
        }
    }
}

/// The unified log can drop or delay the kernel's denial lines on a busy runner. The
/// probe's own EPERM shows that the sandbox refused its connections, which is what the
/// self-test has to prove.
fn accept_sandbox_refusals(runs: &mut [Run]) {
    for run in runs.iter_mut() {
        if run.attempts.is_empty() {
            run.attempts = refused_by_sandbox(&run.output);
            if !run.attempts.is_empty() {
                println!(
                    "network-deny: the unified log had no entry for {}; the probe reports \
                     that the sandbox refused its connections (EPERM)",
                    run.name
                );
            }
        }
    }
}

fn work_dir(root: &Path) -> Result<PathBuf> {
    let dir = root.join("target").join("network-deny");
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    Ok(dir)
}

fn check(exe: &Path, ok: bool) -> Result<()> {
    if ok {
        Ok(())
    } else {
        bail!("the tests in {} failed under the harness", exe.display())
    }
}

/// strace lines with an IPv4 or IPv6 address.
fn inet_lines(log: &str) -> Vec<String> {
    log.lines()
        .filter(|line| line.contains("sa_family=AF_INET"))
        .map(str::to_owned)
        .collect()
}

/// The probe's connections that the sandbox refused: its `network-probe` lines ending in
/// EPERM (`os error 1`).
fn refused_by_sandbox(output: &str) -> Vec<String> {
    output
        .lines()
        .filter(|line| line.starts_with("network-probe") && line.ends_with(": os error 1"))
        .map(str::to_owned)
        .collect()
}

/// Sandbox denials of the process `pid`, such as
/// `Sandbox: network_probe-1(4242) deny(1) network-outbound 192.0.2.1:9`.
fn sandbox_lines(log: &str, pid: u32) -> Vec<String> {
    let tag = format!("({pid})");
    log.lines()
        .filter(|line| line.contains("deny") && line.contains("network-outbound"))
        .filter(|line| line.contains(&tag))
        .map(str::to_owned)
        .collect()
}

/// Outbound WFP connection events (`%%14593`) of the process `pid`. Each line is
/// `id \t pid \t direction \t destination \t application`.
fn wfp_lines(log: &str, pid: u32, exe: &str) -> Vec<String> {
    let pid = pid.to_string();
    // Windows reuses process IDs, and the log is read back over a window that reaches
    // before this run (the self-test's probe runs one step earlier). An event counts only if
    // its application is this executable too (#409: the probe's attempts were pinned on the
    // `speech` tests, which had the probe's old PID). WFP reports a lower-case device path.
    let exe = exe.to_lowercase();
    log.lines()
        .filter(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            let application = fields.get(4).map(|path| path.to_lowercase());
            fields.get(1) == Some(&pid.as_str())
                && fields.get(2) == Some(&"%%14593")
                && application
                    .is_some_and(|path| path.rsplit(['\\', '/']).next() == Some(exe.as_str()))
        })
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strace_lines_with_inet_addresses_are_attempts() {
        let log = "\
4242 connect(3, {sa_family=AF_UNIX, sun_path=\"/run/user/1000/pulse/native\"}, 110) = 0
4243 connect(5, {sa_family=AF_INET, sin_port=htons(9), sin_addr=inet_addr(\"192.0.2.1\")}, 16) = -1 EINPROGRESS
4243 sendto(6, \"x\", 1, 0, {sa_family=AF_INET6, sin6_port=htons(9)}, 28) = 1
4243 sendto(7, \"x\", 1, 0, NULL, 0) = 1";
        assert_eq!(inet_lines(log).len(), 2);
    }

    #[test]
    fn only_the_probes_eperm_counts_as_a_sandbox_refusal() {
        let output = "running 1 test\nnetwork-probe tcp: os error 1\nnetwork-probe udp: os error 1\n\
                      network-probe tcp: TimedOut\nnetwork-probe udp: os error 13\n";
        assert_eq!(refused_by_sandbox(output).len(), 2);
        assert_eq!(
            refused_by_sandbox("network-probe tcp: sent"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn sandbox_denials_count_only_for_the_process() {
        let log = "\
2026-10-21 10:00:00.1 E  kernel[0:1] Sandbox: network_probe-1(4242) deny(1) network-outbound 192.0.2.1:9
2026-10-21 10:00:00.2 E  kernel[0:1] Sandbox: other(99) deny(1) network-outbound 10.0.0.1:443
2026-10-21 10:00:00.3 E  kernel[0:1] Sandbox: network_probe-1(4242) allow network-outbound /private/var/run/syslog";
        assert_eq!(sandbox_lines(log, 4242).len(), 1);
    }

    #[test]
    fn wfp_events_count_only_outbound_ones_of_the_process() {
        let log = "\
5156\t4242\t%%14593\t192.0.2.1:9\t\\device\\harddiskvolume2\\t\\network_probe.exe
5156\t4242\t%%14592\t127.0.0.1:50000\t\\device\\harddiskvolume2\\t\\network_probe.exe
5157\t77\t%%14593\t10.0.0.1:443\t\\device\\harddiskvolume2\\other.exe";
        assert_eq!(wfp_lines(log, 4242, "network_probe.exe").len(), 1);
    }

    #[test]
    fn wfp_events_of_an_earlier_process_with_the_same_pid_do_not_count() {
        // #409: the self-test's probe ran as PID 6916; the `speech` tests later ran as 6916
        // too, and the log window still held the probe's two deliberate attempts.
        let log = "\
5156\t6916\t%%14593\t192.0.2.1:9\t\\device\\harddiskvolume4\\a\\target\\debug\\deps\\network_probe-3f77.exe
5156\t6916\t%%14593\t192.0.2.1:9\t\\device\\harddiskvolume4\\a\\target\\debug\\deps\\network_probe-3f77.exe";
        assert_eq!(
            wfp_lines(log, 6916, "speech-c7bd.exe"),
            Vec::<String>::new()
        );
        assert_eq!(wfp_lines(log, 6916, "network_probe-3f77.exe").len(), 2);
        // The file name is matched without regard to case, as WFP lower-cases the path.
        assert_eq!(wfp_lines(log, 6916, "Network_Probe-3F77.exe").len(), 2);
    }
}
