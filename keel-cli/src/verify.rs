//! `keel verify` (D0476): the pre-commit checks as ONE ladder, in cost order, stopping at the first red.
//!
//! The five checks existed before this command - `gate validate` (under a second), `gate guard`
//! (25 s, or its D0371 receipt), `cargo clippy -D warnings` (about a minute), the sprint's D0388
//! probe pair, and `suite --touched` (7-20 minutes). What did not exist was an ORDER anyone could
//! rely on: in sprint705 a clippy lint was discovered after a twenty-minute touched run because the
//! five ran as separate commands in whatever order the operator typed them, and the fix cost a second
//! twenty-minute run. With the D0425 verifier suspended (single-agent mode, 2026-09-13) the order was
//! the AI's discipline alone, and manual vigilance is not a control (D0047). The panel of 2026-09-13
//! reframed the sequence: it is not a mechanism until it is one command.
//!
//! This module adds NO check. It sequences the five that exist, refuses to continue past a red, and
//! writes one receipt naming the rung it stopped at - the single artifact a test-method result cites
//! (D0232). On green the touched receipt is the one `keel land` honours (D0474), so a green ladder
//! followed by a land runs nothing twice.
//!
//! Each rung runs as a child process with the operator's terminal inherited, so the rung's own output
//! is what the operator reads; this module records only exit codes, seconds and the command line.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// The ladder's receipt, machine-local beside the suite's and the touched run's.
pub const RECEIPT: &str = ".keel/metrics/verify-receipt.toml";

/// The triple CI lints (ci.yml runs clippy on ubuntu-latest).
///
/// A host that is not it never compiles a `#[cfg(not(windows))]` body, so its clippy proves nothing
/// about that body: sprint 725 landed green through every local clippy and CI failed it on
/// touched.rs:662 (issue572, D0495). The clippy rung therefore lints this triple a second time on any
/// other host.
pub const CI_TRIPLE: &str = "x86_64-unknown-linux-gnu";

/// Whether the binary running the ladder was built for [`CI_TRIPLE`] - keel runs on the host it was
/// built for, so the compile-time answer is the host's.
const HOST_IS_CI_TRIPLE: bool = cfg!(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"));

/// What the clippy rung does after the host's own clippy is green.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtherHostLint {
    /// The host IS the CI triple: one clippy is CI's clippy.
    Once,
    /// Lint again with `--target CI_TRIPLE`.
    Again,
    /// The host is not the CI triple and lacks that target's std: the rung is RED with this remedy,
    /// never skipped (D0098 - a check that cannot run must never pass silently).
    Refused(String),
}

/// The pure control (D0495): given whether the host is the CI triple and the toolchain's installed
/// targets, say whether the rung lints once, again, or cannot.
#[must_use]
pub fn other_host_lint(host_is_ci: bool, installed: &[&str]) -> OtherHostLint {
    if host_is_ci {
        OtherHostLint::Once
    } else if installed.iter().any(|t| t.trim() == CI_TRIPLE) {
        OtherHostLint::Again
    } else {
        OtherHostLint::Refused(format!(
            "clippy cannot lint {CI_TRIPLE} - the triple CI lints - because its std is not installed on this host, so every #[cfg(not(windows))] body would reach CI unlinted (issue572). REMEDY: rustup target add {CI_TRIPLE}"
        ))
    }
}

/// The toolchain's installed targets, one per line from `rustup target list --installed`; a toolchain
/// without rustup answers nothing, and nothing means the other-host lint is refused, not skipped.
fn installed_targets() -> Vec<String> {
    Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::to_owned).collect())
        .unwrap_or_default()
}

/// One rung of the ladder. Declaration order IS cost order IS run order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rung {
    /// `keel gate validate ROOT` - the `.tracking` authority.
    Validate,
    /// `keel gate guard ROOT` - every forward guard, answering from its receipt when inputs are equal (D0371).
    Guard,
    /// `cargo clippy --release --workspace --all-targets -- -D warnings`, as ci.yml runs it, then again with
    /// `--target x86_64-unknown-linux-gnu` when this host is not that triple (D0495, issue572).
    Clippy,
    /// The D0388 pair named by `--probe POSITIVE,NEGATIVE`: two commands, both must exit 0.
    Probe,
    /// `keel suite --touched ROOT` - the set `keel land` runs, keyed per D0474.
    Touched,
}

impl Rung {
    /// Every rung, cheapest first.
    pub const ORDER: [Self; 5] = [Self::Validate, Self::Guard, Self::Clippy, Self::Probe, Self::Touched];

    /// The receipt's name for the rung.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Validate => "validate",
            Self::Guard => "guard",
            Self::Clippy => "clippy",
            Self::Probe => "probe",
            Self::Touched => "touched",
        }
    }

    fn from_name(s: &str) -> Option<Self> {
        Self::ORDER.iter().copied().find(|r| r.name() == s)
    }
}

/// What one rung came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Exit 0.
    Pass,
    /// A non-zero exit (or a process that could not be started, recorded as 2).
    Fail(i32),
    /// Never started: an earlier rung was red.
    NotRun,
    /// The probe rung when no `--probe` pair was named - recorded as such, never invented (D0388).
    NotNamed,
}

impl Verdict {
    const fn is_red(&self) -> bool {
        matches!(self, Self::Fail(_))
    }

    fn label(&self) -> String {
        match self {
            Self::Pass => "pass".to_owned(),
            Self::Fail(code) => format!("fail ({code})"),
            Self::NotRun => "not-run".to_owned(),
            Self::NotNamed => "not-named".to_owned(),
        }
    }

    const fn receipt_word(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail(_) => "fail",
            Self::NotRun => "not-run",
            Self::NotNamed => "not-named",
        }
    }
}

/// One rung as the receipt records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub rung: Rung,
    pub verdict: Verdict,
    pub seconds: u64,
    /// The command line that ran (or would have), for whoever re-runs the rung by hand.
    pub command: String,
}

/// The ladder as the receipt holds it - in flight or finished.
///
/// `in_flight` is the rung running NOW: `Some` on the stub rewritten before every rung (issue565 -
/// the D0387 running form the suite and touched receipts already carry), `None` once the ladder ends.
/// A reader during a run, or after a killed one, sees `outcome = "running"` and the rungs finished
/// so far - never the previous run's verdict, which is what the sprint 721 verifier read as its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ladder {
    pub head: String,
    pub at: u64,
    pub seconds: u64,
    pub steps: Vec<Step>,
    /// The rung the ladder stopped at; `None` is a green ladder.
    pub stopped_at: Option<Rung>,
    /// The rung running when this receipt was written; `None` is a finished ladder.
    pub in_flight: Option<Rung>,
    /// The process that wrote the receipt (issue569): a second `keel verify` reads it and refuses
    /// while that process is alive. `0` on a receipt from before the pid rode it.
    pub pid: u32,
}

impl Ladder {
    /// Green when the ladder has ended and no rung is red. A running ladder is not green yet.
    #[must_use]
    pub const fn green(&self) -> bool {
        self.stopped_at.is_none() && self.in_flight.is_none()
    }

    /// The receipt's `outcome` word: `running` while a rung is in flight, else `pass` or `fail`.
    #[must_use]
    pub const fn outcome(&self) -> &'static str {
        match (self.in_flight, self.stopped_at) {
            (Some(_), _) => "running",
            (None, None) => "pass",
            (None, Some(_)) => "fail",
        }
    }

    /// The failing rung's exit code, so `keel verify` exits as the rung did.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        self.steps.iter().find_map(|s| match s.verdict {
            Verdict::Fail(c) => Some(c),
            _ => None,
        }).unwrap_or(0)
    }
}

/// The D0388 pair the probe rung runs, with where it came from.
///
/// `from` is the `--probe-from` file the pair was read from (D0500), or `None` for a pair typed on
/// the command line. The receipt's probe rung row names it, so a reader can diff the file the
/// verifier was handed against the two lines the rung actually ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The known-positive command line.
    pub pos: String,
    /// The known-negative command line.
    pub neg: String,
    /// The file both lines were read from, when they were.
    pub from: Option<PathBuf>,
}

impl Probe {
    /// The rung's command text: both sides, prefixed by the file when there was one.
    #[must_use]
    pub fn text(&self) -> String {
        self.from.as_ref().map_or_else(
            || format!("{} ; {}", self.pos, self.neg),
            |f| format!("--probe-from {}: {} ; {}", f.display(), self.pos, self.neg),
        )
    }
}

/// The D0388 pair as named on the command line: `--probe POSITIVE,NEGATIVE` or `--probe-from FILE`.
///
/// Each side is a command line split on whitespace and run in the project root; both must exit 0.
/// `None` when neither flag is present (the rung is then recorded `not-named`); an error when a flag
/// is present but does not name exactly two non-empty sides - a pair with one side is not a pair, and
/// a check probed on one case is the D0388 defect this command exists to refuse.
///
/// `--probe-from FILE` (D0500, issue571): the file holds exactly two non-empty lines, the positive
/// then the negative, written by the actor who chose them before reading the tree; the verifier
/// names the path and transcribes nothing. Both flags together are refused - two sources for one
/// pair is the ambiguity the file exists to remove.
///
/// # Errors
///
/// A flag with no value, an empty side, a side count other than two, an unreadable file, or both
/// flags given.
pub fn parse_probe(args: &[String]) -> Result<Option<Probe>, String> {
    let typed = args.iter().position(|a| a == "--probe");
    let from = args.iter().position(|a| a == "--probe-from");
    match (typed, from) {
        (Some(_), Some(_)) => Err("--probe and --probe-from name the same pair twice; give one (D0500)".to_owned()),
        (None, None) => Ok(None),
        (Some(i), None) => {
            let Some(value) = args.get(i + 1) else {
                return Err("--probe takes POSITIVE,NEGATIVE - two command lines, comma-separated (D0388)".to_owned());
            };
            let sides: Vec<&str> = value.split(',').map(str::trim).collect();
            match sides.as_slice() {
                [pos, neg] if !pos.is_empty() && !neg.is_empty() => Ok(Some(Probe { pos: (*pos).to_owned(), neg: (*neg).to_owned(), from: None })),
                _ => Err(format!("--probe names {} side(s); a D0388 pair is exactly two, POSITIVE,NEGATIVE, both non-empty: `{value}`", sides.iter().filter(|s| !s.is_empty()).count())),
            }
        }
        (None, Some(i)) => {
            let Some(path) = args.get(i + 1) else {
                return Err("--probe-from takes FILE - two lines, the known-positive command then the known-negative (D0500)".to_owned());
            };
            let text = std::fs::read_to_string(path).map_err(|e| format!("--probe-from {path}: cannot read it: {e}"))?;
            parse_probe_text(&text, Path::new(path)).map(Some)
        }
    }
}

/// The pair as a `--probe-from` file's text: exactly two lines, neither blank.
///
/// A trailing newline is not a third line; a blank line anywhere is - a file that a comment, a
/// heading or an empty line has crept into is not the pair the primary wrote, and the rung would
/// otherwise run `""` or the heading as a command.
///
/// # Errors
///
/// A line count other than two, or a line that is blank after trimming.
pub fn parse_probe_text(text: &str, path: &Path) -> Result<Probe, String> {
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    let [pos, neg] = lines[..] else {
        return Err(format!("--probe-from {}: {} line(s); a D0388 pair file is exactly two, the known-positive command line then the known-negative", path.display(), lines.len()));
    };
    if let Some(n) = lines.iter().position(|l| l.is_empty()) {
        return Err(format!("--probe-from {}: line {} is blank; both lines of a D0388 pair file are command lines", path.display(), n + 1));
    }
    Ok(Probe { pos: pos.to_owned(), neg: neg.to_owned(), from: Some(path.to_path_buf()) })
}

/// The arguments the ladder itself consumes, so a root can be found among what is left.
#[must_use]
pub fn own_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if a == "--probe" || a == "--probe-from" {
            skip = true;
            continue;
        }
        if a == "--no-receipt" || a == "--wait" || a == "--help" || a == "-h" {
            continue;
        }
        out.push(a.clone());
    }
    out
}

/// The pure ladder: run each rung through `runner`, in `Rung::ORDER`, and STOP at the first red.
///
/// `runner` returns the rung's verdict and the command line it ran. Rungs after a red are recorded
/// `NotRun` with their command line and zero seconds, so the receipt says what was NOT measured as
/// plainly as what was. `probe` is the named pair's text (`Probe::text`): the probe rung is asked of
/// the runner only when a pair was named, and a not-run probe row still carries that text, so a
/// verifier reading a ladder that stopped below the pair copies the pair it was given, not the flag's
/// usage (sprint 730's first receipt said `not named by the dispatch` over a named file). With no pair
/// the rung is `NotNamed` and the ladder continues - an unnamed pair is a fact about the invocation,
/// not a red about the tree.
///
/// `progress` is called with the rungs finished so far BEFORE each rung runs - the hook that rewrites
/// the receipt as a running stub (issue565), so the file on disk never holds a previous run's verdict
/// while this one is in flight.
pub fn climb<F, P>(probe: Option<&str>, mut runner: F, mut progress: P) -> (Vec<Step>, Option<Rung>)
where
    F: FnMut(Rung) -> (Verdict, String),
    P: FnMut(&[Step], Rung),
{
    let mut steps = Vec::with_capacity(Rung::ORDER.len());
    let mut stopped_at = None;
    for rung in Rung::ORDER {
        if stopped_at.is_some() {
            let command = match (rung, probe) {
                (Rung::Probe, Some(pair)) => pair.to_owned(),
                _ => command_text(rung),
            };
            steps.push(Step { rung, verdict: Verdict::NotRun, seconds: 0, command });
            continue;
        }
        if rung == Rung::Probe && probe.is_none() {
            steps.push(Step { rung, verdict: Verdict::NotNamed, seconds: 0, command: "--probe POSITIVE,NEGATIVE / --probe-from FILE not given".to_owned() });
            continue;
        }
        progress(&steps, rung);
        let started = Instant::now();
        let (verdict, command) = runner(rung);
        let seconds = started.elapsed().as_secs();
        if verdict.is_red() {
            stopped_at = Some(rung);
        }
        steps.push(Step { rung, verdict, seconds, command });
    }
    (steps, stopped_at)
}

/// The command line a rung runs, as the receipt prints it for a rung that did not run.
fn command_text(rung: Rung) -> String {
    match rung {
        Rung::Validate => "keel gate validate ROOT".to_owned(),
        Rung::Guard => "keel gate guard ROOT".to_owned(),
        Rung::Clippy if HOST_IS_CI_TRIPLE => "cargo clippy --release --workspace --all-targets -- -D warnings".to_owned(),
        Rung::Clippy => format!("cargo clippy --release --workspace --all-targets -- -D warnings ; cargo clippy --release --workspace --all-targets --target {CI_TRIPLE} -- -D warnings"),
        Rung::Probe => "--probe POSITIVE,NEGATIVE / --probe-from FILE".to_owned(),
        Rung::Touched => "keel suite --touched ROOT".to_owned(),
    }
}

/// Run one command line with the terminal inherited; the verdict is its exit status.
fn status_of(cmd: &mut Command) -> Verdict {
    match cmd.status() {
        Ok(s) if s.success() => Verdict::Pass,
        Ok(s) => Verdict::Fail(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("keel verify: could not start {}: {e}", cmd.get_program().display());
            Verdict::Fail(2)
        }
    }
}

/// A whitespace-split command line as a `Command`, run in `repo`.
fn shell_free(line: &str, repo: &Path) -> Option<Command> {
    let mut parts = line.split_whitespace();
    let program = parts.next()?;
    let mut cmd = Command::new(program);
    cmd.args(parts).current_dir(repo);
    Some(cmd)
}

/// Every child is spawned without the operator's profiling variable (issue539): the perf report goes
/// to stderr at process end, and a rung's stderr is the operator's to read, not a parser's to trip on.
fn scrub(cmd: &mut Command) -> &mut Command {
    cmd.env_remove("KEEL_PERF")
}

/// The real runner: each rung as a child process, the terminal inherited.
fn run_rung(rung: Rung, repo: &Path, exe: &Path, probe: Option<&Probe>, forced: bool) -> (Verdict, String) {
    let keel = |args: &[&str]| -> (Verdict, String) {
        let mut cmd = Command::new(exe);
        cmd.args(args).arg(repo);
        let text = format!("{} {} {}", exe.display(), args.join(" "), repo.display());
        (status_of(scrub(&mut cmd)), text)
    };
    match rung {
        Rung::Validate => keel(&["gate", "validate"]),
        Rung::Guard => {
            if forced {
                keel(&["gate", "guard", "--no-receipt"])
            } else {
                keel(&["gate", "guard"])
            }
        }
        Rung::Clippy => {
            let clippy = |target: Option<&str>| -> Command {
                let mut cmd = Command::new("cargo");
                // `--workspace`, as ci.yml runs it: a member's own test target (keel-write's scaffold
                // tests, say) is linted by CI and was not by a run over keel-cli's manifest alone.
                cmd.args(["clippy", "--release", "--workspace", "--all-targets", "--manifest-path"]).arg(repo.join("Cargo.toml"));
                if let Some(t) = target {
                    cmd.args(["--target", t]);
                }
                cmd.args(["--", "-D", "warnings"]).current_dir(repo);
                cmd
            };
            let host_line = "cargo clippy --release --workspace --all-targets -- -D warnings";
            let v = status_of(scrub(&mut clippy(None)));
            if v.is_red() {
                return (v, host_line.to_owned());
            }
            // D0495: the host's clippy is green; now the triple CI lints, unless this host is it.
            let installed = installed_targets();
            let installed: Vec<&str> = installed.iter().map(String::as_str).collect();
            match other_host_lint(HOST_IS_CI_TRIPLE, &installed) {
                OtherHostLint::Once => (v, host_line.to_owned()),
                OtherHostLint::Again => {
                    println!("keel verify: clippy again for {CI_TRIPLE}, the triple CI lints (D0495)");
                    (status_of(scrub(&mut clippy(Some(CI_TRIPLE)))), command_text(Rung::Clippy))
                }
                OtherHostLint::Refused(remedy) => {
                    eprintln!("keel verify: {remedy}");
                    (Verdict::Fail(2), command_text(Rung::Clippy))
                }
            }
        }
        Rung::Probe => {
            let Some(pair) = probe else {
                return (Verdict::NotNamed, command_text(Rung::Probe));
            };
            let text = pair.text();
            for side in [&pair.pos, &pair.neg] {
                let Some(mut cmd) = shell_free(side, repo) else {
                    eprintln!("keel verify: probe side is empty");
                    return (Verdict::Fail(2), text);
                };
                println!("keel verify: probe `{side}`");
                let v = status_of(scrub(&mut cmd));
                if v.is_red() {
                    return (v, text);
                }
            }
            (Verdict::Pass, text)
        }
        Rung::Touched => {
            if forced {
                keel(&["suite", "--touched", "--no-receipt"])
            } else {
                keel(&["suite", "--touched"])
            }
        }
    }
}

/// The receipt text. Pure, tested by round trip.
#[must_use]
pub fn render_receipt(l: &Ladder) -> String {
    use std::fmt::Write as _;
    let mut s = format!(
        "# verify receipt (D0476): the pre-commit ladder - validate, guard, clippy, the D0388 probe pair, the\n# touched run - in cost order, stopped at the first red. `stopped_at = \"none\"` is a green ladder; a rung\n# after the red is `not-run`; a probe rung with no --probe pair is `not-named`, never invented. On green\n# the touched receipt ({}) is the one keel land honours (D0474).\n# `outcome = \"running\"` is the stub rewritten before every rung (D0387, issue565): a ladder in flight, or\n# one that was killed - `running` names the rung, the [[rung]] blocks are the ones finished so far, and\n# `at` is when this file was WRITTEN (the launch, for the first stub) - never the previous run's verdict.\nhead = \"{}\"\nat = {}\nseconds = {}\noutcome = \"{}\"\nstopped_at = \"{}\"\nrunning = \"{}\"\npid = {}\n",
        crate::touched::RECEIPT,
        l.head,
        l.at,
        l.seconds,
        l.outcome(),
        l.stopped_at.map_or("none", Rung::name),
        l.in_flight.map_or("none", Rung::name),
        l.pid,
    );
    for st in &l.steps {
        let code = match st.verdict {
            Verdict::Fail(c) => c,
            _ => 0,
        };
        let _ = write!(s, "\n[[rung]]\nname = \"{}\"\nverdict = \"{}\"\nexit = {}\nseconds = {}\ncommand = \"{}\"\n", st.rung.name(), st.verdict.receipt_word(), code, st.seconds, st.command.replace('\\', "/").replace('"', "'"));
    }
    s
}

fn value_of<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|l| l.strip_prefix(key).and_then(|r| r.trim_start().strip_prefix('=')).map(str::trim)).map(|v| v.trim_matches('"'))
}

/// Read a receipt back. `None` for text that is not one.
#[must_use]
pub fn parse_receipt(text: &str) -> Option<Ladder> {
    let (head_block, rungs) = text.split_once("\n[[rung]]").unwrap_or((text, ""));
    let head = value_of(head_block, "head")?.to_owned();
    let at = value_of(head_block, "at")?.parse().ok()?;
    let seconds = value_of(head_block, "seconds")?.parse().ok()?;
    let stopped_at = match value_of(head_block, "stopped_at")? {
        "none" => None,
        other => Some(Rung::from_name(other)?),
    };
    // A receipt from before the running form has no `running` key: it was written at the end, so
    // nothing is in flight.
    let in_flight = match value_of(head_block, "running").unwrap_or("none") {
        "none" => None,
        other => Some(Rung::from_name(other)?),
    };
    let mut steps = Vec::new();
    for block in rungs.split("\n[[rung]]").filter(|b| !b.trim().is_empty()) {
        let rung = Rung::from_name(value_of(block, "name")?)?;
        let exit: i32 = value_of(block, "exit")?.parse().ok()?;
        let verdict = match value_of(block, "verdict")? {
            "pass" => Verdict::Pass,
            "fail" => Verdict::Fail(exit),
            "not-run" => Verdict::NotRun,
            "not-named" => Verdict::NotNamed,
            _ => return None,
        };
        steps.push(Step { rung, verdict, seconds: value_of(block, "seconds")?.parse().ok()?, command: value_of(block, "command")?.to_owned() });
    }
    let pid = value_of(head_block, "pid").and_then(|p| p.parse().ok()).unwrap_or(0);
    Some(Ladder { head, at, seconds, steps, stopped_at, in_flight, pid })
}

/// One ladder at a time per tree (issue569).
///
/// The refusal line when the receipt on disk is a running stub whose writer `alive(pid)` holds
/// for. `None` for a finished ladder, a receipt from before the pid rode it, or a writer that is
/// gone - a killed ladder's stub is replaced, as issue565 intended.
#[must_use]
pub fn exclusive_refusal(text: &str, alive: impl Fn(u32) -> bool) -> Option<String> {
    let l = parse_receipt(text)?;
    let rung = l.in_flight?;
    (l.pid != 0 && alive(l.pid)).then(|| {
        format!(
            "a ladder is already in flight - {RECEIPT} says running at rung {}, written by pid {} at {}. REFUSING to launch a second: two ladders reach the touched run together and race on its receipt and the tests' scratch dirs (issue569). Nothing was written; wait for that process to exit. A stub whose pid is gone is a killed ladder and is replaced.",
            rung.name(),
            l.pid,
            l.at
        )
    })
}

/// What one read of the receipt tells `keel verify --wait` (issue575).
///
/// The sprint 726 verifier read the stub 17 s after launch, saw `running` with a pid it did not
/// test, and reported a green 718 s ladder as killed: the wait was a sentence in a skill, so it was
/// skipped. This is the wait as a control - the receipt text and one liveness answer decide, and
/// nothing here is a verdict until the ladder has written one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitState {
    /// No file, or text that is not a receipt: nothing to wait for and nothing to report.
    NoReceipt,
    /// A rung in flight and the process that wrote the stub alive: keep waiting.
    InFlight { rung: Rung, pid: u32 },
    /// A `running` stub whose writer is gone (or, from before the pid rode it, unnamed): the ladder
    /// died mid-rung. Not a pass, not a fail - the rung it died in is the whole report.
    Killed { rung: Rung, pid: u32 },
    /// The ladder ended and wrote its verdict.
    Finished(Ladder),
}

/// The pure control behind `--wait`: `text` is the receipt on disk, `alive` answers for its `pid`.
#[must_use]
pub fn wait_state(text: &str, alive: impl Fn(u32) -> bool) -> WaitState {
    let Some(l) = parse_receipt(text) else {
        return WaitState::NoReceipt;
    };
    match l.in_flight {
        None => WaitState::Finished(l),
        Some(rung) if l.pid != 0 && alive(l.pid) => WaitState::InFlight { rung, pid: l.pid },
        Some(rung) => WaitState::Killed { rung, pid: l.pid },
    }
}

/// The rung table and the summary line, exiting as the ladder did - what `keel verify` prints at the
/// end of its own run and what `--wait` prints when the run it waited on ends.
fn report(ladder: &Ladder) -> i32 {
    for st in &ladder.steps {
        println!("  {:<9} {:<12} {:>5} s", st.rung.name(), st.verdict.label(), st.seconds);
    }
    ladder.stopped_at.map_or_else(
        || {
            println!("keel verify: pass - every rung green in {} s; receipt {RECEIPT}", ladder.seconds);
            0
        },
        |r| {
            println!("keel verify: fail - stopped at {} after {} s; receipt {RECEIPT}", r.name(), ladder.seconds);
            ladder.exit_code()
        },
    )
}

/// `keel verify --wait`: block on the receipt until its writer exits, then report what it wrote.
///
/// One line per rung change while in flight; the finished receipt's table and exit code when the
/// ladder ends (0 green, the red rung's code otherwise); `KILLED during <rung>` and exit 2 for a
/// stub whose writer is gone - never a verdict on either side (issue575). Exit 2 with no receipt.
fn wait(repo: &Path) -> i32 {
    let started = Instant::now();
    let mut last: Option<Rung> = None;
    loop {
        let text = std::fs::read_to_string(repo.join(RECEIPT)).unwrap_or_default();
        match wait_state(&text, crate::touched::pid_alive) {
            WaitState::NoReceipt => {
                eprintln!("keel verify --wait: no ladder receipt at {RECEIPT} - nothing is in flight and nothing has been judged; launch `keel verify` first");
                return 2;
            }
            WaitState::InFlight { rung, pid } => {
                if last != Some(rung) {
                    println!("keel verify --wait: in flight: {}, pid {pid} alive, {} s", rung.name(), started.elapsed().as_secs());
                    last = Some(rung);
                }
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            WaitState::Killed { rung, pid } => {
                println!("keel verify --wait: KILLED during {} - {RECEIPT} says running and its writer (pid {pid}) is gone. Not a verdict on either side; the rungs before it are the ones the receipt holds. Relaunch `keel verify`.", rung.name());
                return 2;
            }
            WaitState::Finished(l) => {
                println!("keel verify --wait: the ladder that wrote {RECEIPT} has ended (head {}, at {})", l.head, l.at);
                return report(&l);
            }
        }
    }
}

/// Write the receipt, naming the failure on stderr; the ladder itself does not stop for it.
fn write_receipt(repo: &Path, ladder: &Ladder) {
    let path = repo.join(RECEIPT);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = crate::write::write_atomic(&path, render_receipt(ladder)) {
        eprintln!("keel verify: receipt could not be written: {e}");
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn print_help() {
    println!("usage: keel verify [ROOT] [--probe POSITIVE,NEGATIVE | --probe-from FILE] [--no-receipt] | --wait [ROOT]");
    println!("  the pre-commit checks as one ladder in cost order, stopping at the first red (D0476):");
    println!("    1. keel gate validate       2. keel gate guard (from its receipt, D0371)");
    println!("    3. cargo clippy --release --workspace --all-targets -- -D warnings (then --target x86_64-unknown-linux-gnu, the triple CI lints, on any other host; D0495)");
    println!("    4. the D0388 probe pair: --probe POSITIVE,NEGATIVE, or --probe-from FILE whose two lines are the known-positive");
    println!("       command then the known-negative, written by the actor who chose them (D0500) - both must exit 0;");
    println!("       any other line count, a blank line, or both flags together is refused before a rung runs");
    println!("    5. keel suite --touched (binaries observed green at this content are skipped, D0474)");
    println!("  Writes {RECEIPT} naming the rung it stopped at; a rung after the red is not-run, a probe rung");
    println!("  with no pair is not-named. Exits as the failing rung did. --no-receipt forces guard and the");
    println!("  touched run to run everything. On green the touched receipt is the one keel land honours.");
    println!("  While a rung runs the receipt says outcome = \"running\" and names the rung (D0387): a reader");
    println!("  mid-run waits; it never sees the previous run's verdict. The stub names its writer (pid): a second");
    println!("  keel verify is refused (exit 2) while that process is alive; a dead writer's stub is replaced (issue569).");
    println!("  --wait launches nothing: it blocks on the receipt while its writer's pid is alive, printing one line per rung");
    println!("  change, then prints the finished table and exits as the ladder did; a running stub whose pid is gone is");
    println!("  KILLED during <rung>, exit 2 - never a verdict on either side (issue575). Exit 2 with no receipt.");
}

/// `keel verify`.
#[must_use]
pub fn cmd(args: &[String], repo: &Path) -> i32 {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return 0;
    }
    if args.iter().any(|a| a == "--wait") {
        // issue575: a reader, not a launcher. A pair or --no-receipt beside it would be silently
        // ignored, which is how a verifier comes to believe it ran something; refuse instead.
        if let Some(extra) = args.iter().find(|a| *a == "--probe" || *a == "--probe-from" || *a == "--no-receipt") {
            eprintln!("keel verify: --wait reads the receipt of a ladder already launched; `{extra}` belongs to the launch, not the wait");
            return 2;
        }
        return wait(repo);
    }
    let probe = match parse_probe(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("keel verify: {e}");
            return 2;
        }
    };
    if !crate::suite::is_self_build(repo) {
        eprintln!("keel verify: {} holds no keel-cli/Cargo.toml - the clippy and touched rungs have nothing to run here (a downstream project's gate is `keel gate`)", repo.display());
        return 2;
    }
    // issue150: the touched rung relinks the binary; refuse from the image it would overwrite BEFORE
    // the cheap rungs spend their minute, not after.
    if let Some(reason) = crate::suite::own_image_refusal(repo, "keel verify") {
        eprintln!("{reason}");
        return 2;
    }
    let exe: PathBuf = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("keel verify: cannot resolve this binary: {e}");
            return 2;
        }
    };
    // issue569: the receipt on disk is READ before this ladder writes its first stub. A live
    // ladder's stub refuses this launch - two ladders would reach the touched run together.
    if let Some(line) = exclusive_refusal(&std::fs::read_to_string(repo.join(RECEIPT)).unwrap_or_default(), crate::touched::pid_alive) {
        eprintln!("keel verify: {line}");
        return 2;
    }
    let forced = crate::receipt::forced(args);
    let head = crate::gitx::git().arg("-C").arg(repo).args(["rev-parse", "--short", "HEAD"]).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let started = Instant::now();
    let probe_text = probe.as_ref().map(Probe::text);
    let (steps, stopped_at) = climb(
        probe_text.as_deref(),
        |rung| {
            println!("keel verify: rung {} - {}", rung.name(), command_text(rung));
            run_rung(rung, repo, &exe, probe.as_ref(), forced)
        },
        // issue565: the receipt says `running` from before the first rung starts, so a reader mid-run
        // (the D0425 verifier) waits instead of taking the previous run's verdict for this one.
        |so_far, rung| {
            write_receipt(repo, &Ladder { head: head.clone(), at: now_secs(), seconds: started.elapsed().as_secs(), steps: so_far.to_vec(), stopped_at: None, in_flight: Some(rung), pid: std::process::id() });
        },
    );
    let ladder = Ladder { head, at: now_secs(), seconds: started.elapsed().as_secs(), steps, stopped_at, in_flight: None, pid: std::process::id() };
    write_receipt(repo, &ladder);
    report(&ladder)
}

#[cfg(test)]
mod tests {
    use super::{climb, exclusive_refusal, other_host_lint, own_args, parse_probe, parse_probe_text, parse_receipt, render_receipt, wait_state, Ladder, OtherHostLint, Probe, Rung, Step, Verdict, WaitState, CI_TRIPLE};

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    /// D0388 known-positive: a red clippy stops the ladder BEFORE the touched run - the runner is never
    /// asked for the touched rung, and the receipt records probe and touched as not-run. This is the
    /// sprint705 shape (a lint found after a twenty-minute run) refused by construction. The not-run
    /// probe row names the pair it was given, not the flag's usage (sprint 730's first receipt).
    #[test]
    fn a_red_rung_stops_the_ladder_before_the_test_binaries_compile() {
        let mut asked = Vec::new();
        let (steps, stopped) = climb(Some("--probe-from pair.txt: cargo test a ; cargo test b"), |r| {
            asked.push(r);
            let v = if r == Rung::Clippy { Verdict::Fail(101) } else { Verdict::Pass };
            (v, format!("cmd {}", r.name()))
        }, |_, _| {});
        assert_eq!(stopped, Some(Rung::Clippy));
        assert_eq!(asked, vec![Rung::Validate, Rung::Guard, Rung::Clippy], "nothing after the red is run");
        let verdicts: Vec<_> = steps.iter().map(|s| s.verdict.clone()).collect();
        assert_eq!(verdicts, vec![Verdict::Pass, Verdict::Pass, Verdict::Fail(101), Verdict::NotRun, Verdict::NotRun]);
        assert_eq!(steps[3].command, "--probe-from pair.txt: cargo test a ; cargo test b", "the not-run probe row names the pair the dispatch gave");
        assert_eq!(steps[4].command, "keel suite --touched ROOT", "the not-run rung still names what it would have run");
    }

    /// D0388 known-negative: a green ladder asks every rung in cost order and reaches the touched run once.
    #[test]
    fn a_green_ladder_reaches_the_touched_run_last() {
        let mut asked = Vec::new();
        let (steps, stopped) = climb(Some("a ; b"), |r| {
            asked.push(r);
            (Verdict::Pass, r.name().to_owned())
        }, |_, _| {});
        assert_eq!(stopped, None);
        assert_eq!(asked, Rung::ORDER.to_vec());
        assert_eq!(asked.iter().filter(|r| **r == Rung::Touched).count(), 1);
        assert!(steps.iter().all(|s| s.verdict == Verdict::Pass));
    }

    /// D0388 known-positive (D0495, issue572): a host that is not the CI triple and lacks its std is
    /// REFUSED naming the rustup remedy - the sprint 725 shape (a `#[cfg(not(windows))]` lint reaching
    /// CI unlinted) is red here, never a skipped check.
    #[test]
    fn a_host_without_the_ci_triples_std_is_refused_with_the_remedy() {
        let OtherHostLint::Refused(remedy) = other_host_lint(false, &["x86_64-pc-windows-msvc", "wasm32-unknown-unknown"]) else {
            panic!("a missing std must refuse, not skip");
        };
        assert!(remedy.contains(&format!("rustup target add {CI_TRIPLE}")), "{remedy}");
        assert!(remedy.contains("issue572"), "{remedy}");
        assert_eq!(other_host_lint(false, &[]), OtherHostLint::Refused(remedy), "no rustup at all is the same refusal");
    }

    /// D0388 known-negative: a host that is not the CI triple but has its std lints AGAIN; the CI triple
    /// itself lints once - its clippy already is CI's.
    #[test]
    fn a_host_with_the_ci_triples_std_lints_again_and_the_ci_triple_once() {
        assert_eq!(other_host_lint(false, &["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]), OtherHostLint::Again);
        assert_eq!(other_host_lint(false, &["  x86_64-unknown-linux-gnu\r"]), OtherHostLint::Again, "rustup's line endings are trimmed");
        assert_eq!(other_host_lint(true, &[]), OtherHostLint::Once);
        assert_eq!(other_host_lint(true, &["x86_64-unknown-linux-gnu"]), OtherHostLint::Once);
    }

    /// An unnamed pair is recorded, not invented (D0388): the probe rung is `not-named`, the runner is
    /// not asked for it, and the ladder continues to the touched run.
    #[test]
    fn an_unnamed_probe_pair_is_recorded_not_invented() {
        let mut asked = Vec::new();
        let (steps, stopped) = climb(None, |r| {
            asked.push(r);
            (Verdict::Pass, String::new())
        }, |_, _| {});
        assert_eq!(stopped, None);
        assert!(!asked.contains(&Rung::Probe));
        assert!(asked.contains(&Rung::Touched));
        assert_eq!(steps[3].verdict, Verdict::NotNamed);
    }

    /// `--probe` takes exactly a pair: two sides pass, one side is refused, absent is `None`.
    #[test]
    fn probe_flag_takes_exactly_a_pair() {
        assert_eq!(parse_probe(&args(&["--probe", "cargo test a, cargo test b"])).unwrap(), Some(Probe { pos: "cargo test a".into(), neg: "cargo test b".into(), from: None }));
        assert_eq!(parse_probe(&args(&["."])).unwrap(), None);
        assert!(parse_probe(&args(&["--probe", "cargo test a"])).unwrap_err().contains("1 side(s)"));
        assert!(parse_probe(&args(&["--probe", "a,"])).is_err());
        assert!(parse_probe(&args(&["--probe"])).is_err());
    }

    /// D0500 known-positive (issue571): a two-line file IS the pair - the positive first, the
    /// negative second, the source kept - and the rung's text names the file and both lines, so the
    /// receipt says where the pair came from. A trailing newline is not a third line, and a comma
    /// inside a command line survives, which `--probe POS,NEG` could never carry.
    #[test]
    fn a_probe_file_names_the_pair() {
        let dir = keel_fs::scratch("keel-verify-pair");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("pair.txt");
        std::fs::write(&file, "cargo test --release -p keel-cli --lib -- a::pos\ncargo test --release -p keel-cli --lib -- a::neg -- --exact,--nocapture\n").unwrap();
        let path = file.to_string_lossy().into_owned();
        let pair = parse_probe(&args(&["--probe-from", &path, "."])).unwrap().unwrap();
        assert_eq!(pair.pos, "cargo test --release -p keel-cli --lib -- a::pos");
        assert_eq!(pair.neg, "cargo test --release -p keel-cli --lib -- a::neg -- --exact,--nocapture");
        assert_eq!(pair.from.as_deref(), Some(file.as_path()));
        let text = pair.text();
        assert!(text.starts_with(&format!("--probe-from {}: ", file.display())), "{text}");
        assert!(text.contains("a::pos ; cargo test"), "{text}");
        // the typed form has no source and no prefix
        assert_eq!(Probe { pos: "a".into(), neg: "b".into(), from: None }.text(), "a ; b");
        // the path is the flag's value, never the root
        assert_eq!(own_args(&args(&["--probe-from", &path, "."])), vec![".".to_owned()]);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// D0500 known-negative: one line, three lines, a blank second line, an unreadable path, a flag
    /// with no value, and `--probe` beside `--probe-from` are each refused at parse naming the shape -
    /// before any rung runs, instead of eighty seconds later as `program not found` at the rung.
    #[test]
    fn a_probe_file_that_is_not_a_pair_is_refused() {
        let here = std::path::Path::new("pair.txt");
        assert!(parse_probe_text("cargo test a\n", here).unwrap_err().contains("1 line(s)"));
        assert!(parse_probe_text("a\nb\nc\n", here).unwrap_err().contains("3 line(s)"));
        assert!(parse_probe_text("a\n\n", here).unwrap_err().contains("line 2 is blank"));
        assert!(parse_probe_text("\nb\n", here).unwrap_err().contains("line 1 is blank"));
        assert!(parse_probe_text("a\n   \n", here).unwrap_err().contains("line 2 is blank"));
        assert!(parse_probe_text("a\nb\n\n", here).unwrap_err().contains("3 line(s)"));
        let dir = keel_fs::scratch("keel-verify-nopair");
        let missing = dir.join("absent.txt").to_string_lossy().into_owned();
        assert!(parse_probe(&args(&["--probe-from", &missing])).unwrap_err().contains("cannot read"));
        assert!(parse_probe(&args(&["--probe-from"])).unwrap_err().contains("takes FILE"));
        let both = parse_probe(&args(&["--probe", "a,b", "--probe-from", &missing])).unwrap_err();
        assert!(both.contains("twice"), "{both}");
    }

    /// The root is found among what the ladder does not consume: the probe value is never a path.
    #[test]
    fn the_probe_value_is_not_mistaken_for_a_root() {
        assert_eq!(own_args(&args(&["--probe", "cargo test a,cargo test b", ".", "--no-receipt"])), vec![".".to_owned()]);
        assert!(own_args(&args(&["--probe", "x,y"])).is_empty());
        assert!(own_args(&args(&["--probe-from", "pair.txt"])).is_empty());
    }

    /// The receipt round-trips: the rung stopped at, every rung's verdict, exit and seconds.
    #[test]
    fn receipt_round_trips() {
        let l = Ladder {
            head: "1e2ed25".into(),
            at: 1_789_376_056,
            seconds: 61,
            steps: vec![
                Step { rung: Rung::Validate, verdict: Verdict::Pass, seconds: 1, command: "keel gate validate .".into() },
                Step { rung: Rung::Guard, verdict: Verdict::Pass, seconds: 0, command: "keel gate guard .".into() },
                Step { rung: Rung::Clippy, verdict: Verdict::Fail(101), seconds: 60, command: "cargo clippy".into() },
                Step { rung: Rung::Probe, verdict: Verdict::NotRun, seconds: 0, command: "--probe POSITIVE,NEGATIVE".into() },
                Step { rung: Rung::Touched, verdict: Verdict::NotRun, seconds: 0, command: "keel suite --touched ROOT".into() },
            ],
            stopped_at: Some(Rung::Clippy),
            in_flight: None,
            pid: 4242,
        };
        let text = render_receipt(&l);
        assert!(text.contains("stopped_at = \"clippy\""));
        assert!(text.contains("outcome = \"fail\""));
        assert_eq!(parse_receipt(&text), Some(l.clone()));
        assert_eq!(l.exit_code(), 101);
        let green = Ladder { stopped_at: None, steps: vec![], ..l };
        let text = render_receipt(&green);
        assert!(text.contains("stopped_at = \"none\"") && text.contains("outcome = \"pass\""));
        assert_eq!(parse_receipt(&text).map(|p| p.stopped_at), Some(None));
        assert_eq!(parse_receipt("not a receipt"), None);
    }

    /// issue565, the D0388 pair. Known-positive: a ladder with a rung in flight renders `outcome =
    /// "running"`, names the rung, carries only the rungs finished so far and is not green - so a
    /// reader mid-run cannot take it for a verdict. Known-negative: the same steps with nothing in
    /// flight render the verdict. A receipt written before the running form (no `running` key) parses
    /// as finished, so the last landed receipt is still readable.
    #[test]
    fn a_receipt_mid_run_says_running_and_a_finished_one_says_its_verdict() {
        let done = vec![Step { rung: Rung::Validate, verdict: Verdict::Pass, seconds: 1, command: "keel gate validate .".into() }];
        let running = Ladder { head: "9703282".into(), at: 1_789_478_355, seconds: 1, steps: done, stopped_at: None, in_flight: Some(Rung::Guard), pid: 4242 };
        let text = render_receipt(&running);
        assert!(text.contains("outcome = \"running\"") && text.contains("running = \"guard\""), "{text}");
        assert!(text.contains("\npid = 4242\n"), "the stub names its writer (issue569): {text}");
        assert!(!text.contains("outcome = \"pass\""));
        assert_eq!(text.matches("\n[[rung]]").count(), 1, "only the rungs finished so far");
        let back = parse_receipt(&text).expect("a running receipt parses");
        assert_eq!(back, running);
        assert!(!back.green(), "a running ladder is not green yet");
        let finished = Ladder { in_flight: None, ..running };
        assert_eq!(finished.outcome(), "pass");
        assert!(render_receipt(&finished).contains("outcome = \"pass\"\nstopped_at = \"none\"\nrunning = \"none\""));
        let legacy = "head = \"1e2ed25\"\nat = 5\nseconds = 61\noutcome = \"pass\"\nstopped_at = \"none\"\n";
        assert_eq!(parse_receipt(legacy).map(|l| (l.in_flight, l.green(), l.pid)), Some((None, true, 0)));
    }

    fn stub_by(pid: u32, in_flight: Option<Rung>) -> String {
        render_receipt(&Ladder { head: "9703282".into(), at: 1_789_478_355, seconds: 1, steps: vec![], stopped_at: None, in_flight, pid })
    }

    /// issue569 known-positive, chosen before the tree is read: the receipt says a rung is running
    /// and names THIS process as its writer - alive by construction - so a second `keel verify` is
    /// refused with a line naming the rung, the pid and the stub's `at`, through the real liveness read.
    #[test]
    fn a_second_ladder_is_refused_while_the_stub_writer_is_alive() {
        let me = std::process::id();
        let line = exclusive_refusal(&stub_by(me, Some(Rung::Clippy)), crate::touched::pid_alive).expect("refused: the writer is alive");
        assert!(line.contains("rung clippy") && line.contains(&format!("pid {me} at 1789478355")), "{line}");
        assert!(line.contains("REFUSING") && line.contains("issue569"), "{line}");
    }

    /// issue569 known-negative: a running stub whose writer is gone (a pid no process holds), a
    /// finished receipt, and a receipt from before the pid rode it all refuse nothing - the launch
    /// proceeds and the stub is replaced.
    #[test]
    fn a_stub_whose_writer_is_gone_is_replaced() {
        let mut child = if cfg!(windows) { std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn() } else { std::process::Command::new("true").spawn() }.expect("spawn a short-lived child");
        let gone = child.id();
        let _ = child.wait();
        assert_eq!(exclusive_refusal(&stub_by(gone, Some(Rung::Guard)), crate::touched::pid_alive), None, "a dead writer's stub is replaced (pid {gone})");
        assert_eq!(exclusive_refusal(&stub_by(7, Some(Rung::Guard)), |_| false), None);
        assert_eq!(exclusive_refusal(&stub_by(7, None), |_| true), None, "a finished ladder is not in flight");
        let legacy = "head = \"1e2ed25\"\nat = 5\nseconds = 61\noutcome = \"running\"\nstopped_at = \"none\"\nrunning = \"guard\"\n";
        assert_eq!(exclusive_refusal(legacy, |_| true), None, "a stub from before the pid rode it names no writer");
        assert_eq!(exclusive_refusal("", |_| true), None, "no receipt, no refusal");
    }

    /// issue575 known-positive, chosen before the tree is read: a `running` stub whose writer is gone
    /// is KILLED during that rung - not a pass, not a fail - through the real liveness read; a stub
    /// from before the pid rode it names no writer and is the same report.
    #[test]
    fn a_running_stub_whose_writer_is_gone_is_killed_not_a_verdict() {
        let mut child = if cfg!(windows) { std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn() } else { std::process::Command::new("true").spawn() }.expect("spawn a short-lived child");
        let gone = child.id();
        let _ = child.wait();
        assert_eq!(wait_state(&stub_by(gone, Some(Rung::Clippy)), crate::touched::pid_alive), WaitState::Killed { rung: Rung::Clippy, pid: gone });
        assert_eq!(wait_state(&stub_by(7, Some(Rung::Touched)), |_| false), WaitState::Killed { rung: Rung::Touched, pid: 7 });
        let legacy = "head = \"1e2ed25\"\nat = 5\nseconds = 61\noutcome = \"running\"\nstopped_at = \"none\"\nrunning = \"guard\"\n";
        assert_eq!(wait_state(legacy, |_| true), WaitState::Killed { rung: Rung::Guard, pid: 0 }, "no writer named, nothing to wait for");
    }

    /// issue575 known-negative: a finished receipt is the ladder's own verdict, red or green, and a
    /// `running` stub whose writer is alive (this process) is in flight - the wait continues and
    /// nothing is judged. The sprint 726 shape (in flight read as killed) is the case in the middle.
    #[test]
    fn a_finished_receipt_is_the_verdict_and_a_live_writer_is_in_flight() {
        let me = std::process::id();
        assert_eq!(wait_state(&stub_by(me, Some(Rung::Clippy)), crate::touched::pid_alive), WaitState::InFlight { rung: Rung::Clippy, pid: me });
        let green = Ladder { head: "abc1234".into(), at: 9, seconds: 718, steps: vec![Step { rung: Rung::Validate, verdict: Verdict::Pass, seconds: 1, command: "v".into() }], stopped_at: None, in_flight: None, pid: me };
        let WaitState::Finished(l) = wait_state(&render_receipt(&green), |_| true) else { panic!("a finished ladder is the verdict, whoever is alive") };
        assert!(l.green());
        assert_eq!(l.exit_code(), 0);
        let red = Ladder { stopped_at: Some(Rung::Clippy), steps: vec![Step { rung: Rung::Clippy, verdict: Verdict::Fail(101), seconds: 40, command: "c".into() }], ..green };
        let WaitState::Finished(l) = wait_state(&render_receipt(&red), |_| false) else { panic!("a red ladder is a verdict too") };
        assert_eq!((l.green(), l.exit_code(), l.stopped_at), (false, 101, Some(Rung::Clippy)));
        assert_eq!(wait_state("", |_| true), WaitState::NoReceipt);
        assert_eq!(own_args(&args(&["--wait", "."])), args(&["."]), "--wait is the ladder's own flag; the root is what is left");
    }

    /// issue565's control, D0047: the running form was added per receipt three times (issue399 suite,
    /// issue468 touched, now verify) and the file left out was the one that fired. This census reads
    /// every module in the crate that writes a `.keel/metrics/*-receipt.toml` with an `outcome`, and
    /// fails for one that never renders `running` - a fourth such receipt cannot ship without it.
    #[test]
    fn every_receipt_writer_with_an_outcome_renders_a_running_form() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let sources: Vec<(String, String)> = std::fs::read_dir(&src)
            .expect("src listing")
            .map(|e| e.expect("entry").path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .map(|p| (p.file_name().expect("name").to_string_lossy().into_owned(), std::fs::read_to_string(&p).expect("read")))
            .collect();
        let (writers, missing) = receipt_writer_census(&sources);
        assert_eq!(writers, ["suite.rs", "touched.rs", "verify.rs"], "the census is the three receipts; a new one joins this list AND renders running");
        assert!(missing.is_empty(), "receipt writer(s) with no running form: {missing:?}");
    }

    /// The census predicate, probed both ways (D0388) before it is trusted on the crate: a module that
    /// writes an `outcome` receipt and never says `running` is named (known-positive); one that renders
    /// the word inside a format string is not (known-negative); a module with no receipt is not a writer.
    #[test]
    fn the_census_names_a_writer_without_running_and_passes_one_with_it() {
        let sources = vec![
            ("fourth.rs".to_owned(), "pub const RECEIPT: &str = \".keel/metrics/fourth-receipt.toml\";\nfn render() -> String { format!(\"outcome = \\\"{}\\\"\", \"pass\") }".to_owned()),
            ("fine.rs".to_owned(), "pub const RECEIPT: &str = \".keel/metrics/fine-receipt.toml\";\nfn render() -> String { \"outcome = \\\"running\\\"\".to_owned() }".to_owned()),
            ("bystander.rs".to_owned(), "fn outcome = running tests".to_owned()),
        ];
        let (writers, missing) = receipt_writer_census(&sources);
        assert_eq!(writers, ["fine.rs", "fourth.rs"]);
        assert_eq!(missing, ["fourth.rs"]);
    }

    /// (sorted writers, the writers among them with no `running` form). A writer is a module holding a
    /// `.keel/metrics/*` RECEIPT constant and rendering an `outcome` key; the word counts as a Rust
    /// literal (`"running"`) or inside a format string (`\"running\"`).
    fn receipt_writer_census(sources: &[(String, String)]) -> (Vec<String>, Vec<String>) {
        let mut writers = Vec::new();
        let mut missing = Vec::new();
        for (name, text) in sources {
            if !(text.contains("RECEIPT: &str = \".keel/metrics/") && text.contains("outcome = ")) {
                continue;
            }
            if !text.replace("\\\"", "\"").contains("\"running\"") {
                missing.push(name.clone());
            }
            writers.push(name.clone());
        }
        writers.sort();
        (writers, missing)
    }
}
