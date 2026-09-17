//! keel-hooks: the Claude hooks (D0134) - the in-loop gates, IN THE BINARY.
//!
//! The ninth D0479 extraction (sprint 739), and the first to slice `main.rs` itself: `cmd_hook` through
//! `hook_stop` (the `Stop` / `PostToolUse` / `PreToolUse` / `UserPromptSubmit` / `SubagentStop` entry points, the
//! ledger writers, the override unlock, the bash and write classifiers, the console probe and the headless
//! ask) moved here whole by `scripts/extract_hooks.py`, with `shellcheck` (the pre-bash advisories) and
//! `proactive` (the post-edit advisories), whose only readers they are. keel-cli re-exports the crate as
//! `hooks` and the two modules at their old paths, and `main.rs` dispatches `keel hook` to [`cmd_hook`]
//! exactly as before. The crate reads the write API, the guards, the view layer, the suite's hook-binary
//! refresh and the process layer's project discovery, and nothing reads it but the binary - so an edit to
//! a hook rebuilds this crate and keel-cli only.
#![forbid(unsafe_code)]
#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing, clippy::todo, clippy::unimplemented)]
#![allow(clippy::implicit_hasher, clippy::too_long_first_doc_paragraph, clippy::module_name_repetitions)]
// Tests may use unwrap/expect/panic/indexing/asserts freely.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]

pub mod proactive;
pub mod shellcheck;

use std::path::{Path, PathBuf};

use keel_process::workspace::{engine_version_skew, find_repo_root};

/// Hard latency cap for prompt-path recall. Measured at 641-863ms on this repo's 13.6k-item corpus, so
/// the cap leaves headroom while bounding the worst case: a prompt-path cost that grows with the corpus
/// is a tax that compounds silently, and the reader is TOLD when it is hit rather than left to wonder.
pub const RECALL_CAP_MS: u128 = 2500;

/// Default character budget for a recalled payload. Bounded on purpose: injected context costs the
/// human tokens on every turn, so the cost has to be predictable rather than proportional to how
/// connected the term happens to be (D0242 part 1).
pub const RECALL_BUDGET: usize = 4000;


/// `keel hook <stop|post-edit>` (D0134) — the in-loop gates, IN THE BINARY.
///
/// These were python wrappers. The CHECKING was always Rust; python only parsed the hook's stdin
/// JSON and emitted the response — pure glue, bought with a SECOND RUNTIME DEPENDENCY on every
/// contributor's machine. That is the issue076 class inside the governance layer again: where python
/// is absent the gate silently does not run, and D0129 puts five mixed-OS machines in scope. Since
/// `keel` is already a hard requirement for these gates to mean anything, and `serde_json` is already
/// a dependency, the glue belongs here and the dependency goes away.
///
/// Reads the hook payload on stdin, writes the hook protocol on stdout, and NEVER fails a turn:
/// any internal error exits 0 silently.
pub fn cmd_hook(args: &[String]) -> i32 {
    use std::io::Read as _;
    // issue179: an unrecognised event already errors, but a flag should be told it is a flag.
    if let Some(f) = args.first().filter(|a| a.starts_with('-')) {
        eprintln!("error: `{f}` looks like a flag, not a hook event (issue179).");
        return 2;
    }
    let Some(event) = args.first().map(String::as_str) else {
        eprintln!("usage: keel hook <stop|post-edit|pre-bash|user-prompt>");
        return 2;
    };
    // BOUNDED LIFETIME (issue180b). `read_to_string` on stdin blocks until EOF, and if the parent goes
    // away WITHOUT closing stdin the hook waits forever - holding a Windows file lock on
    // `target/release/keel.exe`, so every later `cargo build` fails with `Access is denied`. That
    // happened three times in one turn, and the error names cargo and a file permission, so it reads as
    // a toolchain problem rather than as the hook. An in-loop gate that can wedge the build it gates is
    // the worst failure mode available to it.
    //
    // The watchdog exits 0, never nonzero: D0134 says a hook NEVER fails a turn, so a hook that gave up
    // waiting must look exactly like a hook with nothing to say — to the HARNESS. To the ledger it
    // must not (panel R1, robotics finding 4 / K2): a wedged hook that vanished without a line was
    // the one fail-SILENT branch inside the layer the enforcement-report reads, invisible to D0180's
    // single instrumentation path. The watchdog now appends its own line before exiting.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(HOOK_DEADLINE_SECS));
        if let Some(root) = find_repo_root() {
            ledger_emit(&root, "", "hook-watchdog-timeout", 0, u128::from(HOOK_DEADLINE_SECS) * 1000);
        }
        std::process::exit(0);
    });
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
    let root = find_repo_root().unwrap_or_else(|| PathBuf::from("."));
    if !root.join(".tracking").is_dir() {
        return 0; // not a keel project -> silent no-op, correctly
    }
    // D0391/issue408: on a self-build tree the hooks keep their own stable copy current from the build
    // output, so cargo never has to unlink the file a hook is running. Best-effort: a failure is one
    // stderr line and the hook proceeds on the image it has.
    match keel_suite::hook_binary::refresh(&root) {
        Ok(Some(r)) => {
            let session = payload.get("session_id").and_then(serde_json::Value::as_str).unwrap_or("");
            ledger_emit(&root, session, "hook-binary-refreshed", 0, 0);
            eprintln!("[keel] hook binary refreshed: {} <- {}", r.copy.display(), r.from.display());
        }
        Ok(None) => {}
        Err(e) => eprintln!("[keel] hook binary refresh failed ({e}); this fire runs the image it has"),
    }

    // Fire-ledger + subagent baseline (D0174/P0.1, D0180): every fire leaves one machine-local
    // JSONL line keyed by session id and event — the SINGLE instrumentation path the
    // hooks-actually-fired checks read. A session's FIRST fire also stores the tree fingerprint,
    // which is the SubagentStop baseline (P0.6).
    let session = payload.get("session_id").and_then(serde_json::Value::as_str).unwrap_or("").to_string();
    // issue337: the hook runs in the HARNESS's environment, not the session's shell, so the session's
    // declared actor is not in KEEL_ACTOR here. The payload carries the session id; seed it so
    // `actor::resolve` can read the session binding the session's own commands remembered. Never
    // overrides an id already present, and sets only what the payload actually said.
    if !session.is_empty() && std::env::var_os("CLAUDE_CODE_SESSION_ID").is_none() {
        std::env::set_var("CLAUDE_CODE_SESSION_ID", &session);
    }
    // Never write the baseline from the subagent-stop event itself: a subagent whose FIRST fire is
    // its own stop would baseline against the post-work tree and silently skip the gate — found by
    // running the branch, not by reading it.
    if !session.is_empty() && event != "subagent-stop" {
        let bl = root.join(".keel").join("metrics").join(format!("baseline-{session}.fp"));
        if !bl.exists() {
            let _ = std::fs::create_dir_all(root.join(".keel").join("metrics"));
            let _ = std::fs::write(&bl, keel_model::fingerprint::of(&root).to_string());
        }
    }
    // D0501 / issue578: a subagent is gated against the tree at its OWN start, not the session's. Every
    // hook fire inside a subagent carries `agent_id` (SubagentStart first, then its tool fires), so the
    // first such fire that is not the agent's own stop stores its baseline; never overwritten, so a
    // long agent's later fires cannot move it.
    if event != "subagent-stop" {
        if let Some(agent_id) = payload.get("agent_id").and_then(serde_json::Value::as_str) {
            let bl = agent_baseline_path(&root, agent_id);
            if !bl.exists() {
                let _ = std::fs::create_dir_all(root.join(".keel").join("metrics"));
                let _ = std::fs::write(&bl, keel_model::fingerprint::of(&root).to_string());
            }
        }
    }
    // D0414 / issue429: a hook fire collects its phases whether or not KEEL_PERF is set, so a slow
    // one can write what it spent its time on into its own ledger line.
    keel_perf::perf::collect_phases();
    let started = std::time::Instant::now();
    let code = match event {
        "post-edit" => hook_post_edit(&payload, &root),
        "stop" => hook_stop(&payload, &root),
        "user-prompt" => hook_user_prompt(&root, &payload, &session),
        "pre-bash" => hook_pre_bash(&payload, &root, &session),
        "pre-write" => hook_pre_write(&payload, &root),
        "subagent-stop" => hook_subagent_stop(&payload, &root, &session),
        "subagent-start" => 0, // D0501: the baseline write above is the whole event; the fire line counts it
        "config-change" => hook_config_change(&payload),
        other => {
            eprintln!("unknown hook event '{other}' (expected stop|post-edit|pre-bash|user-prompt|pre-write|subagent-start|subagent-stop|config-change)");
            2
        }
    };
    ledger_fire(&root, &session, event, code, started.elapsed().as_millis());
    code
}

/// `ConfigChange` (D0296 / issue365): a repo-scope settings change that sets the hook kill switch or
/// alters a keel-owned hook entry is REFUSED (exit 2 - the change is not applied to this session)
/// and the file is RESTORED in place, because Claude Code does not revert a blocked change and a
/// key left on disk kills every hook at the next launch (D0296 run 5). The payload carries `source`
/// and `file_path` only - no content (run 3) - so the file is read here. Runs in every profile: an
/// advisory against a kill switch would be the first thing switched off. A file that cannot be read
/// or parsed is REPORTED and allowed - the hook never blocks on what it cannot see.
fn hook_config_change(payload: &serde_json::Value) -> i32 {
    let source = payload.get("source").and_then(serde_json::Value::as_str).unwrap_or("");
    let is_local = match source {
        "project_settings" => false,
        "local_settings" => true,
        _ => return 0, // user, policy and skills changes are not this hook's business
    };
    let Some(file) = payload.get("file_path").and_then(serde_json::Value::as_str) else { return 0 };
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[keel] config-change: {file} cannot be read ({e}) - nothing to restore; the plugin rendering still carries the hook set");
            return 0;
        }
    };
    let doc: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[keel] config-change: {file} is not JSON ({e}) - not restored; fix the file or run `keel sync-claude`");
            return 0;
        }
    };
    let Some((restored, why)) = keel_write::claude_surface::restored_settings(&doc, is_local) else { return 0 };
    let pretty = serde_json::to_string_pretty(&restored).unwrap_or_default() + "\n";
    match keel_write::write::write_atomic(Path::new(file), &pretty) {
        Ok(()) => eprintln!(
            "[keel] config-change REFUSED: {file} - {why} (issue365/D0296). The change is not applied and the file is restored in place; a hook host changes through a Decision, never by switching the hooks off."
        ),
        Err(e) => eprintln!("[keel] config-change REFUSED: {file} - {why}; the restore FAILED ({e}) - run `keel sync-claude` before the next launch"),
    }
    2
}

/// Append one fire-ledger line (machine-local, `.keel/metrics/hooks.jsonl`, gitignored class).
/// Best-effort by design: the ledger is evidence infrastructure, and a full disk must not turn an
/// advisory hook into a blocker — but a write failure is still printed, never swallowed (K2).
fn ledger_emit(root: &Path, session: &str, event: &str, exit: i32, ms: u128) {
    ledger_line(root, session, event, exit, ms, None);
}

/// The verdict a hook fire EMITTED and the control that emitted it (issue446). `hook_emit` exits 0 for
/// allow and block alike - the harness reads the verdict from the JSON on stdout, never from the exit
/// code - so for its first 16 196 lines the ledger derived `allow` from `exit == 0` and held ONE block
/// (config-change, the only handler that exits non-zero). The emitting site notes what it wrote here
/// through `hook_refuse`; the dispatcher's `ledger_fire` reads it once. First verdict wins: a fire emits
/// at most one protocol object the harness acts on.
static EMITTED_VERDICT: std::sync::Mutex<Option<(String, String)>> = std::sync::Mutex::new(None);

fn note_verdict(decision: &str, control: &str) {
    if let Ok(mut g) = EMITTED_VERDICT.lock() {
        if g.is_none() {
            *g = Some((decision.to_string(), control.to_string()));
        }
    }
}

/// Emit a REFUSING hook-protocol object (`decision: block` or `permissionDecision: deny`) and note the
/// verdict for the ledger under `control`, the name of the check that refused. Same exit as `hook_emit`.
fn hook_refuse(control: &str, v: &serde_json::Value) -> i32 {
    let decision = if v.get("decision").and_then(|d| d.as_str()) == Some("block") {
        "block"
    } else if v.pointer("/hookSpecificOutput/permissionDecision").and_then(|d| d.as_str()) == Some("deny") {
        "deny"
    } else {
        "allow"
    };
    note_verdict(decision, control);
    hook_emit(v)
}

/// The fire line for a dispatched hook event: the EMITTED verdict when one was noted, the exit code
/// only for a handler that emitted nothing. A refusal carries `control` and `actorKind`, so the ledger
/// can answer whose act a control fell on (issue445/446, D0424: a control is kept on evidence).
fn ledger_fire(root: &Path, session: &str, event: &str, exit: i32, ms: u128) {
    let emitted = EMITTED_VERDICT.lock().ok().and_then(|mut g| g.take());
    ledger_line(root, session, event, exit, ms, emitted);
}

/// A write-path refusal is a ledger fact (issue445): `keel accept` / `reject` and the other API writes
/// that refuse used to exit non-zero and write nothing anywhere, so the question "has an agent ever
/// tried this" had no record to be read from. `event = refused`, `control` names the check, `verb` the
/// command; ms is 0 because the refusal is the whole run.
pub fn ledger_refused(root: &Path, verb: &str, control: &str) {
    let session = std::env::var("CLAUDE_CODE_SESSION_ID").unwrap_or_default();
    // issue449: the control name is a registry fact (`write::WRITE_PATH_REFUSALS`), not a free string.
    // A name the registry never declared is still a fire, so it is written - marked, so the census
    // cannot count it under a row that does not exist.
    let name = if keel_write::write::write_path_refusal(verb, control).is_some() {
        format!("{verb}:{control}")
    } else {
        eprintln!("keel: refusal {verb}:{control} is not in the write-path registry (issue449); ledgered as unregistered");
        format!("unregistered:{verb}:{control}")
    };
    ledger_line(root, &session, "refused", 1, 0, Some(("refused".to_string(), name)));
}

/// `record decision` / `record issue` refused prose that reads as captured tool output (D0224/issue256):
/// the refusal is a ledger fact - `record:tool-output-prose` is the census row (issue449) - and the
/// verb's exit code.
pub fn refused_injected_prose(root: &Path, e: &dyn std::fmt::Display) -> i32 {
    ledger_refused(root, "record", "tool-output-prose");
    eprintln!("error: {e}");
    1
}
/// The commit tier is in the ledger with the in-loop tiers (dcRefusalIsALedgerFact clause d): the
/// scaffolded pre-commit hook runs `keel gate validate`, `keel gate guard` and `keel gate check-engine` as separate
/// processes, so each writes one `commit-gate-<tier>` line - `allow` when green, `block` naming the
/// refusing controls (the failing guard names, or the tier itself) when red. A run by hand writes the same line; the ledger does not know who
/// invoked it, and a rate over both is still a rate.
pub fn ledger_gate(root: &Path, tier: &str, failing: &[String], ms: u128) {
    let session = std::env::var("CLAUDE_CODE_SESSION_ID").unwrap_or_default();
    let verdict = if failing.is_empty() { None } else { Some(("block".to_string(), failing.join(","))) };
    ledger_line(root, &session, &format!("commit-gate-{tier}"), i32::from(!failing.is_empty()), ms, verdict);
}

/// The kind of actor this process runs as - `human`, `ai` or `unknown` from actors.sysml, `undeclared`
/// when the bound actor has no part there, `unbound` when no actor resolves at all - so a refusal
/// line says whose act the control fell on.
fn ledger_actor_kind(root: &Path) -> String {
    keel_actor::actor::resolve(root, None).map_or_else(
        |_| "unbound".to_string(),
        |name| keel_actor::actor::kind_of(root, &name).unwrap_or_else(|| "undeclared".to_string()),
    )
}

fn ledger_line(root: &Path, session: &str, event: &str, exit: i32, ms: u128, verdict: Option<(String, String)>) {
    use std::io::Write as _;
    let dir = root.join(".keel").join("metrics");
    if std::fs::create_dir_all(&dir).is_err() {
        eprintln!("[keel] fire-ledger unavailable: cannot create {}", dir.display());
        return;
    }
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (decision, control) = match verdict {
        Some((d, c)) => (d, Some(c)),
        None => ((if exit == 0 { "allow" } else { "block" }).to_string(), None),
    };
    let ms = u64::try_from(ms).unwrap_or(u64::MAX);
    // issue378 / GH#55: WHICH binary ran this hook, and which build - the turn-boundary surface's
    // answer to "did the pinned engine gate this", readable from `keel show status`.
    let mut record = serde_json::json!({"ts": ts, "session": session, "event": event, "decision": decision, "exit": exit, "ms": ms,
        "bin": std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(), "build": env!("KEEL_BUILD_COMMIT")});
    if let Some(obj) = record.as_object_mut() {
        // issue446: a line that is not an allow names the control that refused and the kind of actor
        // it refused. An allow carries neither - 16 000 lines a day should not each pay for a fact that
        // only a refusal has.
        if decision != "allow" {
            obj.insert("control".to_string(), serde_json::Value::String(control.unwrap_or_else(|| "exit-code".to_string())));
            obj.insert("actorKind".to_string(), serde_json::Value::String(ledger_actor_kind(root)));
        }
        // D0414 / issue429: a SLOW fire explains itself - the phases it measured, longest first, with the
        // remainder no counter covered named as unattributed. A fast fire carries no field (pm.rs owns the
        // threshold and the reader).
        if let Some(phases) = keel_view::pm::slow_fire_phases(ms) {
            obj.insert("phases".to_string(), phases);
        }
    }
    let line = format!("{record}\n");
    let appended = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("hooks.jsonl"))
        .and_then(|mut f| f.write_all(line.as_bytes()));
    if let Err(e) = appended {
        eprintln!("[keel] fire-ledger write failed: {e}");
    }
}


/// Minimal raw-HTTP call to the LOCAL console (dependency-free; localhost only). Returns the body.
fn console_http(method: &str, path: &str, body: Option<&str>) -> Option<String> {
    use std::io::{Read as _, Write as _};
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], CONSOLE_PORT));
    let mut s = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(500)).ok()?;
    let _ = s.set_read_timeout(Some(std::time::Duration::from_millis(1500)));
    let payload = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{CONSOLE_PORT}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut buf = String::new();
    let _ = s.read_to_string(&mut buf);
    buf.split("\r\n\r\n").nth(1).map(str::to_string)
}

/// The D0182 headless-ask mapping, hook side: inside a LAUNCHED RUN (`KEEL_RUN_ID` set) there is no
/// harness prompt, so an ask-tier write maps to the console approve queue - the ask-pending entry
/// is registered BEFORE any wait, the wait is bounded WELL under the hook deadline, and expiry or
/// an absent console maps to DENY plus a queued obligation (charter note 2: never let the harness
/// timeout decide).
fn headless_ask(root: &Path, path: &str, session: &str, run_id: &str) -> bool {
    let ask_body = serde_json::json!({"path": path, "session": session}).to_string();
    let ask_id = console_http("POST", "/api/run/ask", Some(&ask_body))
        .and_then(|b| serde_json::from_str::<serde_json::Value>(&b).ok())
        .and_then(|v| v.get("id").and_then(serde_json::Value::as_str).map(str::to_string));
    let mut denied_by: Option<String> = None;
    if let Some(id) = ask_id {
        // ~60s bounded wait (the hook deadline is 100s+; margin per charter note 2)
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let v = console_http("GET", &format!("/api/run/answer?id={id}"), None)
                .and_then(|b| serde_json::from_str::<serde_json::Value>(&b).ok());
            let answer = v.as_ref().and_then(|v| v.get("answer").and_then(serde_json::Value::as_str).map(str::to_string));
            let by = v.as_ref().and_then(|v| v.get("by").and_then(serde_json::Value::as_str)).unwrap_or("").to_string();
            match answer.as_deref() {
                Some("allow") => {
                    // issue200/K7: the ALLOW is a human authorization of a protected write and must
                    // leave a record naming the human, exactly as the override tier's consumption
                    // does. Recorded under the APPROVER — theirs is the judgment being recorded.
                    let approver = if by.is_empty() { "unknown".to_string() } else { by };
                    let _ = keel_write::write::record_obligation(
                        root,
                        "ask-allow",
                        &format!("headless run write ALLOWED by {approver}: {path}"),
                        &format!("{approver} allowed a launched run's ({run_id}) ask-tier write to {path} from the console approve queue (D0182 headless mapping; issue200/K7 - an authorization is a recorded fact, not an evaporating click). Discharge: review the landed write, then triage with a #Resolves edge."),
                        &approver,
                    );
                    return true;
                }
                Some("deny") => {
                    if !by.is_empty() {
                        denied_by = Some(by);
                    }
                    break;
                }
                _ => {}
            }
        }
    }
    // deny + queued obligation: a HUMAN's deny is recorded as their judgment (issue200); expiry or
    // an absent console stays attributed to the run's own actor, because nobody judged anything.
    let (actor, why) = denied_by.as_ref().map_or_else(
        || (keel_actor::actor::resolve(root, None), "no human approval arrived within the bounded wait, so it was DENIED".to_string()),
        |by| (Ok(by.clone()), format!("{by} DENIED it from the console approve queue")),
    );
    if let Ok(actor) = actor {
        let _ = keel_write::write::record_obligation(
            root,
            "headless-ask",
            &format!("headless run write denied pending review: {path}"),
            &format!("A launched run ({run_id}) requested an ask-tier write to {path}; {why} and queued (D0182 headless mapping). Discharge: review whether the write should happen, perform or decline it, and triage with a #Resolves edge."),
            &actor,
        );
    }
    false
}

/// The declared adoption profile: `strict`, `guided`, or `undeclared` (pre-adoption trees).
fn adoption_profile(root: &Path) -> &'static str {
    match std::fs::read_to_string(root.join(".engine").join("contracts").join("adoption-profile.toml")) {
        Ok(t) if t.lines().any(|l| l.trim().starts_with("profile") && l.contains("strict")) => "strict",
        Ok(t) if t.lines().any(|l| l.trim().starts_with("profile") && l.contains("guided")) => "guided",
        _ => "undeclared",
    }
}

/// A pending override unlock (D0176 tier 3): single-use, target-path-bound, expiring. The consuming
/// session is recorded in the obligation fact — the CLI cannot know the harness session a priori, so
/// session-binding is realized as short expiry + single use + consumed-by-session recorded.
pub const OVERRIDE_TTL_SECS: u64 = 900;

#[must_use]
pub fn override_path(root: &Path) -> PathBuf {
    root.join(".keel").join("override.json")
}

/// One file, named the way the filesystem names it (issue427/dcOverrideBindsToOneFile): absolute,
/// short names and links resolved, forward slashes, no `\\?\` prefix. A file that does not exist yet
/// is keyed by its existing parent's canonical form plus its own name, so an unlock armed for a file
/// about to be created and the write that creates it agree. `None` when the path cannot be placed.
fn override_key(root: &Path, path: &str) -> Option<String> {
    let p = Path::new(path);
    let abs = if p.is_absolute() { p.to_path_buf() } else { root.join(p) };
    let canon = match std::fs::canonicalize(&abs) {
        Ok(c) => c,
        Err(_) => std::fs::canonicalize(abs.parent()?).ok()?.join(abs.file_name()?),
    };
    Some(canon.to_string_lossy().replace('\\', "/").trim_start_matches("//?/").to_string())
}

/// What `keel override` may arm (issue427, UCA-O1): exactly one file. Before this, the unlock matched
/// by substring in either direction, so `keel override .tracking` armed an unlock the next write to
/// ANY path containing `.tracking` consumed, and a one-character target covered every path holding
/// that character. Now: an existing regular file, or a single new file under an API-owned surface
/// (its parent exists and the path carries a `PROTECTED_PATHS` prefix - the one place the API owns
/// the file's creation). A directory, a prefix, or a file nowhere is refused, and the refusal names
/// the file-not-directory rule. Returns the canonical key the unlock stores.
///
/// # Errors
///
/// A directory, a prefix, or a file nowhere: the message names the file-not-directory rule.
pub fn override_target(root: &Path, target: &str) -> Result<String, String> {
    let p = Path::new(target);
    let abs = if p.is_absolute() { p.to_path_buf() } else { root.join(p) };
    if abs.is_dir() {
        return Err(format!("`{target}` is a directory - an override unlocks ONE FILE, never a directory or a path prefix (issue427/D0176); name the file the write will touch"));
    }
    let key = override_key(root, target)
        .ok_or_else(|| format!("`{target}` cannot be placed - its parent directory does not exist; an override unlocks ONE FILE, never a directory or a path prefix (issue427/D0176)"))?;
    if abs.is_file() {
        return Ok(key);
    }
    if keel_write::claude_surface::PROTECTED_PATHS.iter().any(|(prefix, _)| key.contains(prefix)) {
        return Ok(key); // a single new file the API would otherwise own the creation of
    }
    Err(format!("`{target}` does not exist - an override unlocks ONE FILE that exists, or one new file under an API-owned surface, never a directory or a path prefix (issue427/D0176)"))
}

/// Consume a matching unlock: returns the reason when `path` is covered. Deletes the unlock (single
/// use) and records the tracked obligation naming the path ACTUALLY written (K7); on a failed
/// tracked write, degrades to a local ledger entry with a sync obligation (charter note 1).
fn consume_override(root: &Path, written_path: &str, session: &str) -> Option<String> {
    let op = override_path(root);
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&op).ok()?).ok()?;
    let target = v.get("path").and_then(serde_json::Value::as_str)?.replace('\\', "/");
    let created = v.get("ts").and_then(serde_json::Value::as_u64).unwrap_or(0);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    if now.saturating_sub(created) > OVERRIDE_TTL_SECS {
        let _ = std::fs::remove_file(&op);
        eprintln!("[keel] override unlock EXPIRED ({OVERRIDE_TTL_SECS}s) — run `keel override` again if still needed");
        return None;
    }
    // Path-bound by canonical EQUALITY (issue427): an unlock for one file covers no other - not a
    // file whose name contains the target's, not a file under a directory the target named.
    if override_key(root, &target)? != override_key(root, written_path)? {
        return None;
    }
    let reason = v.get("reason").and_then(serde_json::Value::as_str).unwrap_or("(no reason recorded)").to_string();
    let actor = v.get("actor").and_then(serde_json::Value::as_str).unwrap_or("unknown").to_string();
    let _ = std::fs::remove_file(&op); // single-use
    let title = format!("override used: direct write to {written_path}");
    let desc = format!(
        "A recorded override unlocked a direct write. Path actually written: {written_path}. Reason given: {reason}. Session: {session}. Discharge: a human reviews the write and triages this obligation with a #Resolves edge."
    );
    match keel_write::write::record_obligation(root, "override", &title, &desc, &actor) {
        Ok(p) => {
            // issue207/D0193 (srK14): the NORMAL consumption is counted too - without this line the
            // report's override counter saw only the UNSYNCED failure path and structurally
            // under-read override pressure in the very evidence promotion reviews must cite.
            ledger_emit(root, session, "override-consumed", 0, 0);
            eprintln!("[keel] override consumed — obligation recorded: {}", p.display());
        }
        Err(e) => {
            // The tracked write failed (possibly BECAUSE the target is the corrupted file being
            // repaired) — local ledger + sync obligation, never a silent unlock (K7).
            ledger_emit(root, session, "override-obligation-UNSYNCED", 1, 0);
            eprintln!("[keel] override consumed but the tracked obligation could not be written ({e}) — a local ledger entry holds it; SYNC OBLIGATION: record it with `keel record issue` once the tree is writable");
        }
    }
    Some(reason)
}

/// `PreToolUse` on Write|Edit over `.tracking`/`.engine` fact surfaces — the D0176 three-tier model,
/// invoked by the scaffolded pure-shell test when the binary is present.
///
///   tier 1 — API-owned surfaces: hard deny ABSENT A RECORDED OVERRIDE, refusal naming the command;
///   tier 2 — other `.tracking` writes: `permissionDecision: "ask"` (the harness prompt is a human
///            channel; the headless mapping is D0182's, exercised by the P5 launcher);
///   tier 3 — the recorded override reaches every tier (a corrupted API-owned file is repairable).
///
/// Profile-aware (P0.4/D0176): `strict` blocks as above; `guided`/`undeclared` is ADVISORY-FIRST —
/// the same detection runs, the fire-ledger accrues the D0180 evidence, and promotion to blocking is
/// a recorded decision citing it (K14).
fn hook_pre_write(payload: &serde_json::Value, root: &Path) -> i32 {
    let path = payload
        .pointer("/tool_input/file_path")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .replace('\\', "/");
    let session = payload.get("session_id").and_then(serde_json::Value::as_str).unwrap_or("");
    // The hook kill switch is DENIED in every profile (issue365/D0296): a repo-scope
    // `disableAllHooks` silences every hook from every scope, and the write asking for it is the
    // one write no advisory can follow - the advisory would be the first thing switched off.
    if keel_write::claude_surface::REPO_SCOPE_SETTINGS.iter().any(|p| path.contains(p)) {
        let text = ["/tool_input/content", "/tool_input/new_string"]
            .iter()
            .filter_map(|ptr| payload.pointer(ptr).and_then(serde_json::Value::as_str))
            .collect::<Vec<_>>()
            .join("\n");
        if keel_write::claude_surface::text_sets_kill_switch(&text) {
            let key = keel_write::claude_surface::HOOK_KILL_SWITCH;
            hook_refuse(
                "kill-switch",
                &serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                    "permissionDecisionReason": format!("[keel] {path} would set {key} - that silences EVERY hook from every scope (issue365/D0296). Refused in all profiles; a hook host changes through a Decision, never by switching the hooks off.")}}),
            );
            return 0;
        }
    }
    // Control plane first (D0179/K7): weakening or redirecting enforcement is approval-gated and,
    // under strict, leaves an orient-visible record — never a quiet config edit.
    if keel_write::claude_surface::CONTROL_PLANE_PATHS.iter().any(|p| path.contains(p)) {
        if adoption_profile(root) == "strict" {
            if let Ok(actor) = keel_actor::actor::resolve(root, None) {
                let _ = keel_write::write::record_obligation(
                    root,
                    "control-plane",
                    &format!("control-plane write requested: {path}"),
                    &format!("A Write|Edit touched enforcement configuration ({path}), session {session} (D0179/K7). The harness asked the human; this fact records that the control plane moved. Discharge: review the diff and triage with a #Resolves edge."),
                    &actor,
                );
            }
            println!(
                "{}",
                serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "ask",
                    "permissionDecisionReason": format!("[keel] {path} is CONTROL PLANE (D0179/K7) - approve only if you intend to change enforcement; the write is recorded")}})
            );
        } else {
            println!("[keel] control-plane write: {path} (D0179 - ask-tier under strict; the fire-ledger records this fire)");
        }
        return 0;
    }
    if !(path.contains(".tracking/") || path.contains(".engine/decisions/")) {
        return 0;
    }
    if let Some(reason) = consume_override(root, &path, session) {
        println!("[keel] override active for this write (reason: {reason}) - recorded, single-use");
        return 0;
    }
    let tier1 = keel_write::claude_surface::PROTECTED_PATHS.iter().find(|(p, _)| path.contains(p));
    let profile = adoption_profile(root);
    match (tier1, profile) {
        (Some((surface, sanctioned)), "strict") => {
            let reason = format!(
                "[keel] {surface} is an API-owned fact surface. Use the sanctioned path: {sanctioned} - or, for what the API cannot express, `keel override {path} --reason \"...\"` (single-use, recorded, reviewed)."
            );
            hook_refuse(
                "api-owned-surface",
                &serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny", "permissionDecisionReason": reason}}),
            );
        }
        (None, "strict") => {
            if let Ok(run_id) = std::env::var("KEEL_RUN_ID") {
                // Headless launched run: no prompt exists - console proxy, else deny + obligation.
                if headless_ask(root, &path, session, &run_id) {
                    println!("[keel] headless ask APPROVED from the console queue - write allowed");
                } else {
                    hook_refuse(
                        "headless-ask-unapproved",
                        &serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                            "permissionDecisionReason": format!("[keel] headless run: the ask-tier write to {path} was not approved within the bounded wait - DENIED and queued as an obligation (D0182)")}}),
                    );
                }
                return 0;
            }
            println!(
                "{}",
                serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "ask",
                    "permissionDecisionReason": format!("[keel] direct write to {path} (D0176 tier 2) - approve, or route through the write API")}})
            );
        }
        (Some((surface, sanctioned)), _) => {
            println!("[keel advisory] {surface} is an API-owned fact surface (D0176 tier 1 in strict) - prefer: {sanctioned}");
        }
        (None, _) => {} // tier 2 stays silent in advisory profiles: a comment on every ordinary edit is noise (issue094 rule)
    }
    0
}

/// Where an agent's own start fingerprint lives (D0501). The id is a harness string; anything that is
/// not a filename character is folded to `_` so a hostile or odd id cannot name a path elsewhere.
fn agent_baseline_path(root: &Path, agent_id: &str) -> PathBuf {
    let safe: String = agent_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    root.join(".keel").join("metrics").join(format!("agent-{safe}.fp"))
}

/// The fingerprint a stopping subagent is measured against (D0501 / issue578): the tree at ITS OWN
/// start when a fire recorded one, else the session's first-fire baseline - the pre-D0501 interval,
/// kept so a payload without `agent_id` stays gated rather than silently ungated. `None` = no
/// baseline at all.
fn subagent_baseline(root: &Path, payload: &serde_json::Value, session: &str) -> Option<String> {
    let own = payload
        .get("agent_id")
        .and_then(serde_json::Value::as_str)
        .and_then(|id| std::fs::read_to_string(agent_baseline_path(root, id)).ok());
    own.or_else(|| std::fs::read_to_string(root.join(".keel").join("metrics").join(format!("baseline-{session}.fp"))).ok())
}

/// What a `SubagentStop` fire does, decided before the gate runs (D0501/D0502). Pure, so the D0388
/// pair is pinned without a tree or a hook fire.
#[derive(Debug, PartialEq, Eq)]
enum SubagentStopRoute {
    /// no baseline for this agent or session: a `systemMessage` advisory, never a block
    NotGated,
    /// the tree did not move during the agent's lifetime: exit 0, nothing said
    Silent,
    /// a VERIFIER moved the tree: never a block - one `systemMessage`, and the fire's ledger line is
    /// refused-class under `verifier:tree-written` (D0502, beside `recorder:tree-red`)
    VerifierTreeWritten,
    /// any other agent moved the tree: the turn gate over the tree as it is (D0174/P0.6)
    Gate,
}

fn subagent_stop_route(agent_type: Option<&str>, baseline: Option<&str>, now: &str) -> SubagentStopRoute {
    let Some(baseline) = baseline else { return SubagentStopRoute::NotGated };
    if baseline.trim() == now {
        return SubagentStopRoute::Silent;
    }
    if agent_type == Some("verifier") {
        SubagentStopRoute::VerifierTreeWritten
    } else {
        SubagentStopRoute::Gate
    }
}

/// `SubagentStop` (D0174/P0.6): gate ONLY when the tree changed during the subagent's lifetime.
/// Baseline = the fingerprint stored at the agent's own start (D0501), else the session's first hook
/// fire; no baseline → a `systemMessage` advisory, never a block (a read-only subagent pays nothing).
/// A VERIFIER is never blocked (D0502): its receipt is its report and the tree's red is the primary's
/// to fix at the turn gate; a verifier that moved the tree is the ledgered fact instead.
fn hook_subagent_stop(payload: &serde_json::Value, root: &Path, session: &str) -> i32 {
    let agent_type = payload.get("agent_type").and_then(serde_json::Value::as_str);
    let baseline = subagent_baseline(root, payload, session);
    let now = keel_perf::perf::phase("hook:fingerprint", || keel_model::fingerprint::of(root)).to_string();
    match subagent_stop_route(agent_type, baseline.as_deref(), &now) {
        SubagentStopRoute::NotGated => {
            println!(
                "{}",
                serde_json::json!({"systemMessage": "[keel] subagent tree not gated: no baseline fingerprint for this agent or session (first hook fire was this one)"})
            );
            0
        }
        SubagentStopRoute::Silent => 0, // wrote nothing — pays nothing
        SubagentStopRoute::VerifierTreeWritten => {
            note_verdict("refused", "verifier:tree-written");
            println!(
                "{}",
                serde_json::json!({"systemMessage": "[keel] verifier:tree-written - the tree moved during a VERIFIER's lifetime; a verifier writes one receipt and nothing under the project (delegated-ceremony dcyVerifierReceipt, D0502). Not a block: ledgered for the census, and the tree is the primary's to read at the turn gate."})
            );
            0
        }
        SubagentStopRoute::Gate => {
            // The tree changed under this subagent: same gate as the turn boundary. A block for a RECORDER
            // (dcDelegatedCeremonyIsASkill clause d) is ledgered under `recorder:tree-red`, so the census
            // counts how often the D0425 recorder role leaves a red tree; the payload's `agent_type` is the
            // custom agent's name (Claude Code 2.1.270: SubagentStop carries agent_id, agent_type,
            // agent_transcript_path, stop_hook_active). Any other agent keeps the control the gate named.
            let code = hook_stop(payload, root);
            if let Ok(mut g) = EMITTED_VERDICT.lock() {
                if let Some((decision, control)) = g.as_mut() {
                    if decision == "block" {
                        let relabelled = subagent_block_control(agent_type, control);
                        *control = relabelled;
                    }
                }
            }
            code
        }
    }
}

/// The control a subagent-stop BLOCK is ledgered under: `recorder:tree-red` when the stopped agent's
/// type is `recorder` (the D0425 recorder, `.claude/agents/recorder.md`), otherwise the control the
/// gate itself named. Pure, so the pair is pinned without a tree or a hook fire.
fn subagent_block_control(agent_type: Option<&str>, gate_control: &str) -> String {
    if agent_type == Some("recorder") {
        "recorder:tree-red".to_string()
    } else {
        gate_control.to_string()
    }
}

/// `PreToolUse` on Bash: advise on host/shell adaptation before the command runs (issue094).
///
/// ADVISORY ONLY — always returns 0. A blocking heuristic over shell commands is the issue076/
/// issue081 dynamic where an over-strict gate trains its actor to disable it, and the checks here
/// cannot be exact: whether an MSYS path is wrong depends on what reads it.
///
/// Prints nothing when there is nothing to say, which is the common case and the point: a hook that
/// comments on ordinary commands becomes noise the reader skips, at which point it looks like
/// coverage while providing none.
/// Whitespace tokenization with single/double-quote awareness — argv-level, so a commit MESSAGE
/// describing `--no-verify` never matches (D0176/P1.2: no raw-string regex over commands).
fn bash_tokens(cmd: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in cmd.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => quote = Some(c),
            (None, c) if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            (_, c) => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// The D0176/D0178 Bash matcher verdict for one command.
enum BashVerdict {
    Clean,
    /// Unambiguous control-bypass pattern — blocking in strict, advisory otherwise.
    Block(String),
    /// Human-judgment / actor-identity operation — never keel-exempt (K6); ask in strict.
    Ask(String),
}

/// Classify a Bash command per the accepted tiering. UNQUOTED tokens only for operator detection —
/// tokenization strips quotes, so `>` inside a message is not a redirect... it IS stripped into the
/// token stream; operators are matched as standalone tokens or known flags, never substrings of
/// prose (the tokenizer keeps quoted text glued to its word, so `"a > b"` is one token, not three).
fn bash_classify(root: &Path, cmd: &str) -> BashVerdict {
    let toks = bash_tokens(cmd);
    let has_tok = |t: &str| toks.iter().any(|x| x == t);
    let tracking_target =
        toks.iter().any(|x| {
            let tracked = (x.contains(".tracking/") || x.contains(".tracking\\")) && std::path::Path::new(x).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml"));
            tracked || x.contains(".claude/settings") || x.contains(".claude\\settings")
        });
    // Unambiguous operator-level bypass patterns (issue116 vector included).
    if (has_tok(">") || has_tok(">>") || has_tok("tee")) && tracking_target {
        return BashVerdict::Block("redirection into a fact surface - use the keel write API".to_string());
    }
    if has_tok("sed") && has_tok("-i") && tracking_target {
        return BashVerdict::Block("in-place sed over a fact surface - use the keel write API".to_string());
    }
    if toks.windows(2).any(|w| matches!(w, [a, b] if a.ends_with("git") && b == "commit")) && has_tok("--no-verify") {
        return BashVerdict::Block("git commit --no-verify skips the commit gate".to_string());
    }
    if toks.iter().any(|x| x == "SKIP_VALIDATE=1" || x == "SKIP_KEEL=1") {
        return BashVerdict::Block("SKIP_VALIDATE/SKIP_KEEL bypasses the gate - fix the red instead".to_string());
    }
    if toks.windows(3).any(|w| matches!(w, [a, b, c] if a.ends_with("git") && b == "config" && c.contains("core.hooksPath"))) {
        return BashVerdict::Block("git config core.hooksPath is the issue116 control-bypass vector (K7)".to_string());
    }
    // The keel carve-out (D0178): keel-invoking commands are exempt EXCEPT the derived
    // human-judgment/actor-identity set, which is never exempt.
    let keel_idx = toks.iter().position(|t| {
        let base = t.rsplit(['/', '\\']).next().unwrap_or(t);
        base == "keel" || base == "keel.exe"
    });
    if let Some(i) = keel_idx {
        if let Some(sub) = toks.get(i + 1) {
            // D0451 folded `apply-review` under `record`, so a protected op may be two tokens
            // (`record review`); the check reads the verb and the verb with its sub-verb.
            let two = toks.get(i + 2).map(|n| format!("{sub} {n}"));
            let hit = keel_write::write::human_judgment_ops()
                .into_iter()
                .find(|op| *op == sub.as_str() || two.as_deref() == Some(*op));
            if let Some(op) = hit {
                return BashVerdict::Ask(format!(
                    "keel {op} records human judgment or mutates actor identity (K6/K7) - it runs only from a channel the human holds"
                ));
            }
        }
    }
    // Invocations carrying a PERSON's identity (KEEL_ACTOR= / --by / --judged-by naming a Person).
    let persons = keel_actor::actor::person_names(root);
    if !persons.is_empty() {
        for (j, t) in toks.iter().enumerate() {
            let named = t
                .strip_prefix("KEEL_ACTOR=")
                .map(str::to_string)
                .or_else(|| (t == "--by" || t == "--judged-by").then(|| toks.get(j + 1).cloned().unwrap_or_default()));
            if let Some(n) = named {
                if persons.iter().any(|p| p == &n) {
                    return BashVerdict::Ask(format!(
                        "this command writes as the PERSON `{n}` (K6) - a human identity is asserted only from the human's own channel"
                    ));
                }
            }
        }
    }
    BashVerdict::Clean
}

fn hook_pre_bash(payload: &serde_json::Value, root: &Path, session: &str) -> i32 {
    let cmd = payload.pointer("/tool_input/command").and_then(serde_json::Value::as_str).unwrap_or_default();
    // D0309 / issue372: a heredoc whose body carries a backslash is DENIED in every profile - the one
    // pre-bash verdict that blocks. The harness collapses `\\` before bash runs, so source written
    // through a heredoc is silently rewritten; this recurred eight-plus times in a week after being
    // tracked, which is the proof that an advisory and a memory were not controls (D0047).
    if let Some((tag, line)) = crate::shellcheck::heredoc_with_backslash(cmd) {
        hook_refuse(
            "heredoc-backslash",
            &serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                "permissionDecisionReason": format!("[keel] heredoc <<{tag} carries a backslash (`{line}`) - this harness collapses backslash pairs before bash runs, so source written this way is silently rewritten (D0309/issue372, recurred 8+ times). No shell path survives it: a string replacement in an existing file is the Edit tool (or python scripts/textpatch.py), a new file is the Write tool run by path; a heredoc is for prose without backslashes.")}}),
        );
        ledger_advisory(root, session, "heredoc-backslash denied");
        return 0;
    }
    // D0491 / issue564: a `cat`/`tee` at the head of a pipeline with nothing feeding it - no heredoc,
    // no `<`, no operand - waits on a stdin this harness never closes, for the whole tool timeout,
    // and leaves the file it opened empty. Never useful, so it is the second pre-bash deny.
    if let Some(seg) = crate::shellcheck::stdin_starved_write(cmd) {
        hook_refuse(
            "stdin-starved-write",
            &serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                "permissionDecisionReason": format!("[keel] `{seg}` has nothing feeding it - no heredoc, no `<`, no file operand - so it waits on stdin for the whole tool timeout and the file it opened stays empty (D0491/issue564; met as a two-minute hang). A new file is the Write tool run by path; a string replacement is the Edit tool.")}}),
        );
        ledger_advisory(root, session, "stdin-starved-write denied");
        return 0;
    }
    // D0176/D0178 tiering first: unambiguous bypass patterns and the never-exempt set.
    let profile = adoption_profile(root);
    match bash_classify(root, cmd) {
        BashVerdict::Block(why) => {
            if profile == "strict" {
                hook_refuse(
                    "strict-bash-verdict",
                    &serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "deny",
                        "permissionDecisionReason": format!("[keel] {why} (D0176; blocking under the strict profile)")}}),
                );
                return 0;
            }
            println!("[keel] BLOCKED-under-strict pattern: {why} (advisory here; promotion cites the fire-ledger, D0180)");
            ledger_advisory(root, session, &why);
        }
        BashVerdict::Ask(why) => {
            if profile == "strict" {
                println!(
                    "{}",
                    serde_json::json!({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "ask",
                        "permissionDecisionReason": format!("[keel] {why}")}})
                );
                return 0;
            }
            println!("[keel] human-channel operation: {why} (ask-tier under strict; advisory here)");
            ledger_advisory(root, session, &why);
        }
        BashVerdict::Clean => {}
    }
    let advisories = crate::shellcheck::inspect(cmd);
    if advisories.is_empty() {
        return 0;
    }
    println!("[shell-adaptation -- CLAUDE.md sec 6, the #1 avoidable-friction class (issue094)]");
    let mut spoken = String::new();
    for a in &advisories {
        println!("  {}", a.what);
        println!("    fix: {}", a.fix);
        spoken.push_str(&a.what);
    }
    println!("  Advisory only -- nothing is blocked. If the command errors or hangs, SWITCH TOOLS rather than re-issuing the same form.");
    // issue230 (D0197's untriggerable revisit condition): an advisory that SPEAKS leaves its own
    // ledger event, and speaking the SAME advice again in the same session leaves a repeat event —
    // the mechanical ignore signal. Heeded is then computable as issued-without-repeat
    // (approximate, and enforcement-report says so). Silent fires stay one plain pre-bash line.
    ledger_advisory(root, session, &spoken);
    0
}

/// The issue230 advisory instrumentation: emit `advisory-issued` for a spoken advisory, plus
/// `advisory-repeated` when the same advice hash was already the session's last one — all within
/// the frozen 6-field schema (events are declared vocabulary, fields are not touched).
fn ledger_advisory(root: &Path, session: &str, spoken: &str) {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    spoken.hash(&mut h);
    let digest = format!("{:x}", h.finish());
    let dir = root.join(".keel").join("metrics");
    let _ = std::fs::create_dir_all(&dir);
    let marker = dir.join(format!("last-advice-{}.hash", if session.is_empty() { "nosession" } else { session }));
    let repeated = std::fs::read_to_string(&marker).is_ok_and(|prev| prev.trim() == digest);
    let _ = std::fs::write(&marker, &digest);
    ledger_emit(root, session, "advisory-issued", 0, 0);
    if repeated {
        ledger_emit(root, session, "advisory-repeated", 0, 0);
    }
}

/// `UserPromptSubmit`: inject the route-first checklist, plus a warning about out-of-band writes.
///
/// ASCII-only on purpose: this text is injected into the model's context via stdout, and a non-UTF-8
/// Windows console turns non-ASCII into mojibake.
/// Recalled facts for this prompt, or `None` when nothing should be pushed (D0242 part 3).
///
/// # PUSH, not pull — and why this function is the whole point
///
/// The source process (`.engine/processes/knowledge-graph-memory.sysml`, step `kgInjection`) states it
/// plainly: "THIS STEP IS WHAT MAKES IT PUSH RATHER THAN PULL — a graph the model must decide to query
/// is still pull, and inherits every cost this process exists to remove." Seven of the eight spine
/// steps were built in sprint 424; this is the eighth, and its absence is why the store existed for a
/// week with no measurable effect.
///
/// # THREE WAYS TO TURN IT OFF, in the order a human would reach for them
///
/// 1. `keel deactivate knowledge-graph-memory` — the DECLARED off. The process is a deactivatable unit
///    (D0138), so the switch is committed, auditable, and travels with the project. This is the one to
///    use: a project that turned recall off has DECLARED it, rather than having it quietly not work.
/// 2. `KEEL_RECALL=off` — the immediate, machine-local off, for "it is misbehaving right now" without a
///    commit. Deliberately env-based so it cannot be mistaken for a project decision.
/// 3. Doing nothing — a prompt with no informative token pushes nothing. This is NOT a confidence
///    gate: one was built, measured at 2/8 against 5/8 for no gate at all, and removed (issue297). The
///    surviving check asks only whether there is anything to say, never whether to trust it.
///
/// Note what is NOT a kill switch any more: deleting `.knowledge/`. D0243 made seeding corpus-derived,
/// so removing the store costs aliases and question-coverage, not recall. Saying so matters, because
/// that used to be the removability story.
///
/// FAIL-OPEN, always. Any error, any timeout, any absent store returns `None` and the turn proceeds:
/// injection is an advantage, never a dependency, and a broken index must never cost a turn.
fn recalled_facts(root: &Path, payload: &serde_json::Value, session: &str) -> Option<String> {
    if std::env::var("KEEL_RECALL").is_ok_and(|v| v.eq_ignore_ascii_case("off")) {
        return None;
    }
    if !keel_model::activation::Activation::load(root).is_process_active("knowledge-graph-memory") {
        return None;
    }
    let prompt = payload.get("prompt").and_then(serde_json::Value::as_str)?;
    if prompt.trim().is_empty() {
        return None;
    }
    let started = std::time::Instant::now();
    // The confidence verdict and the payload are computed from the same model build, so the cost is
    // paid once. `recall_for_prompt` returns its own "nothing pushed" text for an uninformative prompt,
    // which is a PULL answer - useful at a CLI, noise in a prompt - so the verdict gates it here.
    if !keel_view::view::has_pushable_facts(root, prompt).unwrap_or(false) {
        return None;
    }
    let facts = keel_view::view::recall_for_prompt(root, prompt, RECALL_BUDGET).ok()?;
    let ms = started.elapsed().as_millis();
    // A LATENCY CAP that reports rather than hides: past the cap the facts are dropped for this turn
    // and the reader is told, because a recall that silently doubles every turn's latency is the kind
    // of cost that gets discovered months later.
    if ms > RECALL_CAP_MS {
        // D0390 option A (the human's word 2026-09-09): the cap is read AFTER the walk, so a drop saves
        // nothing - the turn has already paid. Push the facts anyway with the latency NAMED, and count
        // the fire as `recall-slow` (recall-skipped no longer fires; it stays in the ledger as history).
        // The memory channel is least available when the machine is busiest, so it must not be cut then.
        ledger_emit(root, session, "recall-slow", 0, ms);
        return Some(format!(
            "[keel recall — pushed LATE, {ms}ms over the {RECALL_CAP_MS}ms cap (D0390); the walk had already run]\n{facts}"
        ));
    }
    // The VISIBLE recall count and elapsed time the process names as this step's produced artifact.
    Some(format!("[keel recall — pushed before the model, {ms}ms]\n{facts}"))
}

fn hook_user_prompt(root: &Path, payload: &serde_json::Value, session: &str) -> i32 {
    // PUSH FIRST, then the routing contract: the facts have to be in front of the model when it wakes,
    // and the contract is what it should do with them.
    if let Some(facts) = keel_perf::perf::phase("hook:recall", || recalled_facts(root, payload, session)) {
        print!("{facts}");
    }
    // D0064/D0106: routing is structural, fired every turn rather than left to vigilance.
    println!(
        "[engine-triage -- route FIRST (D0064)] Break the request into parts and route EACH before \
acting: CHANGE (sec 3a: workflow/phase/gate/schema) | EXECUTE (sec 3b: tracked artifact, sprinted) \
| RECORD (sec 3c: one atomic fact -- decision/test result/issue) | VIEW (sec 3d: computed answer) | \
ORIENT (sec 3f: where things stand). Flag anything that does NOT cleanly map -- ask, don't \
force-fit. Substantive work goes through a sprint (only trivial one-off edits are exempt). \
method=confirmation needs explicit human sign-off. Invoke the engine-triage skill if unsure."
    );

    // While `keel serve` is live the human authors facts straight into the tree (accepting Decisions,
    // editing items). Those land uncommitted with no signal, and a blanket `git add -A` once swept an
    // accepted D0126/D0127 into a sprint commit unnoticed. Silent when the tree is clean.
    let git = |args: &[&str]| -> String {
        keel_git::gitx::git()
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default()
    };
    let status = git(&["status", "--porcelain", "--", ".tracking", ".engine"]);
    let status = status.trim();
    if status.is_empty() {
        return 0;
    }
    println!("[out-of-band-writes] Uncommitted changes in .tracking/.engine at turn start - possibly authored");
    println!("via keel serve (human accepts/edits/creates), NOT by me. Run `git diff` and stage DELIBERATELY;");
    println!("do NOT blanket `git add -A` over human-attested writes (accepted Decisions, edits). Changed:");
    for line in status.lines().take(40) {
        println!("  {line}");
    }

    // Surface acceptance/status changes specifically: those are the HUMAN's sign-off and must stay
    // attributed to them, never folded silently into an AI commit.
    let diff = git(&["diff", "--", ".engine/decisions"]);
    let accepts: Vec<&str> = diff
        .lines()
        .filter(|l| {
            l.starts_with('+')
                && (l.contains("DecisionStatus::accepted")
                    || l.contains("DecisionStatus::rejected")
                    || l.contains("Accept :")
                    || l.contains("Reject :")
                    || l.contains("judgedBy"))
        })
        .take(20)
        .collect();
    if !accepts.is_empty() {
        println!("DECISION ACCEPTANCE / STATUS CHANGES (human sign-off - verify + keep as THEIR attributed record):");
        for l in accepts {
            println!("  {l}");
        }
    }
    0
}

/// Emit a hook-protocol JSON object and exit 0 (the harness reads stdout).
fn hook_emit(v: &serde_json::Value) -> i32 {
    println!("{v}");
    0
}

/// Run the fast gate over an edited `.sysml` file; block with the violations if it broke the model.
fn hook_post_edit(payload: &serde_json::Value, root: &Path) -> i32 {
    let path = payload
        .pointer("/tool_input/file_path")
        .or_else(|| payload.pointer("/tool_response/filePath"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if !std::path::Path::new(path).extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml")) {
        return 0; // only model files are gated
    }

    let mut problems: Vec<String> = Vec::new();
    let report = keel_perf::perf::phase("hook:validate", || keel_model::validate::validate_root(root));
    for (p, d) in &report.diagnostics {
        problems.push(format!("ERROR: {}:{} — {}", p.display(), d.line, d.message));
    }
    for e in &report.errors {
        problems.push(format!("PARSE: {} — {}", e.file.display(), e.message));
    }
    // Only the EXACT guards — a per-edit gate must never fire on a heuristic (see cmd_gate).
    for name in ["duplicate-identity", "marker-vocabulary"] {
        if let Some(r) = keel_perf::perf::phase("hook:fast-guards", || keel_guards::run_one(name, root)) {
            for v in &r.violations {
                problems.push(format!("GUARD [{name}]: {v}"));
            }
        }
    }
    if problems.is_empty() {
        // Fast tier clean -> the model is not BROKEN, but the edit may have broken something
        // DOWNSTREAM. D0209 clause 4 (dcProactivePostEdit): surface it as NON-BLOCKING guidance so
        // the author fixes it at the point of edit, not at commit. Silent when there is nothing.
        let rel = std::path::Path::new(path)
            .strip_prefix(root)
            .unwrap_or_else(|_| std::path::Path::new(path))
            .to_string_lossy()
            .replace('\\', "/");
        let advisories = keel_perf::perf::phase("hook:advisory", || crate::proactive::post_edit_advisories(root, &rel));
        if advisories.is_empty() {
            return 0; // clean -> silent, so a passing gate costs nothing
        }
        let mut body = advisories.join("\n");
        body.truncate(2000);
        return hook_emit(&serde_json::json!({
            "systemMessage": format!(
                "[proactive — non-blocking] That edit may have broken something downstream:\n\n{body}\n\nThe model still parses, so this does NOT block — but fix it now, at the point of the edit, before it reaches a commit gate."
            )
        }));
    }
    let mut body = problems.join("\n");
    body.truncate(2000);
    hook_refuse("edit-gate", &serde_json::json!({
        "decision": "block",
        "reason": format!(
            "[edit gate] That edit left the model broken — fix it now, at the point of the edit:\n\n{body}\n\nThis is the FAST tier (validate + duplicate-identity + marker-vocabulary, all exact). Author through the keel write API where one exists."
        )
    }))
}

/// The default console port. One constant, because the hook advisory names it to the reader and a
/// wrong number in an advisory is worse than no advisory.
const CONSOLE_PORT: u16 = 7777;

/// How long a hook process may live before it gives up and exits 0 (issue180b).
///
/// Generous enough that the real work always finishes - the stop hook runs validate plus 38 guards,
/// measured at 6-10s - and short enough that an orphaned process releases the binary before it blocks a
/// build. The alternative, an unbounded wait, wedged three builds in a single turn.
const HOOK_DEADLINE_SECS: u64 = 120;

/// Is a KEEL console answering on `127.0.0.1:<port>`?
///
/// It asks `/api/version` and looks for `apiVersion` rather than merely opening a socket, because
/// "something is listening on 7777" is not the claim being made. Reporting an unrelated process as a
/// running console would be the same class of defect as issue140 and issue149 - a tool asserting more
/// than it checked - and here it would tell the human their work is reachable when it is not.
/// Turn-boundary gate: refuse to end the turn while the model is dishonest. Loop-safe.
fn hook_stop(payload: &serde_json::Value, root: &Path) -> i32 {
    if let Some(w) = engine_version_skew(root) {
        // D0251: the turn boundary is a gate. The skew is reported as a blocking problem (with the
        // repair named) rather than a warning nobody reads.
        eprintln!("{w}");
    }
    let already = payload.get("stop_hook_active").and_then(serde_json::Value::as_bool).unwrap_or(false);

    // THE GUARD RECEIPT (dcGateAnswersFromItsReceipt, D0367 rank 4). A turn boundary whose tree, binary
    // and .keel/ inputs are byte-for-byte the ones the last green run judged is the same question, and
    // it gets the same answer without recomputing it - silently, as any green turn is (D0359). The key
    // is what makes that honest (see receipt.rs); `KEEL_NO_RECEIPT=1` forces the run.
    let receipt_key = keel_perf::perf::phase("hook:receipt-key", || if keel_guards::receipt::forced(&[]) { None } else { keel_guards::receipt::key(root) });
    if let Some(k) = &receipt_key {
        if keel_guards::receipt::read(root, k).is_some_and(|r| r.covers_all(&keel_guards::receipt::ALL_LAYERS)) {
            return 0;
        }
    }

    // Each serial step is a named phase (D0414 / issue429) so a slow fire's ledger line can say
    // which one it was; the guard runner names its own critical path inside `hook:guards`.
    let mut problems: Vec<String> = Vec::new();
    let report = keel_perf::perf::phase("hook:validate", || keel_model::validate::validate_root(root));
    if !report.diagnostics.is_empty() || !report.errors.is_empty() {
        use std::fmt::Write as _;
        let mut s = String::from("keel gate validate:\n");
        for (p, d) in report.diagnostics.iter().take(10) {
            let _ = writeln!(s, "  {}:{} — {}", p.display(), d.line, d.message);
        }
        for e in report.errors.iter().take(10) {
            let _ = writeln!(s, "  {} — {}", e.file.display(), e.message);
        }
        problems.push(s);
    }
    // Through run_all, not a run_one loop: run_all applies the ACTIVATION filter, so a guard whose
    // process this project deactivated no longer blocks turns (D0177/P1.5 fixing the guards.rs
    // bypass the proposal cited — hook_stop was the one caller that skipped the filter).
    let mut failing: Vec<String> = Vec::new();
    let (reports, durations) = keel_perf::perf::phase("hook:guards", || keel_guards::run_all_timed(root));
    for r in &reports {
        for v in r.violations.iter().take(5) {
            failing.push(format!("  [{}] {v}", r.name));
        }
    }
    if !failing.is_empty() {
        problems.push(format!("keel gate guard:\n{}", failing.join("\n")));
    }
    // Declared rules gate the turn (D0177/P1.5): blocking rules block; warning rules report only at
    // their own surfaces (`keel gate rules`), not here — a turn boundary repeats no warning noise.
    match keel_perf::perf::phase("hook:rules", || keel_view::view::check(root)) {
        Ok(json) => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                let empty = Vec::new();
                let mut broken: Vec<String> = Vec::new();
                for r in v.get("rules").and_then(|x| x.as_array()).unwrap_or(&empty) {
                    if r.get("severity").and_then(|s| s.as_str()) == Some("blocking") {
                        for viol in r.get("violations").and_then(|x| x.as_array()).unwrap_or(&empty).iter().take(5) {
                            broken.push(format!("  [rule {}] {viol}", r.get("rule").and_then(|s| s.as_str()).unwrap_or("?")));
                        }
                    }
                }
                if !broken.is_empty() {
                    problems.push(format!("keel gate rules (blocking):\n{}", broken.join("\n")));
                }
            }
        }
        Err(e) => problems.push(format!("keel gate rules: cannot evaluate declared rules: {e}")),
    }

    // THE HUMAN MAY BE BLOCKED WHETHER OR NOT THE MODEL IS (issue150, answering their own question about
    // how this went unnoticed). The first version of this advisory sat INSIDE the green branch, so a red
    // model masked it entirely: the turn reported the model and said nothing about an unreachable queue -
    // the same single-channel blindness that let the console stay down while 70 items waited. The console
    // is their oversight lens (D0093) and its availability is not a function of the model's honesty.
    //
    // THE OVERSIGHT NAG IS GONE (D0269). It counted items waiting on the human and then prescribed
    // how to look at them - start a console on 127.0.0.1, or publish a deck - branching on an env
    // var as a proxy for whether they were at this terminal or elsewhere. Their verdict: I cannot
    // tell where they are reading, the proxy was never evidence that I could, and the line carried
    // almost no value even when right. What waits on them is now surfaced deliberately, as a linked
    // decision page, by the decision-surfacing process - a channel they asked for and can act in,
    // rather than a count appended to every turn.

    if problems.is_empty() {
        // GREEN -> SILENT (D0359). For one day this branch emitted one line naming the Decisions
        // awaiting the owner's word (GH#52/D0351). It was accurate every time, which is exactly why
        // it stopped being read: a per-turn restatement of an unchanged fact trains the reader past
        // it. Their instruction, 2026-09-06: "I don't want stop says text anymore. remove."
        //
        // Nothing is lost to automation - `keel show orient` still answers pendingAcceptances - and
        // nothing is lost to the human, PROVIDED the page is genuinely refreshed: the queue is
        // surfaced as the published decision brief by the decision-surfacing post-analysis, which
        // republishes on a CHANGE in the pending set and is silent otherwise. That the post-analysis
        // runs at all is not enforceable here (no gate reads conversational output, D0151); what is
        // enforceable is that this hook does not substitute a count for it.
        if let Some(k) = &receipt_key {
            let _ = keel_guards::receipt::record_green(root, k, &keel_guards::receipt::ALL_LAYERS, &reports, &durations);
        }
        return 0;
    }
    // A red run leaves no receipt - the next boundary recomputes.
    keel_guards::receipt::delete(root);
    if already {
        // Second consecutive red: allow the stop (loop-avoidance stands, issue081) but the yield is
        // now a TRACKED obligation visible in orient, surviving console downtime (D0176/P1.7) — a
        // yield that lives only in a hook message evaporates with the transcript.
        let session = payload.get("session_id").and_then(serde_json::Value::as_str).unwrap_or("");
        // GH#50 / D0350: the first problem is the fact this record exists to preserve - whole, or cut on
        // a line with the cut counted; `chars().take(500)` used to cut it mid-token.
        let first = problems.first().map(|p| keel_write::write::clip_at_line_boundary(p, 2000)).unwrap_or_default();
        // An unresolvable actor goes straight to the ledger: a tracked fact with a fabricated
        // createdBy would deepen the red it records (the actors guard would fire on it).
        let recorded = keel_actor::actor::resolve(root, None).map_err(keel_write::write::WriteError::Parse).and_then(|actor| {
            keel_write::write::record_obligation(
                root,
                "red-yield",
                "turn gate yielded while red - the tree needs a correction pass",
                &format!("The Stop hook's second red pass yielded (loop avoidance). Session: {session}. First problem at yield: {first}. Discharge: make the tree green and triage this obligation with a #Resolves edge."),
                &actor,
            )
        });
        let note = match recorded {
            Ok(p) => format!("A tracked obligation was recorded at {} (orient-visible).", p.display()),
            Err(e) => {
                ledger_emit(root, session, "red-yield-obligation-UNSYNCED", 1, 0);
                format!("The obligation could NOT be tracked ({e}) - it sits in the local ledger; record it once the tree is writable.")
            }
        };
        return hook_emit(&serde_json::json!({
            "systemMessage": format!("[in-loop gate] Still red after a correction pass — allowing the stop to avoid a loop. Do NOT commit until keel gate validate + guard are green. {note}")
        }));
    }
    let mut body = problems.join("\n\n");
    body.truncate(4000);
    hook_refuse("in-loop-gate", &serde_json::json!({
        "decision": "block",
        "reason": format!(
            "[in-loop gate] The model is not in honest state — resolve before ending the turn:\n\n{body}\n\nFix through the keel write API (record result / record task / record decision); run `keel gate guard <name>` for detail. Then end the turn."
        )
    }))
}

#[cfg(test)]
mod subagent_block_control_tests {
    use super::subagent_block_control;

    /// D0388 pair for dcDelegatedCeremonyIsASkill clause (d): a block under a `recorder` agent is
    /// `recorder:tree-red` whatever the gate named (known positive); a general-purpose agent, an
    /// absent `agent_type` and a `verifier` keep the gate's own control (known negative) - the
    /// verifier writes nothing, so a red under it is the primary's red, not a recorder's.
    #[test]
    fn a_recorder_block_is_counted_as_recorder_tree_red_and_nothing_else_is() {
        assert_eq!(subagent_block_control(Some("recorder"), "in-loop-gate"), "recorder:tree-red");
        assert_eq!(subagent_block_control(Some("recorder"), "edit-gate"), "recorder:tree-red");
        assert_eq!(subagent_block_control(Some("general-purpose"), "in-loop-gate"), "in-loop-gate");
        assert_eq!(subagent_block_control(Some("verifier"), "in-loop-gate"), "in-loop-gate");
        assert_eq!(subagent_block_control(None, "in-loop-gate"), "in-loop-gate");
    }
}

#[cfg(test)]
mod subagent_stop_route_tests {
    use super::{agent_baseline_path, subagent_baseline, subagent_stop_route, SubagentStopRoute};

    const SESSION_START: &str = "1111"; // the session's first-fire tree
    const AGENT_START: &str = "2222"; // the tree when the agent began - the primary edited in between
    const AGENT_MOVED: &str = "3333"; // the tree after the agent itself wrote

    /// D0388 pair for dcVerifierStopIsNeverABlock (D0501/D0502), stated before the tree is read.
    /// Known positive: a verifier over a tree that differs from the SESSION baseline but not from its
    /// own start is silent - no block, no refused line; a verifier whose own fingerprint moved is the
    /// ledgered control `verifier:tree-written`, still never the gate.
    #[test]
    fn a_verifier_is_never_gated_and_its_own_write_is_the_ledgered_fact() {
        assert_eq!(subagent_stop_route(Some("verifier"), Some(AGENT_START), AGENT_START), SubagentStopRoute::Silent, "the primary's edits before the agent started are not the agent's");
        assert_eq!(subagent_stop_route(Some("verifier"), Some(AGENT_START), AGENT_MOVED), SubagentStopRoute::VerifierTreeWritten);
        // the pre-D0501 measurement would have gated this verifier over the primary's red (issue578)
        assert_ne!(subagent_stop_route(Some("verifier"), Some(SESSION_START), AGENT_START), SubagentStopRoute::Gate, "a verifier never reaches the gate, whichever baseline it fell back to");
    }

    /// Known negative: a recorder over a moved tree still reaches the gate (whose block is relabelled
    /// `recorder:tree-red` by `subagent_block_control`), a payload with no `agent_type` keeps the gate,
    /// and an agent with no baseline at all is advised, not blocked.
    #[test]
    fn a_recorder_and_an_untyped_agent_keep_the_gate_over_a_moved_tree() {
        assert_eq!(subagent_stop_route(Some("recorder"), Some(AGENT_START), AGENT_MOVED), SubagentStopRoute::Gate);
        assert_eq!(subagent_stop_route(None, Some(AGENT_START), AGENT_MOVED), SubagentStopRoute::Gate);
        assert_eq!(subagent_stop_route(Some("general-purpose"), Some(AGENT_START), AGENT_START), SubagentStopRoute::Silent);
        assert_eq!(subagent_stop_route(Some("recorder"), None, AGENT_MOVED), SubagentStopRoute::NotGated);
        assert_eq!(subagent_stop_route(Some("verifier"), None, AGENT_MOVED), SubagentStopRoute::NotGated);
    }

    /// D0501: the agent's own start file wins over the session baseline when it exists; a payload with
    /// no `agent_id`, or whose agent left no file, reads the session baseline as before.
    #[test]
    #[allow(clippy::expect_used)] // test setup: a failed mkdir or write should abort the test loudly
    fn the_agents_own_start_is_read_before_the_sessions() {
        let root = keel_fs::scratch("subagent-stop-route");
        let metrics = root.join(".keel").join("metrics");
        std::fs::create_dir_all(&metrics).expect("metrics dir");
        std::fs::write(metrics.join("baseline-s1.fp"), SESSION_START).expect("session baseline");
        std::fs::write(agent_baseline_path(&root, "agent/one"), AGENT_START).expect("agent baseline");
        assert!(agent_baseline_path(&root, "agent/one").ends_with("agent-agent_one.fp"), "a separator in the id is folded, never a path");
        let own = serde_json::json!({"agent_id": "agent/one", "agent_type": "verifier"});
        let no_file = serde_json::json!({"agent_id": "agent-two", "agent_type": "verifier"});
        let no_id = serde_json::json!({"agent_type": "verifier"});
        assert_eq!(subagent_baseline(&root, &own, "s1").as_deref(), Some(AGENT_START));
        assert_eq!(subagent_baseline(&root, &no_file, "s1").as_deref(), Some(SESSION_START));
        assert_eq!(subagent_baseline(&root, &no_id, "s1").as_deref(), Some(SESSION_START));
        assert_eq!(subagent_baseline(&root, &no_id, "s2"), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
mod tests {
    use super::{bash_classify, bash_tokens, consume_override, override_path, override_target, BashVerdict};

    /// issue427 (stpa-self run 2, UCA-O1): an override unlock covers exactly one file. A directory is
    /// refused by name; an unlock for file A is NOT consumed by a write to file B whose path contains
    /// A's as a substring (the old either-direction match let it); the ordinary one-file override
    /// still consumes once, records its obligation, and is gone for the second write.
    #[test]
    #[allow(clippy::expect_used)] // test setup: a failed mkdir or write should abort the test loudly
    fn override_unlocks_exactly_one_file() {
        let root = std::env::temp_dir().join(format!("keel-override-one-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        std::fs::create_dir_all(root.join(".keel")).expect("mkdir");
        let a = root.join(".tracking").join("issues.sysml");
        let b = root.join(".tracking").join("issues.sysml.bak");
        std::fs::write(&a, "package A {\n}\n").expect("a");
        std::fs::write(&b, "package B {\n}\n").expect("b");

        // A directory is refused, and the refusal names the rule.
        let refused = override_target(&root, ".tracking").expect_err("a directory must be refused");
        assert!(refused.contains("directory") && refused.contains("ONE FILE"), "{refused}");
        // A file nowhere, outside every API-owned surface, is refused too.
        let missing = override_target(&root, ".tracking/nothing-here.sysml").expect_err("a missing file outside the API surfaces must be refused");
        assert!(missing.contains("does not exist"), "{missing}");
        // A single NEW file under an API-owned surface is allowed: the API would own its creation.
        assert!(override_target(&root, ".tracking/issues-probe.sysml").is_ok(), "a new per-actor issues file is the sanctioned exception");
        // The ordinary target: an existing file, stored canonically (forward slashes, no `//?/`).
        let key = override_target(&root, ".tracking/issues.sysml").expect("an existing file arms");
        assert!(key.ends_with("/.tracking/issues.sysml") && !key.contains('\\') && !key.starts_with("//?/"), "{key}");

        // Arm A the way `keel override` does, then write B (contains A's path as a substring): NOT consumed.
        let arm = || {
            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            std::fs::write(override_path(&root), serde_json::json!({"path": key, "reason": "probe reason of enough length", "actor": "probeAi", "ts": ts}).to_string()).expect("arm");
        };
        arm();
        let b_written = b.to_string_lossy().replace('\\', "/");
        assert!(consume_override(&root, &b_written, "s").is_none(), "a write to B must not consume A's unlock");
        assert!(override_path(&root).exists(), "the unlock survives a non-matching write");
        // The write to A itself consumes once and records the obligation; the second write finds no unlock.
        let a_written = a.to_string_lossy().replace('\\', "/");
        assert_eq!(consume_override(&root, &a_written, "s").as_deref(), Some("probe reason of enough length"));
        assert!(!override_path(&root).exists(), "single-use: the unlock is gone");
        let recorded = std::fs::read_dir(root.join(".tracking").join("obligations")).map_or(0, |rd| rd.flatten().count());
        assert_eq!(recorded, 1, "consumption records exactly one obligation");
        assert!(consume_override(&root, &a_written, "s").is_none(), "the second write after one unlock is locked again");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// D0176/P1.2: argv-level matching — operators as tokens, never substrings of prose. A commit
    /// message DESCRIBING --no-verify does not match; the real flag does; the keel carve-out
    /// exempts ordinary keel commands but NEVER the human-judgment set.
    #[test]
    #[allow(clippy::expect_used)] // test setup: a failed mkdir should abort the test loudly
    fn bash_matcher_is_argv_level_and_carves_out_human_judgment() {
        let root = keel_fs::scratch("keel-bash-classify");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".tracking")).expect("mkdir");
        std::fs::write(
            root.join(".tracking").join("actors.sysml"),
            "package A {\n    part hum : Person { :>> name = \"H\"; }\n}\n",
        )
        .expect("actors");
        assert!(matches!(bash_classify(&root, "git commit --no-verify -m x"), BashVerdict::Block(_)));
        assert!(
            matches!(bash_classify(&root, "git commit -m \"never use --no-verify\""), BashVerdict::Clean),
            "prose inside quotes is ONE token and must not match"
        );
        assert!(matches!(bash_classify(&root, "echo x > .tracking/issues.sysml"), BashVerdict::Block(_)));
        assert!(matches!(bash_classify(&root, "sed -i s/a/b/ .tracking/backlog.sysml"), BashVerdict::Block(_)));
        assert!(matches!(bash_classify(&root, "git config core.hooksPath /dev/null"), BashVerdict::Block(_)));
        assert!(matches!(bash_classify(&root, "SKIP_VALIDATE=1 git commit -m x"), BashVerdict::Block(_)));
        assert!(matches!(bash_classify(&root, "keel gate validate ."), BashVerdict::Clean), "ordinary keel is exempt");
        assert!(matches!(bash_classify(&root, "keel accept d1 --by hum"), BashVerdict::Ask(_)), "accept is never exempt");
        assert!(matches!(bash_classify(&root, "keel actor set hum"), BashVerdict::Ask(_)), "actor mutation is never exempt");
        assert!(
            matches!(bash_classify(&root, "keel record review --batch f"), BashVerdict::Ask(_)),
            "D0451: apply-review's K7 protection follows it to `record review`"
        );
        assert!(
            matches!(bash_classify(&root, "keel record task --file f --def D --task T"), BashVerdict::Clean),
            "a two-token spelling that is NOT protected stays the ordinary agent write"
        );
        assert!(
            matches!(bash_classify(&root, "KEEL_ACTOR=hum keel record result --file f --task t --sha s"), BashVerdict::Ask(_)),
            "writing AS a Person routes to the human channel"
        );
        assert!(
            matches!(bash_classify(&root, "KEEL_ACTOR=someAi keel record result --file f --task t --sha s"), BashVerdict::Clean),
            "writing as a non-Person is the normal agent case"
        );
        assert_eq!(bash_tokens("a \"b c\" d"), vec!["a", "b c", "d"], "quoted text glues to one token");
    }
}
