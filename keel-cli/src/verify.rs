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

/// One rung of the ladder. Declaration order IS cost order IS run order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rung {
    /// `keel gate validate ROOT` - the `.tracking` authority.
    Validate,
    /// `keel gate guard ROOT` - every forward guard, answering from its receipt when inputs are equal (D0371).
    Guard,
    /// `cargo clippy --release --all-targets -- -D warnings` over `keel-cli`.
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

/// The finished ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ladder {
    pub head: String,
    pub at: u64,
    pub seconds: u64,
    pub steps: Vec<Step>,
    /// The rung the ladder stopped at; `None` is a green ladder.
    pub stopped_at: Option<Rung>,
}

impl Ladder {
    /// Green when no rung is red.
    #[must_use]
    pub const fn green(&self) -> bool {
        self.stopped_at.is_none()
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

/// The D0388 pair as named on the command line: `--probe POSITIVE,NEGATIVE`.
///
/// Each side is a command line split on whitespace and run in the project root; both must exit 0.
/// `None` when the flag is absent (the rung is then recorded `not-named`); an error when the flag is
/// present but does not name exactly two non-empty sides - a pair with one side is not a pair, and
/// a check probed on one case is the D0388 defect this command exists to refuse.
///
/// # Errors
///
/// A `--probe` with no value, an empty side, or a side count other than two.
pub fn parse_probe(args: &[String]) -> Result<Option<(String, String)>, String> {
    let Some(i) = args.iter().position(|a| a == "--probe") else {
        return Ok(None);
    };
    let Some(value) = args.get(i + 1) else {
        return Err("--probe takes POSITIVE,NEGATIVE - two command lines, comma-separated (D0388)".to_owned());
    };
    let sides: Vec<&str> = value.split(',').map(str::trim).collect();
    match sides.as_slice() {
        [pos, neg] if !pos.is_empty() && !neg.is_empty() => Ok(Some(((*pos).to_owned(), (*neg).to_owned()))),
        _ => Err(format!("--probe names {} side(s); a D0388 pair is exactly two, POSITIVE,NEGATIVE, both non-empty: `{value}`", sides.iter().filter(|s| !s.is_empty()).count())),
    }
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
        if a == "--probe" {
            skip = true;
            continue;
        }
        if a == "--no-receipt" || a == "--help" || a == "-h" {
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
/// plainly as what was. The probe rung is asked of the runner only when a pair was named; otherwise
/// it is `NotNamed` and the ladder continues - an unnamed pair is a fact about the invocation, not a
/// red about the tree.
pub fn climb<F: FnMut(Rung) -> (Verdict, String)>(probe_named: bool, mut runner: F) -> (Vec<Step>, Option<Rung>) {
    let mut steps = Vec::with_capacity(Rung::ORDER.len());
    let mut stopped_at = None;
    for rung in Rung::ORDER {
        if stopped_at.is_some() {
            steps.push(Step { rung, verdict: Verdict::NotRun, seconds: 0, command: command_text(rung) });
            continue;
        }
        if rung == Rung::Probe && !probe_named {
            steps.push(Step { rung, verdict: Verdict::NotNamed, seconds: 0, command: "--probe POSITIVE,NEGATIVE not given".to_owned() });
            continue;
        }
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
        Rung::Clippy => "cargo clippy --release --all-targets -- -D warnings".to_owned(),
        Rung::Probe => "--probe POSITIVE,NEGATIVE".to_owned(),
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
fn run_rung(rung: Rung, repo: &Path, exe: &Path, probe: Option<&(String, String)>, forced: bool) -> (Verdict, String) {
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
            let mut cmd = Command::new("cargo");
            cmd.args(["clippy", "--release", "--all-targets", "--manifest-path"]).arg(repo.join("keel-cli").join("Cargo.toml")).args(["--", "-D", "warnings"]).current_dir(repo);
            (status_of(scrub(&mut cmd)), "cargo clippy --release --all-targets --manifest-path keel-cli/Cargo.toml -- -D warnings".to_owned())
        }
        Rung::Probe => {
            let Some((pos, neg)) = probe else {
                return (Verdict::NotNamed, command_text(Rung::Probe));
            };
            let text = format!("{pos} ; {neg}");
            for side in [pos, neg] {
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
        "# verify receipt (D0476): the pre-commit ladder - validate, guard, clippy, the D0388 probe pair, the\n# touched run - in cost order, stopped at the first red. `stopped_at = \"none\"` is a green ladder; a rung\n# after the red is `not-run`; a probe rung with no --probe pair is `not-named`, never invented. On green\n# the touched receipt ({}) is the one keel land honours (D0474).\nhead = \"{}\"\nat = {}\nseconds = {}\noutcome = \"{}\"\nstopped_at = \"{}\"\n",
        crate::touched::RECEIPT,
        l.head,
        l.at,
        l.seconds,
        if l.green() { "pass" } else { "fail" },
        l.stopped_at.map_or("none", Rung::name),
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
    Some(Ladder { head, at, seconds, steps, stopped_at })
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

fn print_help() {
    println!("usage: keel verify [ROOT] [--probe POSITIVE,NEGATIVE] [--no-receipt]");
    println!("  the pre-commit checks as one ladder in cost order, stopping at the first red (D0476):");
    println!("    1. keel gate validate       2. keel gate guard (from its receipt, D0371)");
    println!("    3. cargo clippy --release --all-targets -- -D warnings");
    println!("    4. the D0388 probe pair named by --probe: two command lines, both must exit 0");
    println!("    5. keel suite --touched (binaries observed green at this content are skipped, D0474)");
    println!("  Writes {RECEIPT} naming the rung it stopped at; a rung after the red is not-run, a probe rung");
    println!("  with no pair is not-named. Exits as the failing rung did. --no-receipt forces guard and the");
    println!("  touched run to run everything. On green the touched receipt is the one keel land honours.");
}

/// `keel verify`.
#[must_use]
pub fn cmd(args: &[String], repo: &Path) -> i32 {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return 0;
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
    let forced = crate::receipt::forced(args);
    let head = crate::gitx::git().arg("-C").arg(repo).args(["rev-parse", "--short", "HEAD"]).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let started = Instant::now();
    let (steps, stopped_at) = climb(probe.is_some(), |rung| {
        println!("keel verify: rung {} - {}", rung.name(), command_text(rung));
        run_rung(rung, repo, &exe, probe.as_ref(), forced)
    });
    let ladder = Ladder { head, at: now_secs(), seconds: started.elapsed().as_secs(), steps, stopped_at };
    let path = repo.join(RECEIPT);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = crate::write::write_atomic(&path, render_receipt(&ladder)) {
        eprintln!("keel verify: receipt could not be written: {e}");
    }
    for st in &ladder.steps {
        println!("  {:<9} {:<12} {:>5} s", st.rung.name(), st.verdict.label(), st.seconds);
    }
    match ladder.stopped_at {
        None => {
            println!("keel verify: pass - every rung green in {} s; receipt {RECEIPT}", ladder.seconds);
            0
        }
        Some(r) => {
            println!("keel verify: fail - stopped at {} after {} s; receipt {RECEIPT}", r.name(), ladder.seconds);
            ladder.exit_code()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{climb, own_args, parse_probe, parse_receipt, render_receipt, Ladder, Rung, Step, Verdict};

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    /// D0388 known-positive: a red clippy stops the ladder BEFORE the touched run - the runner is never
    /// asked for the touched rung, and the receipt records probe and touched as not-run. This is the
    /// sprint705 shape (a lint found after a twenty-minute run) refused by construction.
    #[test]
    fn a_red_rung_stops_the_ladder_before_the_test_binaries_compile() {
        let mut asked = Vec::new();
        let (steps, stopped) = climb(true, |r| {
            asked.push(r);
            let v = if r == Rung::Clippy { Verdict::Fail(101) } else { Verdict::Pass };
            (v, format!("cmd {}", r.name()))
        });
        assert_eq!(stopped, Some(Rung::Clippy));
        assert_eq!(asked, vec![Rung::Validate, Rung::Guard, Rung::Clippy], "nothing after the red is run");
        let verdicts: Vec<_> = steps.iter().map(|s| s.verdict.clone()).collect();
        assert_eq!(verdicts, vec![Verdict::Pass, Verdict::Pass, Verdict::Fail(101), Verdict::NotRun, Verdict::NotRun]);
        assert_eq!(steps[4].command, "keel suite --touched ROOT", "the not-run rung still names what it would have run");
    }

    /// D0388 known-negative: a green ladder asks every rung in cost order and reaches the touched run once.
    #[test]
    fn a_green_ladder_reaches_the_touched_run_last() {
        let mut asked = Vec::new();
        let (steps, stopped) = climb(true, |r| {
            asked.push(r);
            (Verdict::Pass, r.name().to_owned())
        });
        assert_eq!(stopped, None);
        assert_eq!(asked, Rung::ORDER.to_vec());
        assert_eq!(asked.iter().filter(|r| **r == Rung::Touched).count(), 1);
        assert!(steps.iter().all(|s| s.verdict == Verdict::Pass));
    }

    /// An unnamed pair is recorded, not invented (D0388): the probe rung is `not-named`, the runner is
    /// not asked for it, and the ladder continues to the touched run.
    #[test]
    fn an_unnamed_probe_pair_is_recorded_not_invented() {
        let mut asked = Vec::new();
        let (steps, stopped) = climb(false, |r| {
            asked.push(r);
            (Verdict::Pass, String::new())
        });
        assert_eq!(stopped, None);
        assert!(!asked.contains(&Rung::Probe));
        assert!(asked.contains(&Rung::Touched));
        assert_eq!(steps[3].verdict, Verdict::NotNamed);
    }

    /// `--probe` takes exactly a pair: two sides pass, one side is refused, absent is `None`.
    #[test]
    fn probe_flag_takes_exactly_a_pair() {
        assert_eq!(parse_probe(&args(&["--probe", "cargo test a, cargo test b"])).unwrap(), Some(("cargo test a".into(), "cargo test b".into())));
        assert_eq!(parse_probe(&args(&["."])).unwrap(), None);
        assert!(parse_probe(&args(&["--probe", "cargo test a"])).unwrap_err().contains("1 side(s)"));
        assert!(parse_probe(&args(&["--probe", "a,"])).is_err());
        assert!(parse_probe(&args(&["--probe"])).is_err());
    }

    /// The root is found among what the ladder does not consume: the probe value is never a path.
    #[test]
    fn the_probe_value_is_not_mistaken_for_a_root() {
        assert_eq!(own_args(&args(&["--probe", "cargo test a,cargo test b", ".", "--no-receipt"])), vec![".".to_owned()]);
        assert!(own_args(&args(&["--probe", "x,y"])).is_empty());
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
}
