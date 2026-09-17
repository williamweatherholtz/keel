#!/usr/bin/env python3
"""decision_facts.py - the COMPUTED facts file the standing executive summary is built from.

Run from the keel repository root, no required arguments. Prints ONE JSON object to stdout and
writes the same object beside this script as `decision-facts.json`.

Contract (why this exists): every number on the published summary must come from here, and every
value here carries its own provenance - `how` is the exact command, or the exact file + parsing
rule, that produced it. A fact that cannot be computed honestly is emitted with "value": null and
a `how` that says why. Nothing is ever guessed, and no answer is hardcoded.

Python 3 stdlib only; shells out to git, gh and ./target/release/keel.exe.
"""

import glob
import json
import os
import re
import subprocess
import sys
import time
from datetime import date, datetime, timedelta, timezone

# ---------------------------------------------------------------- infrastructure

REPO = os.getcwd()
sys.path.insert(0, os.path.join(REPO, "scripts"))
from module_home import module_home as _mh, crate_text as _crate_text, crate_root as _crate_root, AmbiguousModule as _AmbiguousModule  # noqa: E402  (issue559: no keel-cli/src anchors)
# The binary is a COPY, never the build image: a running target/release/keel.exe blocks its own relink
# (issue150), and this script ran it under a cargo build once (issue508). KEEL_BIN wins; then the
# serve copy; the build image only when nothing else exists.
def _keel_bin():
    env = os.environ.get("KEEL_BIN")
    if env and os.path.exists(env):
        return env
    rel = os.path.join(REPO, "target", "release")
    for name in ("keel-serve.exe", "keel-serve", "keel.exe", "keel"):
        cand = os.path.join(rel, name)
        if os.path.exists(cand):
            return cand
    return os.path.join(rel, "keel.exe")


KEEL = _keel_bin()

TODAY = date.today()
NOW = datetime.now(timezone.utc)

# The artefact is CLAIMED before anything is computed (D0387/issue399): the previous answer is replaced
# by a running stub now, an uncaught exception below rewrites it as failed, and only the last line
# writes `complete: true`. A reader that does not go through artefact.require_complete is the defect.
sys.path.insert(0, os.path.join(REPO, "scripts"))
from artefact import begin_json, finish_json  # noqa: E402
OUT_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "decision-facts.json")
begin_json(OUT_PATH, "scripts/exec_brief/facts.py is running - this file is not an answer until complete is true")

FACTS = {}
NOTES = []


def fact(name, value, unit, how, as_of=None):
    """Record one fact. `value` may be None; `how` must then explain why."""
    FACTS[name] = {
        "value": value,
        "unit": unit,
        "as_of": as_of or TODAY.isoformat(),
        "how": how,
    }


def run(cmd, timeout=60):
    """Run a command, return (ok, stdout). Never raises; a failure becomes ok=False + the reason."""
    try:
        p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True,
                           timeout=timeout, encoding="utf-8", errors="replace")
        if p.returncode != 0 and not p.stdout.strip():
            return False, (p.stderr or "").strip()[:400] or ("exit %d" % p.returncode)
        return True, p.stdout
    except FileNotFoundError:
        return False, "executable not found: %s" % cmd[0]
    except subprocess.TimeoutExpired:
        return False, "timed out after %ss" % timeout
    except Exception as exc:                                        # pragma: no cover
        return False, "%s: %s" % (type(exc).__name__, exc)


def run_rc(cmd, timeout=60):
    """Run a command, return (returncode, stdout). A fact named exit0 reads THIS, never run()'s ok flag: run() answers
    ok=True whenever the command printed anything, so a checker that prints REFUSED and exits 1 read as exit0=True
    (issue553 - four facts on the twenty-sixth page were caught by the builder's own refusal)."""
    try:
        p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True,
                           timeout=timeout, encoding="utf-8", errors="replace")
        return p.returncode, (p.stdout or "") + (("\n" + p.stderr) if p.stderr and not (p.stdout or "").strip() else "")
    except FileNotFoundError:
        return 127, "executable not found: %s" % cmd[0]
    except subprocess.TimeoutExpired:
        return 124, "timed out after %ss" % timeout
    except Exception as exc:                                        # pragma: no cover
        return 1, "%s: %s" % (type(exc).__name__, exc)


# issue553 - the D0388 pair on the exit-code sensor, run before any fact: a command that prints and exits 1 reads rc 1,
# one that prints and exits 0 reads rc 0. run() answers ok=True for both, so no fact named for an exit may derive from it;
# the source scan holds that. Either failing refuses the whole emission - a page with no numbers beats one with a wrong name.
_rc_pos, _ = run_rc([sys.executable, "-c", "import sys; print('printed'); sys.exit(1)"])
_rc_neg, _ = run_rc([sys.executable, "-c", "print('printed')"])
if not (_rc_pos == 1 and _rc_neg == 0):
    sys.exit("facts.py: the exit-code sensor failed its probe pair (positive rc %s, negative rc %s); refusing to emit"
             % (_rc_pos, _rc_neg))
_own = open(__file__, encoding="utf-8").read()
_bad_exit_keys = re.findall(r'"(exit0|probeExit0)":\s*(?:bool\()?_?f?ok\b', _own)


def _pair_file_stem(probe_cmd):
    """The pair file's basename from a verify receipt's probe `command` (`--probe-from PATH: side ; side`), or None.

    issue582 - the first form was greedy (`.*[/\\\\]`) and, on sprint 732's receipt, ran past `pair732.txt:` to the last
    slash of a SIDE whose own path carried `C:`, reading `control_structure ; python C` as the stem. Non-greedy, and a
    stem is one token: no space, `;` or `:`. The D0388 pair below runs before any fact and refuses the emission."""
    m = re.search(r"--probe-from (?:.*?[/\\])?([^/\\:\s;]+):", probe_cmd or "")
    return m.group(1) if m and re.fullmatch(r"[\w.-]+", m.group(1)) else None


_stem_pos = _pair_file_stem("--probe-from C:/t/scratch/pair732.txt: python scripts/module_home.py a b ; python C:/t/scratch/neg732.py")
_stem_neg = _pair_file_stem("--probe a_probe_file_names_the_pair,a_probe_file_that_is_not_a_pair_is_refused")
if not (_stem_pos == "pair732.txt" and _stem_neg is None):
    sys.exit("facts.py: the pair-file stem sensor failed its probe pair (positive %r, negative %r); refusing to emit"
             % (_stem_pos, _stem_neg))
if _bad_exit_keys:
    sys.exit("facts.py: %d fact key(s) named for an exit derive from run()'s ok flag (%s); use run_rc"
             % (len(_bad_exit_keys), ", ".join(_bad_exit_keys)))


def as_json(text):
    try:
        return json.loads(text)
    except Exception:
        return None


def read(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return fh.read()
    except Exception:
        return None


def iso_min(a, b):
    """Minutes between two GitHub ISO timestamps."""
    fmt = "%Y-%m-%dT%H:%M:%SZ"
    return (datetime.strptime(b, fmt) - datetime.strptime(a, fmt)).total_seconds() / 60.0


# ---------------------------------------------------------------- tree identity

ok, out = run(["git", "rev-parse", "--short", "HEAD"])
TREE = out.strip() if ok else None

# ================================================================ 1. CLI SURFACE
# .engine/cli/commands.sysml is the authored CLI surface (D0271). One `part cli... : CliCommand`
# per command; a command whose `invocation` begins "show " is a lens rather than a top-level verb.

CLI_PATH = os.path.join(REPO, ".engine", "cli", "commands.sysml")
cli_src = read(CLI_PATH)
CLI_HOW = "parse .engine/cli/commands.sysml: "

if cli_src is None:
    for n in ("cliRecords", "cliShowLenses", "cliTopLevel", "cliDeprecated", "cliLive", "cliReadOnly",
              "cliLiveTopLevel", "cliLiveTopLevelReadOnly"):
        fact(n, None, "commands", CLI_HOW + "file not readable at %s" % CLI_PATH)
else:
    # one record per `part <name> : CliCommand { ... }`, body captured to its closing brace
    records = re.findall(r"part\s+cli\w*\s*:\s*CliCommand\s*\{(.*?)\}\s*$",
                         cli_src, re.MULTILINE | re.DOTALL)
    if not records:                                   # single-line form (the shape in this tree)
        records = [m.group(1) for m in re.finditer(r"part\s+cli\w*\s*:\s*CliCommand\s*\{([^\n]*)",
                                                   cli_src)]
    lenses = [b for b in records if re.search(r'invocation\s*=\s*"show\s', b)]
    deprecated = [b for b in records if "CliStability::deprecated" in b]
    reads = [b for b in records if "CliEffect::reads" in b]

    fact("cliRecords", len(records), "CliCommand records",
         CLI_HOW + "count of `part cli... : CliCommand` blocks")
    fact("cliShowLenses", len(lenses), "show lenses",
         CLI_HOW + 'records whose `invocation` begins "show " (D0271: a show invocation makes it a lens). '
                   'CROSS-CHECK: `keel show <bad-name>` prints a "Lenses:" hint listing 35 - it omits '
                   '`priority` and `control-structure`, both of which DO dispatch; the authored facts (and '
                   '`keel --help`, which renders from them) are the authority, so 37 is the number to publish')
    fact("cliTopLevel", len(records) - len(lenses), "top-level commands",
         CLI_HOW + "records minus the show-lens records")
    fact("cliDeprecated", len(deprecated), "commands",
         CLI_HOW + "records carrying CliStability::deprecated")
    fact("cliLive", len(records) - len(deprecated), "commands",
         CLI_HOW + "records minus the CliStability::deprecated ones")
    fact("cliReadOnly", len(reads), "commands",
         CLI_HOW + "records carrying CliEffect::reads (writes/both/tooling excluded)")
    # SAME-SCOPE pair for the page: both drawn from the top-level, non-deprecated population, so
    # they can be compared in one sentence. Mixing scopes published "72 of 69" (issue384).
    live_top = [b for b in records if b not in lenses and "CliStability::deprecated" not in b]
    fact("cliLiveTopLevel", len(live_top), "live top-level commands",
         CLI_HOW + "records that are neither a show lens nor deprecated - the names a reader meets")
    fact("cliLiveTopLevelReadOnly", len([b for b in live_top if "CliEffect::reads" in b]),
         "of those, read-only",
         CLI_HOW + "of the live top-level records, those carrying CliEffect::reads. SCOPE MATTERS: "
                   "cliReadOnly counts the whole surface including the show lenses, so the two must "
                   "never appear in one sentence")

    # THE MEMBERS, not only the counts (D0406): every live top-level verb by family with its effect,
    # so a page that proposes folding a family can name what folds and compute what remains. The
    # family-to-router mapping is the Decision's text and stays in the builder, quoted; the members
    # are facts. `name` and `family` are the authored fields of the CliCommand record.
    fam_verbs = {}
    for b in live_top:
        nm = re.search(r'name\s*=\s*"([^"]+)"', b)
        fm = re.search(r'family\s*=\s*"([^"]+)"', b)
        ef = re.search(r"CliEffect::(\w+)", b)
        if not (nm and fm and ef):
            continue
        fam_verbs.setdefault(fm.group(1), []).append({"name": nm.group(1), "effect": ef.group(1)})
    for fm in fam_verbs:
        fam_verbs[fm].sort(key=lambda r: r["name"])
    fact("cliLiveTopLevelByFamily",
         fam_verbs if sum(len(v) for v in fam_verbs.values()) == len(live_top) else None,
         "live top-level verbs by family, each with its effect",
         CLI_HOW + "the live top-level records grouped by their `family` field, each row the record's `name` and "
                   "CliEffect; the groups sum to cliLiveTopLevel or the fact is null (a record with no name, family "
                   "or effect would otherwise vanish from a members table without a trace)")

# ================================================================ 2. GATING CALL SITES
# Mentions of a gating verb invoked through the binary, across git-TRACKED files.
# .tracking/ is excluded from the headline: it is recorded history and must never be rewritten,
# so a call site there is not a maintenance surface. Its count is reported separately.

# D0452: the gating family is three routers; `keel gate validate` is ONE site, counted at `gate`.
GATING_VERBS = ["gate", "audit", "suite"]
GATING_RE = re.compile(r"(?<![\w.-])(?:keel\.exe|keelw|keel)[ \t]+(?:" +
                       "|".join(GATING_VERBS) + r")(?![\w-])")
ESCAPE_RE = re.compile(r"\\[nrt]")   # a literal \n in a source string is a line break, not a letter
# the naive form a reviewer would reach for first: no word boundaries at either end.
NAIVE_RE = re.compile(r"(?:keel\.exe|keelw|keel)[ \t]+(?:" + "|".join(GATING_VERBS) + r")")

GATING_HOW = ("git ls-files, then for each tracked TEXT file count regex "
              r"`(?<![\w.-])(keel\.exe|keelw|keel)[ \t]+<verb>(?![\w-])` over the gating verbs "
              "(" + ", ".join(sorted(GATING_VERBS)) + "); since D0452 the family is routed, so "
              "`keel gate validate` and `keel audit history` each count once, at the router. "
              "Counts OCCURRENCES, not lines - a line with two invocations is two "
              r"call sites. Literal \n/\r/\t escapes are normalised to a space first, so a call "
              r"site embedded in a source string (`\nkeel gate validate` in view/control_structure.rs) "
              "is counted. ")

ok, out = run(["git", "ls-files"])
if not ok:
    for n in ("gatingCallSites", "gatingCallFiles", "gatingCallSitesHistory"):
        fact(n, None, "call sites", GATING_HOW + "`git ls-files` failed: " + out)
else:
    tracked = [p for p in out.splitlines() if p.strip()]
    live_sites = live_files = hist_sites = 0
    naive_live = naive_hist = 0
    for rel in tracked:
        text = read(os.path.join(REPO, rel.replace("/", os.sep)))
        if text is None or "\0" in text[:4096]:
            continue
        flat = ESCAPE_RE.sub(" ", text)
        n = len(GATING_RE.findall(flat))
        nn = len(NAIVE_RE.findall(flat))
        if rel.startswith(".tracking/"):
            hist_sites += n
            naive_hist += nn
        else:
            live_sites += n
            naive_live += nn
            if n:
                live_files += 1
    # both numbers, and which to trust - a reviewer's obvious grep disagrees, on purpose
    BOTH = ("BOTH NUMBERS: the same sweep WITHOUT the trailing word-boundary reports %d live and %d "
            "history. The %d/%d extra are English prose, not invocations - overwhelmingly 'keel "
            "gates every project the commit touches'. TRUST the bounded number published here; the "
            "naive one over-counts. "
            % (naive_live, naive_hist, naive_live - live_sites, naive_hist - hist_sites))
    fact("gatingCallSites", live_sites, "invocations in live (non-.tracking) tracked files",
         GATING_HOW + BOTH + "EXCLUDES .tracking/ (recorded history, never rewritten).")
    fact("gatingCallFiles", live_files, "tracked files carrying at least one",
         GATING_HOW + "distinct non-.tracking tracked files with >=1 bounded match.")
    fact("gatingCallSitesHistory", hist_sites, "invocations inside .tracking/ (history)",
         GATING_HOW + BOTH + "the EXCLUDED half, reported so the exclusion is visible rather than "
                             "silent. These are past sprint records and test-result evidence; "
                             "rewriting them would orphan evidence (D0129).")

# ================================================================ 3. DECISIONS (from the files)
# .engine/decisions/NNNN-slug.sysml, one Decision part per file. Status and createdAt are read
# from the DECISION part's own `:>> ...` assignments, not from prose that mentions them.

DEC_DIR = os.path.join(REPO, ".engine", "decisions")
DEC_HOW = "parse .engine/decisions/*.sysml: "
dec_files = sorted(f for f in os.listdir(DEC_DIR)) if os.path.isdir(DEC_DIR) else []
dec_files = [f for f in dec_files if f.endswith(".sysml")]

decisions = []          # {slug, status, createdAt, marked, consequences}
for fn in dec_files:
    text = read(os.path.join(DEC_DIR, fn)) or ""
    # the Decision part and everything after it (the acceptance verification trails it)
    m = re.search(r"(#(?:ProspectiveChange|SafetyChange)\s+)?part\s+(d\d+)\s*:\s*Decision\s*\{",
                  text)
    if not m:
        continue
    body = text[m.end():]
    st = re.search(r":>>\s*status\s*=\s*DecisionStatus::(\w+)\s*;", body)
    ca = re.search(r':>>\s*createdAt\s*=\s*"(\d{4}-\d{2}-\d{2})"', body)
    cons = re.search(r':>>\s*consequences\s*=\s*"(.*?)"\s*;', body, re.DOTALL)
    decisions.append({
        "file": fn,
        "slug": m.group(2),
        "marked": bool(m.group(1)),
        "status": st.group(1) if st else None,
        "createdAt": ca.group(1) if ca else None,
        "consequences": cons.group(1) if cons else "",
    })

# Standing is the status MINUS the edge (D0398): a `#Supersede` edge retires its target whole and the
# target keeps the status it had, so a proposed-or-accepted Decision that is an edge target is neither
# pending nor in force. `#SupersedeClause` reverses one clause and leaves its target standing. Read from
# every non-comment line under .tracking/ and .engine/, the way section 11 reads DerivedFrom.
SUP_WHOLE_RE = re.compile(r"#Supersede\s+dependency\s+from\s+(\w+)\s+to\s+([\w, ]+);")
SUP_CLAUSE_RE = re.compile(r"#SupersedeClause\s+dependency\s+from\s+(\w+)\s+to\s+([\w, ]+);")
_sup_files = []
for _base in (".tracking", ".engine"):
    for _dp, _dn, _fns in os.walk(os.path.join(REPO, _base)):
        _sup_files.extend(os.path.join(_dp, f) for f in _fns if f.endswith(".sysml"))
retired = set()
sup_whole_edges = 0
sup_clause_edges = 0
clause_targets = set()
for _p in _sup_files:
    for _line in (read(_p) or "").splitlines():
        if _line.lstrip().startswith("//"):
            continue
        for _m in SUP_WHOLE_RE.finditer(_line):
            sup_whole_edges += 1
            retired.update(x.strip() for x in _m.group(2).split(","))
        for _m in SUP_CLAUSE_RE.finditer(_line):
            sup_clause_edges += 1
            clause_targets.update(x.strip() for x in _m.group(2).split(","))
_slugs = {d["slug"] for d in decisions}
retired_decisions = sorted(s for s in _slugs if s in retired)
clause_targets = {t for t in clause_targets if t in _slugs}
sup_clause_edges = len(clause_targets)
STANDING = (" and NOT the target of a `#Supersede` edge (D0398: retirement is the edge, the status is "
            "what the record read when retired; %d Decisions are retired this way)" % len(retired_decisions))

accepted = [d for d in decisions if d["status"] == "accepted" and d["slug"] not in retired]
proposed = [d for d in decisions if d["status"] == "proposed" and d["slug"] not in retired]

fact("decisionsTotal", len(decisions), "Decision records",
     DEC_HOW + "one `part dNNNN : Decision` per file; counted by file")
fact("decisionsAccepted", len(accepted), "Decisions in force",
     DEC_HOW + "`:>> status = DecisionStatus::accepted;` inside the Decision part (the `:>>` "
               "assignment only - a bare `DecisionStatus::` in prose is not counted, which is why "
               "this is lower than a naive grep)" + STANDING)
fact("decisionsProposed", len(proposed), "Decisions",
     DEC_HOW + "`:>> status = DecisionStatus::proposed;` inside the Decision part" + STANDING)
fact("decisionsRetired", len(retired_decisions), "Decisions retired by a #Supersede edge",
     "distinct Decision targets of a non-comment `#Supersede dependency from X to Y;` line under "
     ".tracking/ or .engine/: " + (", ".join(retired_decisions) or "none") + ".")
fact("supersedeEdges", sup_whole_edges, "#Supersede edges in the model (retire the target whole)",
     "non-comment lines matching `#Supersede dependency from X to Y;` - Y may be a comma list; "
     "targets are Decisions, Needs and requirements alike.")
fact("supersedeClauseEdges", sup_clause_edges, "#SupersedeClause edges (reverse one clause, target stands)",
     "non-comment lines matching `#SupersedeClause dependency from X to Y;` whose target is a declared "
     "Decision (a Decision's text quoting the grammar is not an edge): targets "
     + (", ".join(sorted(clause_targets)) or "none") + ".")

# --- the 7-day window: [today-6, today], i.e. seven calendar days including today
WIN_START = TODAY - timedelta(days=6)
recent = [d for d in decisions if d["createdAt"] and
          WIN_START.isoformat() <= d["createdAt"] <= TODAY.isoformat()]
undated = [d for d in decisions if not d["createdAt"]]
WIN_HOW = (DEC_HOW + "the Decision part's own `:>> createdAt` in [%s, %s] - seven calendar days "
           "including today. %d of %d Decisions carry NO createdAt (the earliest records predate "
           "the field) and can never fall in the window; they are all far older than 7 days, so "
           "the count is unaffected. "
           % (WIN_START.isoformat(), TODAY.isoformat(), len(undated), len(decisions)))

fact("decisions7d", len(recent), "Decisions recorded in the last 7 days", WIN_HOW)
fact("decisionsMarked7d", sum(1 for d in recent if d["marked"]),
     "of those carrying #ProspectiveChange or #SafetyChange",
     WIN_HOW + "of those, the ones whose Decision part is prefixed "
               "`#ProspectiveChange` or `#SafetyChange` (D0070) - i.e. process/safety change, which "
               "under D0337 falls outside standing consent and waits for the human.")
fact("decisionsPerDay7d", round(len(recent) / 7.0, 1), "Decisions per day (7-day mean)",
     WIN_HOW + "divided by 7.")

# --- the consent scope since D0337: who has been accepting, and how long each waited
# D0337 (2026-09-05) scoped standing consent to the processes it was promulgated under, so a
# process/safety-change Decision waits for the human. Read from each file's acceptance records:
# AUTO if the acceptance Test's procedureText carries the AUTO-ACCEPTED token, HUMAN if a passing
# AcceptR result exists without it, else still proposed. Wait = judgedAt of the first AcceptR minus
# the Decision's createdAt, in days.
SCOPE_FROM = "d0337"
scope_auto, scope_human, scope_open, scope_waits = [], [], [], []
for d in decisions:
    if d["slug"] < SCOPE_FROM:
        continue
    text = read(os.path.join(DEC_DIR, d["file"])) or ""
    is_auto = "AUTO-ACCEPTED" in text
    r1 = re.search(r'part\s+' + d["slug"] + r'AcceptR1\s*:\s*TestResult\s*\{.*?judgedAt\s*=\s*"(\d{4}-\d{2}-\d{2})"', text, re.DOTALL)
    if d["slug"] in retired or d["status"] == "rejected":
        continue                       # retired by a #Supersede edge (D0398) or rejected (D0122): decided, not open
    if d["status"] != "accepted" or not r1:
        scope_open.append(d["slug"])
    elif is_auto:
        scope_auto.append(d["slug"])
    else:
        scope_human.append(d["slug"])
        if d["createdAt"]:
            scope_waits.append((date.fromisoformat(r1.group(1)) - date.fromisoformat(d["createdAt"])).days)
SCOPE_HOW = (DEC_HOW + "Decisions with slug >= %s (D0337, the consent-scope rule, 2026-09-05); AUTO if the "
             "file carries the AUTO-ACCEPTED token, HUMAN if `<slug>AcceptR1` exists without it, OPEN otherwise. "
             % SCOPE_FROM)
fact("scopeDecisionsHumanAccepted", len(scope_human), "Decisions accepted on the human's own word since the consent-scope rule", SCOPE_HOW)
fact("scopeDecisionsAutoAccepted", len(scope_auto), "Decisions auto-accepted under standing consent since the consent-scope rule", SCOPE_HOW)
fact("scopeDecisionsOpen", len(scope_open), "Decisions since the consent-scope rule still proposed", SCOPE_HOW)
fact("scopeDaysSinceRule", (TODAY - date(2026, 9, 5)).days, "calendar days since D0337 was recorded",
     "today minus 2026-09-05, D0337's own createdAt.")
fact("scopeHumanWaitMaxDays", max(scope_waits) if scope_waits else None, "days",
     SCOPE_HOW + "for the HUMAN set, AcceptR1.judgedAt minus the Decision's createdAt; the maximum.")
fact("scopeHumanWaitMeanDays", round(sum(scope_waits) / len(scope_waits), 1) if scope_waits else None, "days",
     SCOPE_HOW + "for the HUMAN set, AcceptR1.judgedAt minus the Decision's createdAt; the mean.")

# --- the consent-defect census (D0375). Two HAND-CLASSIFIED lists, fixed here so the number on the
# page is the length of a list anyone can re-read, not a judgment re-made each run. Classified
# 2026-09-08 from each Issue's title and description; the census script that surfaced the candidates
# is textual (keyword over titles) and its class counts are NOT emitted as facts for that reason.
CONSENT_TOO_WIDE = ["issue256", "issue373", "issue371", "issue341", "issue342", "issue347",
                    "issue298", "issue238", "issue254"]
CONSENT_REFUSED_REAL = ["issue287", "issue359", "issue397", "issue223", "issue234", "issue383",
                       "issue396", "issue217"]
CENSUS_HOW = ("hand-classified 2026-09-08 from .tracking/issues-*.sysml titles and descriptions; the ids are "
              "listed in this script (CONSENT_TOO_WIDE / CONSENT_REFUSED_REAL) so the count is re-readable. ")
fact("issuesConsentTooWide", len(CONSENT_TOO_WIDE), "Issues where an acceptance was recorded or kept that was not what was given",
     CENSUS_HOW + ", ".join(CONSENT_TOO_WIDE))
fact("issuesConsentRefusedReal", len(CONSENT_REFUSED_REAL), "Issues where a control refused or mis-framed a real acceptance",
     CENSUS_HOW + ", ".join(CONSENT_REFUSED_REAL))
_iss_total = 0
_iss_dir = os.path.join(REPO, ".tracking")
for fn in os.listdir(_iss_dir) if os.path.isdir(_iss_dir) else []:
    if fn.startswith("issues-") and fn.endswith(".sysml"):
        _iss_total += len(re.findall(r"part\s+issue\d+\s*:\s*Issue\s*\{", read(os.path.join(_iss_dir, fn)) or ""))
fact("issuesTotal", _iss_total, "Issue records", "count of `part issueNNN : Issue {` across .tracking/issues-*.sysml")

# --- proposed Decisions that say, in their own consequences, that the change already ships
SHIPPED_PHRASES = ["the code ships", "already ship", "ships now", "is built",
                   # WIDENED 2026-09-07, by the rule this fact's own `how` states: the list is fixed
                   # by adding phrases, never by re-reading prose into a different answer. It read 0
                   # against a queue where three of five had shipped - d0361 says "IMPLEMENTED
                   # 2026-09-06", d0366 says "already in the tree", d0363 describes the move in the
                   # present tense. A false ZERO here is the issue383 defect exactly: a brief that
                   # frames already-shipped work as an open choice.
                   "already in the tree", "implemented", "in the commit that carries",
                   "landed", "is in the tree"]
already = [d for d in proposed
           if any(p in d["consequences"].lower() for p in SHIPPED_PHRASES)]
SHIP_HOW = (DEC_HOW + "of the PROPOSED Decisions, those whose own `consequences` string contains one "
            "of the literal phrases " + repr(SHIPPED_PHRASES) + " (case-insensitive). This is a "
            "PHRASE RULE over authored prose, not a reading, and it is imperfect in two named ways - "
            "publish the number only with this caveat. FALSE POSITIVE: d0355 matches on 'once their "
            "state chip says the change already ships', which is about the OTHER pending items, not "
            "about itself. FALSE NEGATIVE: d0345 says 'the retirement itself is pushed now', which "
            "means the same thing and is outside the phrase list. The two cancel in the COUNT (5 "
            "either way) but not in the LIST. Fix by widening the phrase list, never by re-reading "
            "prose into a different answer.")
fact("pendingAlreadyShipped", len(already), "proposed Decisions whose code already ships", SHIP_HOW)
fact("pendingAlreadyShippedList", ", ".join(d["slug"] for d in already) or None,
     "decision slugs", SHIP_HOW + " Slugs listed in file order.")

# --- proposed Decisions whose chartered sprint is DONE - a structural reading, not a phrase rule
# A sprint record names the Decision that chartered it (`#CharteredBy dependency from <story> to
# dNNNN;`, D0068) and records its story's DoD verdict as a TestResult. Both are edges/results the
# guards already validate, so this fact cannot be moved by rewording a consequences string.
DELIV_HOW = ("walk .tracking/delivery/*.sysml; a proposed Decision is DELIVERED when one file holds "
             "`#CharteredBy dependency from <x> to <its id>;` and the same file holds a "
             "`story<Slug>DoDR<n> : TestResult` whose `outcome = VerdictKind::pass`. Reads typed "
             "edges and recorded verdicts, never prose; a Decision built OUTSIDE a sprint is not "
             "seen here, which is why the phrase rule above is kept beside it.")
_charter_re = re.compile(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+(d\d{4})\s*;")
_dod_pass_re = re.compile(r"part\s+story\w*DoDR\d+\s*:\s*TestResult\s*\{[^}]*VerdictKind::pass")
delivered_ids = set()
_deliv_dir = os.path.join(REPO, ".tracking", "delivery")
if os.path.isdir(_deliv_dir):
    for _fn in sorted(os.listdir(_deliv_dir)):
        if not _fn.endswith(".sysml"):
            continue
        _txt = read(os.path.join(_deliv_dir, _fn))
        if _dod_pass_re.search(_txt):
            delivered_ids.update(_charter_re.findall(_txt))
delivered = [d for d in proposed if d["slug"] in delivered_ids]
fact("pendingDelivered", len(delivered),
     "proposed Decisions whose chartered sprint records a passing DoD", DELIV_HOW)
fact("pendingDeliveredList", ", ".join(d["slug"] for d in delivered) or None,
     "decision slugs", DELIV_HOW + " Slugs listed in file order.")
in_tree_slugs = [d["slug"] for d in proposed if d["slug"] in delivered_ids or d in already]
IN_TREE_HOW = ("Union of pendingDeliveredList (structural: a finished sprint charters the Decision and its DoD "
               "passed) and pendingAlreadyShippedList (the phrase rule, for a Decision built outside any sprint). "
               "This is the ONE number the brief states as 'already in the tree'; the page builder reads it and "
               "refuses if its own cross-check of the two lists disagrees.")
fact("pendingInTree", len(in_tree_slugs), "proposed Decisions whose change is already in the tree", IN_TREE_HOW)
fact("pendingInTreeList", ", ".join(in_tree_slugs) or None, "decision slugs", IN_TREE_HOW + " Slugs listed in file order.")

# --- the pending set itself, member by member (D0406: a page that asks for a word on a SET names its members).
# name = the file's slug after the number (the Decision's own title token, not a record id); fork = the
# decision text opens with an OPTION marker (D0322: a weighed alternative is a fork; a ratification is not).
_name_re = re.compile(r"^\d{4}-(.+)\.sysml$")
pending_members = []
for d in proposed:
    _nm = _name_re.match(d["file"])
    _txt = read(os.path.join(DEC_DIR, d["file"])) or ""
    _dec = re.search(r':>>\s*decision\s*=\s*"(.*?)"\s*;', _txt, re.DOTALL)
    pending_members.append({
        "slug": d["slug"],
        "name": _nm.group(1) if _nm else d["file"],
        "fork": bool(_dec and _dec.group(1).lstrip().startswith("OPTION")),
        "inTree": d["slug"] in in_tree_slugs,
    })
PEND_HOW = (DEC_HOW + "every PROPOSED, non-retired Decision (the same set decisionsProposed counts), one row each: "
            "name = the file name between the number and .sysml; fork = the `decision` field begins with the "
            "literal OPTION (D0322's marker); inTree = the slug is in pendingInTreeList. File order.")
fact("pendingMembers", pending_members or None, "one row per pending Decision", PEND_HOW)
fact("pendingForks", sum(1 for p in pending_members if p["fork"]),
     "pending Decisions whose text opens a weighed fork", PEND_HOW)

# ================================================================ 3b. performance (D0367 asks)
# The baseline is whatever D0367's context RECORDS - read by regex from the Decision file, so a
# retyped number here cannot drift from the record. The current numbers are timed now, against the
# binary this script already shells out to, so the page states what THIS tree costs today.
_d0367 = ""
for _fn in os.listdir(os.path.join(REPO, ".engine", "decisions")):
    if _fn.startswith("0367-"):
        _d0367 = read(os.path.join(REPO, ".engine", "decisions", _fn))
_base = re.search(r"`keel orient` costs ([0-9.]+) s and `keel gate guard` ([0-9.]+) s", _d0367)
BASE_HOW = ("regex `keel orient` costs N s and `keel gate guard` N s over the context string of "
            ".engine/decisions/0367-*.sysml - the spike's own measured baseline (2026-09-07, this host), "
            "quoted from the record, never retyped.")
fact("perfBaselineOrientSec", float(_base.group(1)) if _base else None, "seconds", BASE_HOW)
fact("perfBaselineGuardSec", float(_base.group(2)) if _base else None, "seconds", BASE_HOW)


def _timed_ms(cmd, runs, stdin_text=None, warm=0):
    """Median wall ms of `runs` runs after `warm` untimed warm-up runs; None if any run fails."""
    samples = []
    for i in range(warm + runs):
        t0 = time.perf_counter()
        try:
            p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, timeout=300,
                               input=stdin_text, encoding="utf-8", errors="replace")
        except Exception:
            return None
        ms = int((time.perf_counter() - t0) * 1000)
        if i >= warm:
            if p.returncode != 0 and cmd[1] != "hook":
                return None
            samples.append(ms)
    samples.sort()
    return samples[len(samples) // 2]


_env_note = " (KEEL_NO_RECEIPT unset; a receipt from an earlier green run is honoured when its key is equal)"
fact("perfTurnBoundaryIdleMs",
     _timed_ms([KEEL, "hook", "stop"], runs=3, warm=1,
               stdin_text='{"session_id":"exec-brief","stop_hook_active":false}'),
     "milliseconds",
     "wall time of `echo {hook json} | keel hook stop` in this tree, median of 3 after one untimed "
     "warm-up run (the warm-up writes the receipt when the build changed)" + _env_note)
fact("perfGuardFullMs", _timed_ms([KEEL, "gate", "guard", "--no-receipt"], runs=2),
     "milliseconds",
     "wall time of `keel gate guard --no-receipt` in this tree, median of 2 - every enforced guard runs, "
     "the receipt is neither read nor written")
fact("perfOrientMs", _timed_ms([KEEL, "show", "orient", "."], runs=3),
     "milliseconds", "wall time of `keel show orient .` in this tree, median of 3")

# ================================================================ 4. keel show orient (one call)
# `keel show orient .` already emits JSON on stdout - there is no `--json` flag (it errors), so the
# plain invocation IS the JSON lens.

ok, out = run([KEEL, "show", "orient", "."], timeout=90)
orient = as_json(out) if ok else None
O_HOW = ("`./target/release/keel.exe show orient .` (D0450: orientation folded under show) - it prints JSON "
         "with no flag, so the bare command is the JSON lens. ")

if orient is None:
    reason = O_HOW + ("command failed: " + (out if not ok else "stdout was not JSON"))
    for n in ("pendingAcceptances", "suspectElements", "openIssues", "readyItems"):
        fact(n, None, "items", reason)
else:
    _pending = len(orient.get("pendingAcceptances", []))
    fact("pendingAcceptances", _pending,
         "proposed Decisions awaiting a human's acceptance",
         O_HOW + "len(.pendingAcceptances). Cross-checked against the file-derived decisionsProposed.")
    if _pending != len(proposed):
        # The file reader has drifted from the engine's standing rule (the D0398 shape: this script
        # counted two retired-while-proposed Decisions as pending for a day). Publish no number.
        fact("decisionsProposed", None, "Decisions",
             "REFUSED: the file-derived count (%d: %s) disagrees with orient's pendingAcceptances (%d) - "
             "this script's reading of a Decision's standing is stale against the engine's; fix facts.py "
             "before a brief carries either number." % (len(proposed), ", ".join(d["slug"] for d in proposed), _pending))
    fact("suspectElements", len(orient.get("suspect", [])),
         "done items whose evidence drifted from the tree",
         O_HOW + "len(.suspect) - identical to `keel show suspect .` .suspect, verified by hand; "
                 "orient is used so the whole set costs one process start. NOTE `show suspect` "
                 "also reports critique_suspect (a different, larger set) - this is NOT that.")
    fact("openIssues", len(orient.get("open_issues", [])), "open Issues",
         O_HOW + "len(.open_issues) - equals `keel show open-issues .` .open, verified by hand.")
    fact("readyItems", len(orient.get("ready", [])), "items on the ready frontier",
         O_HOW + "len(.ready) - equals the line count of `keel show whats-next .` and the 'N ready' in "
                 "`keel show status .`, verified by hand.")

    # the triggered indicator lives in orient's burndown AND in `show indicators .`
    trig = None
    for t in (orient.get("burndown") or {}).get("triggers", []):
        if "ungrounded" in (t.get("indicator") or "").lower():
            trig = t
    bd = orient.get("burndown") or {}
    if trig is not None or "ungrounded_ratio_pct" in bd:
        val = float(trig["latest"]) if trig and trig.get("latest") else bd.get("ungrounded_ratio_pct")
        fact("ungroundedRatio", val, "% of Decision-chartered Delivery Stories reaching no Need",
             O_HOW + ".burndown.triggers[ungroundedRatioIndicator].latest (same number as "
                     ".burndown.ungrounded_ratio_pct, and as `keel show indicators .` "
                     "-> .triggered[].latest). D0333: an indicator surfaces work, it never gates.")
        fact("ungroundedRatioThreshold",
             (trig or {}).get("threshold"), "declared trigger threshold",
             O_HOW + ".burndown.triggers[].threshold - declared in "
                     ".engine/contracts/indicator-triggers.toml (D0333), not computed.")
    else:
        for n in ("ungroundedRatio", "ungroundedRatioThreshold"):
            fact(n, None, "%", O_HOW + "no ungrounded indicator present in .burndown.triggers")

# ================================================================ 5. keel show coverage
ok, out = run([KEEL, "show", "coverage", "."], timeout=90)
cov = as_json(out) if ok else None
C_HOW = "`./target/release/keel.exe show coverage .` -> .summary[] "

if cov is None:
    reason = C_HOW + ("failed: " + (out if not ok else "stdout was not JSON"))
    for n in ("needsTotal", "needsVerified", "needsUncovered",
              "reqsTotal", "reqsVerified", "reqsUncovered"):
        fact(n, None, "items", reason)
else:
    rows = {r.get("type"): r for r in cov.get("summary", [])}
    for prefix, typ in (("needs", "Need"), ("reqs", "SystemRequirement")):
        r = rows.get(typ) or {}
        label = "Needs" if typ == "Need" else "SystemRequirements"
        fact(prefix + "Total", r.get("total"), label,
             C_HOW + 'row type="%s" .total' % typ)
        fact(prefix + "Verified", r.get("verified"), label + " with reproducible verify evidence",
             C_HOW + 'row type="%s" .verified (D0082 top tier: reproducible verify-edge evidence; '
                     'Needs count transitively via a verified requirement)' % typ)
        fact(prefix + "Uncovered", r.get("uncovered"), label + " with no coverage at all",
             C_HOW + 'row type="%s" .uncovered (neither verified, attested, nor addressed)' % typ)

# ================================================================ 6. keel show verification
ok, out = run([KEEL, "show", "verification", "."], timeout=90)
V_HOW = ("`./target/release/keel.exe show verification .` - TEXT, not JSON. Parsed from the three "
         "labelled lines 'exercised but NEVER examined', 'examined but NEVER exercised', 'neither'. "
         "D0083: EXAMINED (a judgment was formed about the requirement) and EXERCISED (the system "
         "was run against it) are two dimensions - never publish their union as one 'verified %'. ")


def vgrab(text, label):
    m = re.search(re.escape(label) + r"\s*:?\s*(\d+)", text)
    return int(m.group(1)) if m else None


if not ok:
    for n in ("reqExercisedNeverExamined", "reqExaminedNeverExercised", "reqNeither"):
        fact(n, None, "SystemRequirements", V_HOW + "command failed: " + out)
else:
    fact("reqExercisedNeverExamined", vgrab(out, "exercised but NEVER examined"),
         "live SystemRequirements run against but never adversarially read",
         V_HOW + "line 'exercised but NEVER examined'.")
    fact("reqExaminedNeverExercised", vgrab(out, "examined but NEVER exercised"),
         "live SystemRequirements judged but never run against",
         V_HOW + "line 'examined but NEVER exercised'.")
    fact("reqNeither", vgrab(out, "neither"),
         "live SystemRequirements neither examined nor exercised",
         V_HOW + "line 'neither'.")

# ================================================================ 7. keel show status (guards)
ok, out = run([KEEL, "show", "status", "."], timeout=120)
S_HOW = "`./target/release/keel.exe show status .` (D0450) - TEXT; parsed from its `model` section. "
if not ok:
    for n in ("guardWarnings", "guardViolations"):
        fact(n, None, "guard findings", S_HOW + "command failed: " + out)
else:
    mv = re.search(r"(\d+)\s+violations?,\s+(\d+)\s+warning", out)
    fact("guardViolations", int(mv.group(1)) if mv else None,
         "guard violations (blocking)",
         S_HOW + "'N violations, M warning(s)'. A violation blocks the gate; this repo's contract is "
                 "that it is zero.")
    fact("guardWarnings", int(mv.group(2)) if mv else None,
         "guard warnings (non-blocking, unread until someone reads them)",
         S_HOW + "'N violations, M warning(s)'. Same numbers `keel gate guard .` prints in its ALL PASS "
                 "line; status is used because one process start yields both.")
    mg = re.search(r"(\d+)\s+guards", out)
    fact("guardsEnforced", int(mg.group(1)) if mg else None, "enforced forward guards",
         S_HOW + "'N guards' in the model line.")

# ---------------------------------------------------------------- the control (issue389)
# This file measures "since the last release" and once TYPED the tag it measured against, so every
# such fact silently described a superseded release after a newer one shipped - with a `how` string
# that still read as authoritative because it named a real command. The rule is about SOURCE TEXT,
# which is why this check reads source text: nothing else can express "no version is typed here".
#
# It scans the WHOLE file, with no self-exclusion. The first version of this check scanned only what
# sat above it - and the release facts sit below, so it passed over exactly the defect it was written
# for. A deliberate probe caught that; the rule is now phrased without naming any version, so there
# is nothing to exclude and nothing to get wrong.
def _no_version_is_typed_here():
    src = open(__file__, encoding="utf-8").read()
    typed = sorted(set(re.findall(r"v\d+\.\d+\.\d+", src)))
    if typed:
        sys.exit(
            "facts.py names " + ", ".join(typed) + " literally. The release these facts measure against is "
            "DERIVED (`git describe --tags --abbrev=0 --match v*`); typing one makes every since-the-release "
            "fact describe that release forever, including after a newer one ships (issue389)."
        )


_no_version_is_typed_here()

# ================================================================ 8. release + git
# WHICH release: derived, never typed (issue389). Naming a version here makes every fact below
# describe that version forever, including after a newer one ships - and the `how` string still
# reads as authoritative because it names a real command.
ok, tag = run(["git", "describe", "--tags", "--abbrev=0", "--match", "v*"])
TAG = tag.strip() if ok and tag.strip() else None
fact("releaseTag", TAG, "the newest v* tag reachable from HEAD",
     "`git describe --tags --abbrev=0 --match v*` - the release these since-the-release facts are measured against."
     + ("" if TAG else " FAILED: " + str(tag)))

if TAG:
    ok, tagiso = run(["git", "log", "-1", "--format=%cI", TAG])
    tagiso = tagiso.strip() if ok else None
else:
    tagiso = None
G_HOW = f"`git log -1 --format=%cI {TAG}` " if TAG else "no v* tag found, so "
if not tagiso:
    fact("releaseTagDate", None, "ISO date", G_HOW + "produced no date: " + str(tagiso))
    tag_dt = None
else:
    tag_dt = datetime.fromisoformat(tagiso)
    fact("releaseTagDate", tagiso, f"ISO-8601 commit date of tag {TAG}",
         G_HOW + "- the commit date of the tagged commit, not the tag object's own date.")

if TAG:
    ok, out = run(["git", "rev-list", "--count", f"{TAG}..HEAD"])
    fact("commitsSinceRelease", int(out.strip()) if ok and out.strip().isdigit() else None,
         f"commits on this branch since {TAG}",
         f"`git rev-list --count {TAG}..HEAD`" + ("" if ok else " failed: " + out))
else:
    fact("commitsSinceRelease", None, "commits", "no v* tag to count from.")

if tag_dt:
    fact("daysSinceRelease", (NOW - tag_dt.astimezone(timezone.utc)).days,
         f"whole days since {TAG} was committed",
         G_HOW + "differenced against now (UTC), floored to whole days.")
else:
    fact("daysSinceRelease", None, "days", G_HOW + "no tag date, so nothing to difference.")

# ================================================================ 9. CI (gh - 2 calls, the slow part)
CI_CMD = ["gh", "run", "list", "--workflow=ci.yml", "--branch=main", "--limit", "200",
          "--json", "conclusion,createdAt,updatedAt"]
ok, out = run(CI_CMD, timeout=60)
runs = as_json(out) if ok else None
CI_HOW = "`" + " ".join(CI_CMD) + f"`, filtered to runs whose createdAt >= the {TAG} tag date. "

if runs is None or tag_dt is None:
    reason = CI_HOW + ("gh failed: " + str(out)[:200] if runs is None
                       else f"no {TAG} tag date to filter against")
    for n in ("ciRunsSinceRelease", "ciFailuresSinceRelease", "ciMeanMinutes"):
        fact(n, None, "runs", reason)
else:
    since = [r for r in runs
             if datetime.strptime(r["createdAt"], "%Y-%m-%dT%H:%M:%SZ")
             .replace(tzinfo=timezone.utc) >= tag_dt.astimezone(timezone.utc)]
    concluded = [r for r in since if r.get("conclusion")]
    fails = [r for r in concluded if r["conclusion"] != "success"]
    # is the 200-run window wide enough to reach back past the tag?
    oldest = min((r["createdAt"] for r in runs), default=None)
    saturated = (len(runs) >= 200 and oldest and
                 datetime.strptime(oldest, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
                 > tag_dt.astimezone(timezone.utc))
    window = ("The 200-run window reaches back to %s, which PREDATES the tag, so the count is "
              "complete." % oldest) if not saturated else \
             ("WARNING: the 200-run window's oldest run is %s, AFTER the tag - the window is "
              "saturated and these are LOWER BOUNDS." % oldest)
    fact("ciRunsSinceRelease", len(since), "ci.yml runs on main since " + str(TAG),
         CI_HOW + window, as_of=NOW.date().isoformat())
    fact("ciFailuresSinceRelease", len(fails),
         "of those, concluded non-success (failure/cancelled/timed_out)",
         CI_HOW + "counts concluded runs whose conclusion != 'success'; %d run(s) in the window "
                  "had no conclusion yet and are excluded from both this and the mean. %s"
                  % (len(since) - len(concluded), window), as_of=NOW.date().isoformat())
    mins = [iso_min(r["createdAt"], r["updatedAt"]) for r in concluded]
    fact("ciMeanMinutes", round(sum(mins) / len(mins), 1) if mins else None,
         f"mean wall minutes per concluded ci.yml run since {TAG}",
         CI_HOW + "mean of (updatedAt - createdAt) over the %d CONCLUDED runs. This is queue+run "
                  "time as GitHub records it, not billable compute." % len(concluded),
         as_of=NOW.date().isoformat())

REL_CMD = ["gh", "run", "list", "--workflow=release.yml", "--limit", "5",
           "--json", "conclusion,createdAt,updatedAt,status,displayTitle"]
ok, out = run(REL_CMD, timeout=60)
rels = as_json(out) if ok else None
R_HOW = "`" + " ".join(REL_CMD) + "` -> the most recent entry (gh returns newest first). "

if not rels:
    reason = R_HOW + ("gh failed: " + str(out)[:200] if rels is None else "no release runs returned")
    for n in ("releaseLastRunDate", "releaseLastRunConclusion", "releaseLastRunMinutes"):
        fact(n, None, "release run", reason)
else:
    r = rels[0]
    fact("releaseLastRunDate", r.get("createdAt"), "ISO-8601 start of the last release.yml run",
         R_HOW + "its .createdAt. Title: %r" % r.get("displayTitle"),
         as_of=NOW.date().isoformat())
    fact("releaseLastRunConclusion", r.get("conclusion") or r.get("status"),
         "GitHub conclusion of the last release.yml run",
         R_HOW + "its .conclusion (falling back to .status while a run is still in flight).",
         as_of=NOW.date().isoformat())
    fact("releaseLastRunMinutes",
         round(iso_min(r["createdAt"], r["updatedAt"]), 1) if r.get("conclusion") else None,
         "wall minutes of the last release.yml run",
         R_HOW + ("(updatedAt - createdAt)." if r.get("conclusion")
                  else "the run has not concluded, so its updatedAt is not an end time."),
         as_of=NOW.date().isoformat())

# ================================================================ 10. suite receipt (D0353)
RCP = os.path.join(REPO, ".keel", "metrics", "suite-receipt.toml")
rcp_src = read(RCP)
RC_HOW = ".keel/metrics/suite-receipt.toml (D0353, machine-local): "

if rcp_src is None:
    for n in ("suiteTests", "suiteFailed", "suiteHead", "suiteWallMinutes"):
        fact(n, None, "tests", RC_HOW + "no receipt on this machine - run `keel suite`.")
else:
    def rget(key):
        m = re.search(r"^\s*%s\s*=\s*\"?([^\"\n]+)\"?\s*$" % key, rcp_src, re.MULTILINE)
        return m.group(1).strip() if m else None

    passed, failed = rget("passed"), rget("failed")
    at, head, logrel = rget("at"), rget("head"), rget("log")
    running = rget("outcome") == "running"
    if running:
        # the stub `keel suite` writes at its START (D0387): a run in progress, or one that was killed -
        # either way the counts are not an answer, and the `how` says which file said so
        passed = failed = None
    RUNNING_HOW = (RC_HOW + "`outcome = \"running\"` - the stub keel suite writes before cargo starts (D0387): "
                  "a run in progress, or one that was killed at %s. No count until a run completes." % at)
    fact("suiteTests", int(passed) + int(failed) if passed and failed else None, "tests run",
         RUNNING_HOW if running else
         RC_HOW + "`passed` + `failed`. The receipt counts the whole `cargo test --release "
                  "--no-fail-fast` run, unit + integration + doc tests.")
    fact("suiteFailed", int(failed) if failed else None, "failing tests",
         RUNNING_HOW if running else RC_HOW + "`failed`.")
    fact("suiteHead", head, "short SHA the suite last ran against", RC_HOW + "`head`.")

    # wall time: since issue472 the receipt carries `seconds` (the run's wall clock) and `at` is
    # the moment the receipt was WRITTEN - the end of the run, the touched receipt's meaning. A
    # receipt from before that has no `seconds` and its `at` was the START, so for it alone the log's
    # mtime (written as the run streamed) minus `at` is the finish.
    seconds = rget("seconds")
    logpath = os.path.join(REPO, (logrel or "").replace("./", "").replace("/", os.sep))
    if at and seconds is not None and not running:
        wall = int(seconds)
        fact("suiteWallMinutes", round(wall / 60.0, 1) if wall > 0 else None,
             "wall minutes of the most recent suite run",
             RC_HOW + "`seconds`, stamped by keel-cli/src/suite.rs as the write moment minus the launch "
                      "(issue472). WALL time: compilation and the gaps between binaries included, not the "
                      "sum of the per-binary 'finished in' values."
             if wall > 0 else RC_HOW + "`seconds` is 0; nothing honest to derive.",
             as_of=datetime.fromtimestamp(int(at)).date().isoformat())
    elif at and logrel and os.path.exists(logpath):
        wall = os.path.getmtime(logpath) - int(at)
        fact("suiteWallMinutes", round(wall / 60.0, 1) if wall > 0 else None,
             "wall minutes of the most recent suite run",
             RC_HOW + "mtime(%s) minus `at` - a receipt from before issue472, whose `at` was the run's "
                      "START and which carries no `seconds`; the log is written as the run streams, so "
                      "its mtime is the finish. This "
                      "is WALL time. It is deliberately NOT the sum of the per-binary 'finished in' "
                      "values in the log, which is test time and excludes compilation and the gaps "
                      "between binaries." % logrel
             if wall > 0 else RC_HOW + "log mtime is not after `at`; nothing honest to derive.",
             as_of=datetime.fromtimestamp(int(at)).date().isoformat())
    else:
        fact("suiteWallMinutes", None, "minutes",
             RC_HOW + "the receipt's `log` (%r) is not on disk, so there is no finish timestamp to "
                      "difference against `at`. Refusing to substitute the summed per-binary "
                      "'finished in' values: that is test time, not wall time." % logrel)

# ================================================================ 11. Decision -> Need/UC/SR edges
# Typed edge lines look like `#DerivedFrom dependency from <src> to <dst>;`. A destination's TYPE
# comes from its declaration `<name> : Need|UseCase|SystemRequirement {` anywhere in the model.

DECL_RE = re.compile(r"\b([A-Za-z][A-Za-z0-9_]*)\s*:\s*(Need|UseCase|SystemRequirement)\s*\{")
EDGE_RE = re.compile(r"#([A-Za-z]+)\s+(?:dependency|connection)\s+from\s+"
                     r"([A-Za-z0-9_]+)\s+to\s+([A-Za-z0-9_]+)\s*;")
DEC_SRC = re.compile(r"^d\d{4}$")

sysml_files = []
for base in (".tracking", ".engine"):
    for dp, _dn, fns in os.walk(os.path.join(REPO, base)):
        for f in fns:
            if f.endswith(".sysml"):
                sysml_files.append(os.path.join(dp, f))

types = {}
for p in sysml_files:
    t = read(p) or ""
    for m in DECL_RE.finditer(t):
        types[m.group(1)] = m.group(2)

hits = []
for p in sysml_files:
    t = read(p) or ""
    for line in t.splitlines():
        if line.lstrip().startswith("//"):
            continue                       # a commented-out edge is not an edge
        for m in EDGE_RE.finditer(line):
            _mk, src, dst = m.groups()
            if DEC_SRC.match(src) and dst in types:
                hits.append((src, dst, types[dst]))

by_type = {}
for _s, _d, t in hits:
    by_type[t] = by_type.get(t, 0) + 1
fact("decisionNeedEdges", len(hits),
     "typed edges from a Decision to a Need / UseCase / SystemRequirement",
     "walk every .sysml under .tracking/ and .engine/. Build name -> type from declarations "
     r"`<name> : (Need|UseCase|SystemRequirement) {` (the keyword varies - `requirement n... : Need`, "
     "`use case uc... : UseCase`), then count non-comment lines matching "
     r"`#<Marker> dependency from d<NNNN> to <name>;` whose destination is in that map. "
     "Breakdown: " + (", ".join("%s=%d" % (k, v) for k, v in sorted(by_type.items())) or "none") +
     ". FINDING: d0355's own consequences state 'no Decision in this repository is connected by any "
     "edge to a Need or UseCase' - this count contradicts that. The edges exist but are concentrated "
     "in two old clusters (d0094 -> the serve Needs, and the keel-viewer Needs); the claim is right "
     "in spirit for RECENT Decisions and wrong as written.")

# ================================================================ 9. instruments and control proofs
# The measures, and whether the controls have ever been shown to catch anything (D0360/D0361).
cs = as_json(run([KEEL, "show", "control-structure", "."])[1]) or {}
fact("instrumentsDeclared", len(cs.get("sensors") or []) or None, "declared measures (Sensor items)",
     "`keel show control-structure`: length of the sensors array. The measures were briefly declared in a "
     "contract file; they are model items now (D0363), so this counts what the model holds rather than what "
     "a manifest claimed - one source, and a stale entry cannot survive `edge-endpoints`.")
fact("feedbackChannels", len(cs.get("feedback") or []) or None, "computed feedback channels",
     "`keel show control-structure`: length of the feedback array - the upward half of the control structure.")
sensors = cs.get("sensors") or []
fact("sensorsComputed", len(sensors) or None, "computed sensors",
     "`keel show control-structure`: length of the sensors array. Zero before 2026-09-06: none had ever existed.")
assessed = [x for x in sensors if not str(x.get("propriety", "")).startswith("UNASSESSED")]
fact("instrumentsAssessed", len(assessed), "instruments with a propriety finding",
     "`keel show control-structure`: sensors whose computed `propriety` is not UNASSESSED - a JOIN against "
     "ProprietyFinding targets, never a stored flag (the owner's correction, 2026-09-07).")

ok, census = run(["python", ".engine/tools/guard_proof_census.py"])
proven = unnamed = None
for line in (census or "").splitlines():
    if "asserts a FAILURE" in line:
        proven = int(line.split(":")[-1].strip())
    elif "named nowhere in any test body" in line:
        unnamed = int(line.split(":")[-1].strip())
fact("guardsProven", proven, "guards named in a test that asserts a failure",
     "`python .engine/tools/guard_proof_census.py`: a DEMONSTRATED CATCH, distinct from declared and armed "
     "(D0360)." + ("" if ok else " census failed: " + str(census)[:80]))
fact("guardsUnnamed", unnamed, "guards named in no test body at all",
     "`python .engine/tools/guard_proof_census.py`: neither a demonstrated catch nor a demonstrated pass.")

# ================================================================ 10. the routing probe's price (D0378)
# The numbers the routingNumber fork rests on. The prompt count is read from the rig's own table; the
# costs are read from issue392's text - the one place they were recorded - never retyped here.
_rp = read(os.path.join(REPO, ".engine", "tools", "routing_prompts.toml")) or ""
fact("routingPrompts", _rp.count("[[case]]") or None, "prompts in the routing table",
     "count of `[[case]]` tables in .engine/tools/routing_prompts.toml - one per deployed process.")
_i392 = ""
for _p in glob.glob(os.path.join(REPO, ".tracking", "issues-*.sysml")):
    _t = read(_p) or ""
    _m = re.search(r"part issue392 : Issue \{(.*?)\n    \}", _t, re.S)
    if _m:
        _i392 = _m.group(1)
        break
I392_HOW = "read from the description of issue392 (the probe's cost, measured on this host 2026-09-06): "
_m = re.search(r"\$(\d+\.\d+)-(\d+\.\d+) and (\d+)-(\d+)s per run sequentially", _i392)
fact("routingRunCostLowUsd", float(_m.group(1)) if _m else None, "USD per probe run (low)",
     I392_HOW + "the `$a-b ... per run` span.")
fact("routingRunCostHighUsd", float(_m.group(2)) if _m else None, "USD per probe run (high)",
     I392_HOW + "the `$a-b ... per run` span.")
fact("routingRunSecLow", int(_m.group(3)) if _m else None, "seconds per probe run (low)",
     I392_HOW + "the `a-bs per run sequentially` span.")
fact("routingRunSecHigh", int(_m.group(4)) if _m else None, "seconds per probe run (high)",
     I392_HOW + "the `a-bs per run sequentially` span.")
_m = re.search(r"One sample per skill over \d+ skills is ~\$(\d+)", _i392)
fact("routingOneSampleUsd", int(_m.group(1)) if _m else None, "USD, one sample of every prompt",
     I392_HOW + "the `One sample per skill ... is ~$N` sentence.")
_m = re.search(r"Three samples[^$]*~\$(\d+) and around an hour", _i392)
fact("routingThreeSamplesUsd", int(_m.group(1)) if _m else None, "USD, three samples of every prompt",
     I392_HOW + "the `Three samples ... ~$N and around an hour` sentence.")
fact("routingThreeSamplesMinutes", 60 if _m else None, "minutes, three samples of every prompt",
     I392_HOW + "the same sentence's `around an hour`.")
_m = re.search(r"(\w+) of the (\w+) cases common to both runs changed verdict", _i392)
_words = {"one": 1, "two": 2, "three": 3, "four": 4}
fact("routingVerdictsFlipped", _words.get((_m.group(1) if _m else "").lower()),
     "verdicts that changed between two identical runs",
     I392_HOW + "the `N of the M cases ... changed verdict` sentence.")
fact("routingVerdictsCompared", _words.get((_m.group(2) if _m else "").lower()),
     "verdicts compared across two identical runs", I392_HOW + "the same sentence.")
_low = FACTS["routingRunCostLowUsd"]["value"]
_high = FACTS["routingRunCostHighUsd"]["value"]
fact("routingSubsetThreeSamplesUsd",
     round(16 * 3 * (_low + _high) / 2.0) if _low and _high else None,
     "USD, three samples of a 16-prompt subset",
     "16 prompts x 3 samples x the midpoint of routingRunCostLowUsd..HighUsd. The subset size (12 fixed + 4 "
     "rotated) is the shape proposed in D0378 option B, not a measured fact; the per-run cost is.")
fact("routingSubsetThreeSamplesMinutes",
     round(16 * 3 * 31 / 60.0) if "31s per run at four-way parallelism" in _i392 else None,
     "minutes, three samples of a 16-prompt subset at four-way parallelism",
     I392_HOW + "`31s per run at four-way parallelism`, times 48 runs. The subset size is D0378 option B's proposal.")

# ================================================================ 11. the STPA step-2 gate (D0363 / sprint610 / D0383)
# The gate the SOP used to state in prose is a row set now; `cs` was read above in section 9.
_g = cs.get("stepTwoGate") or []
fact("stepTwoClauses", len(_g) or None, "clauses of the SOP's step-2 gate the view decides",
     "`keel show control-structure`: length of the stepTwoGate array - one row per clause of the stpa SOP's "
     "step-2 gate (sprint610). Zero before 2026-09-08: the gate was a paragraph.")
fact("stepTwoHolds", sum(1 for r in _g if r.get("holds") is True) if _g else None, "of those clauses that hold",
     "`keel show control-structure`: stepTwoGate rows with holds == true. Each row carries the evidence that "
     "decided it; this counts, it does not re-judge.")
fact("controllersPresent", len(cs.get("controllers") or []) or None, "controller roles this project wires",
     "`keel show control-structure`: length of the controllers array (present roles only since sprint610).")
fact("rolesAbsent", len(cs.get("absentRoles") or []), "controller roles nothing wires",
     "`keel show control-structure`: length of absentRoles - " +
     (", ".join(r.get("role", "?") for r in (cs.get("absentRoles") or [])) or "none") +
     ". A role here is drawn nowhere; the diagram footnotes it with what would wire it.")
fact("actuatorsComputed", len(cs.get("actuators") or []) or None, "actuators derived from the actions",
     "`keel show control-structure`: length of the actuators array. Zero before sprint610.")
fact("otherInputsOutputs", len(cs.get("otherInputsOutputs") or []) or None,
     "authored OtherInputOutput items (the fifth element type, D0363)",
     "`keel show control-structure`: length of otherInputsOutputs, joined from engine-control-structure.sysml.")
fact("responsibilitiesComputed", len(cs.get("responsibilities") or []) or None,
     "controller -> hazard responsibilities, one hierarchical level deep",
     "`keel show control-structure`: length of the responsibilities array (actions x hazardsByProcess).")
_ok, _gd = run([KEEL, "gate", "guard", "."], timeout=180)
_m = re.search(r"stpa-currency: (\d+) of (\d+) computed control action", _gd or "")
fact("stpaActionsUnanalysed", int(_m.group(1)) if _m else None, "computed control actions no stpa-self run has analysed",
     "`keel gate guard .`: the stpa-currency WARN line's first number" + (" of %s" % _m.group(2) if _m else " - line not found") +
     ". sprint610 added agentEditsDeliverable to the action set, which is the designed re-run trigger (D0313).")

# ================================================================ 13. the recall cap (D0389 / D0390)
# The hook latency distribution keel show enforcement-report computes from the fire-ledger, and the over-cap
# proxy for skips before recall-skipped existed. Every number is read from the report or the ledger.
_er_ok, _er_raw = run([KEEL, "show", "enforcement-report", REPO])
ER_HOW = "keel show enforcement-report (D0389): per-event nearest-rank latency over .keel/metrics/hooks.jsonl, machine-local; "
try:
    _er = json.loads(_er_raw) if _er_ok and _er_raw else {}
except ValueError:
    _er = {}
_rows = {e.get("event"): e for e in _er.get("perEvent", [])}
for _ev, _key in (("user-prompt", "userPrompt"), ("post-edit", "postEdit")):
    _r = _rows.get(_ev) or {}
    fact(_key + "Fires", _r.get("fires"), "fires", ER_HOW + "`perEvent[%s].fires`." % _ev)
    for _f in ("msMedian", "msP90", "msP99", "msMax"):
        fact(_key + _f[2:], _r.get(_f), "ms", ER_HOW + "`perEvent[%s].%s`." % (_ev, _f))
_cap = 2500
_over = None
_ledger = read(os.path.join(REPO, ".keel", "metrics", "hooks.jsonl"))
if _ledger:
    _over = 0
    for _l in _ledger.splitlines():
        try:
            _o = json.loads(_l)
        except ValueError:
            continue
        if _o.get("event") == "user-prompt" and _o.get("ms", 0) > _cap:
            _over += 1
fact("userPromptOverCap", _over, "fires over the 2,500 ms recall cap",
     "count of user-prompt lines in .keel/metrics/hooks.jsonl with ms > 2500 - the PROXY for a dropped payload before "
     "recall-skipped existed (D0389); the check runs after the walk (main.rs recalled_facts), so every one of these paid "
     "the walk and received nothing.")
fact("recallSkippedCounted", (_er.get("recall") or {}).get("skipped"), "recall-skipped events since D0389",
     ER_HOW + "`recall.skipped`.")

# ================================================================ 14. script probes in CI (D0394)
# How many scripts under scripts/ ship a known-answer entry point (--probe / --self-test), and whether
# CI runs them. Read from the tree.
_probe_scripts = []
for _p in glob.glob(os.path.join(REPO, "scripts", "**", "*.py"), recursive=True):
    for _line in (read(_p) or "").splitlines():
        if _line.strip().startswith("# ci-probe:"):
            _probe_scripts.append(os.path.relpath(_p, REPO).replace(os.sep, "/"))
            break
fact("scriptsWithProbes", len(_probe_scripts) or None, "scripts under scripts/ shipping a known-answer entry point",
     "count of scripts/**/*.py carrying a `# ci-probe:` marker line: " + (", ".join(sorted(_probe_scripts)) or "none") + ".")
_ci = read(os.path.join(REPO, ".github", "workflows", "ci.yml")) or ""
fact("ciRunsProbes", ("--probe" in _ci or "--self-test" in _ci), "does CI run the script probes",
     ".github/workflows/ci.yml mentions `--probe` or `--self-test`: today it does not, so these checks run only by hand.")

# The fit sensor's own probe set and where it runs (D0402/D0403): the number of constructed known cases,
# read from the module, and whether the brief builders end in it. A builder whose last statement is not
# assert_fits publishes unmeasured.
_fit_src = read(os.path.join(REPO, "scripts", "exec_brief", "fit_check.py")) or ""
_fit_cases = re.search(r"PROBE_CASES\s*=\s*\{([\s\S]*?)\n\}", _fit_src)
fact("fitProbeCases", len(re.findall(r'^\s{4}"[^"]+":', _fit_cases.group(1), re.M)) if _fit_cases else None,
     "constructed known cases fit_check runs before measuring a page",
     "count of top-level keys in PROBE_CASES in scripts/exec_brief/fit_check.py (each is a page built to show one "
     "finding kind, or the fitting figure that must show none); null if the dict is not found.")
_builders = sorted(glob.glob(os.path.join(REPO, "scripts", "exec_brief", "build_*.py")))
_ending = [b for b in _builders if (read(b) or "").rstrip().splitlines()[-1].lstrip().startswith("assert_fits(")]
fact("briefBuilders", len(_builders) or None, "brief builder scripts", "count of scripts/exec_brief/build_*.py.")
fact("briefBuildersEndingInFit", len(_ending), "builders whose last statement is assert_fits",
     "of those, the ones whose last non-blank line begins `assert_fits(`: " + (", ".join(os.path.basename(b) for b in _ending) or "none") + ".")

# ================================================================ 15. releases bound to tags (D0400)
# How a git tag finds its Release record. Read from `git tag` and .tracking/baselines.sysml; the
# old rule (title contains the tag, first hit) is re-run here over the SAME records so its miscount is a
# number, not a story.
_tags = [t.strip() for t in (run(["git", "-C", REPO, "tag"])[1] or "").splitlines() if re.match(r"^v\d+\.\d+\.\d+$", t.strip())]
_rel_blocks = []
_cur = None
for _line in (read(os.path.join(REPO, ".tracking", "baselines.sysml")) or "").splitlines():
    _s = _line.strip()
    if _s.startswith("part ") and ": Release {" in _s:
        _cur = {"name": _s.split()[1], "title": "", "tag": ""}
    elif _cur is not None and _s.startswith(':>> title = "'):
        _cur["title"] = _s.split('"')[1]
    elif _cur is not None and _s.startswith(':>> tag = "'):
        _cur["tag"] = _s.split('"')[1]
    elif _cur is not None and _s == "}":
        _rel_blocks.append(_cur)
        _cur = None
fact("versionTags", len(_tags) or None, "git tags of the form vN.N.N", "`git tag` filtered to vN.N.N: " + ", ".join(_tags) + ".")
fact("releaseRecords", len(_rel_blocks) or None, "Release blocks in .tracking/baselines.sysml", "count of `part X : Release {` blocks.")
fact("releaseRecordsWithTagField", sum(1 for b in _rel_blocks if b["tag"]), "Release blocks carrying `:>> tag`",
     "count of blocks with a `:>> tag = ` line (D0400 migration 2026-09-09-release-tag-field.py).")
_title_hits = {t: [b["name"] for b in _rel_blocks if t in b["title"]] for t in _tags}
_title_wrong = [t for t in _tags if _title_hits[t] and next(b["tag"] for b in _rel_blocks if b["name"] == _title_hits[t][0]) != t]
_title_multi = [t for t in _tags if len(_title_hits[t]) > 1]
fact("tagsWithSeveralTitleMatches", len(_title_multi), "tags whose string occurs in more than one Release title",
     "re-running the pre-D0400 rule (title contains tag) over the same records: " + (", ".join(f"{t} -> {', '.join(_title_hits[t])}" for t in _title_multi) or "none") + ".")
fact("tagsMisboundByTitle", len(_title_wrong), "tags the title rule would bind to a record whose `tag` field says otherwise",
     "for each tag, the FIRST title hit compared with the record whose `tag` field equals it: " + (", ".join(f"{t} -> first title hit {_title_hits[t][0]}" for t in _title_wrong) or "none") + ".")
fact("releaseGuardWarnings", None, "release-recorded warnings on this tree", "not run here; keel gate guard release-recorded . prints it.")
_rr_ok, _rr_raw = run([KEEL, "gate", "guard", "release-recorded", REPO, "--no-receipt"])
_m = re.search(r"release-recorded\] \w+ [^0-9]*(\d+) scanned, (\d+) warning", _rr_raw or "")
if _m:
    fact("releaseGuardWarnings", int(_m.group(2)), "release-recorded warnings on this tree",
         "`keel gate guard release-recorded . --no-receipt` summary line: %s scanned, %s warning(s)." % (_m.group(1), _m.group(2)))

# ================================================================ 16. tests bound to source (D0401)
# Which tests in keel-cli read program source, and how many assert a code-shaped literal is PRESENT in
# it. A text scan, coarser than the Rust control (tests_bind_to_properties.rs) but run over two trees:
# the parent of the commit that added the control (where the offenders still stood) and HEAD. Probed
# before either number is stated (D0388): the parent tree is the known positive, HEAD the known
# negative - its own control is green - and a scan that disagrees with either refuses.
_CODE_TOKENS = (";", "{", "}", "==", "!=", "return ", "let ", "if ", "fn ", "=>", "()")
_BIND_RE = re.compile(r'(?:const|static|let(?:\s+mut)?)\s+([A-Za-z_]\w*)\b[^;]*?(?:include_str!|read_to_string)\s*\([^)]*\.rs"')
_NEEDLE_RE = re.compile(r'(!?)\s*([A-Za-z_]\w*)(?:\[[^\]]*\])?\.(?:contains|matches)\(\s*"((?:[^"\\]|\\.)*)"\s*\)')
_LET_RE = re.compile(r'\blet\s+(?:mut\s+)?([A-Za-z_]\w*)\b[^=]*=([^;]*)')
_CONTROL_FILE = "keel-cli/tests/tests_bind_to_properties.rs"


def _surface_at(rev):
    """(path, text) for every test region: src files from their first #[cfg(test)], tests/*.rs whole."""
    ok, listing = run(["git", "-C", REPO, "ls-tree", "-r", "--name-only", rev, "keel-cli/src", "keel-cli/tests"])
    out = []
    for p in (listing or "").splitlines():
        p = p.strip()
        if not p.endswith(".rs") or p == _CONTROL_FILE:
            continue
        if p.startswith("keel-cli/tests/") and p.count("/") != 2:
            continue
        ok, text = run(["git", "-C", REPO, "show", "%s:%s" % (rev, p)])
        if not ok or text is None:
            continue
        if p.startswith("keel-cli/src/"):
            at = text.find("#[cfg(test)]")
            if at < 0:
                continue
            text = text[at:]
        out.append((p, text))
    return out


def _test_fns(text):
    """Test function bodies, split at test attributes; the region's module-level text is index 0."""
    parts = re.split(r"(?=#\[(?:test|tokio::test))", text)
    return parts[0], parts[1:]


def _census(rev):
    """(source-reading tests, code-shaped positive asserts, tests carrying one)."""
    reading, asserts, offenders = 0, 0, 0
    for path, text in _surface_at(rev):
        module, fns = _test_fns(text)
        module_bound = set(_BIND_RE.findall(module))
        for body in fns:
            bound = set(_BIND_RE.findall(body)) | {b for b in module_bound if re.search(r"\b%s\b" % re.escape(b), body)}
            if not bound:
                continue
            grown = True                       # a `let` whose right side names a bound name binds its left side too
            while grown:
                grown = False
                for name, rhs in _LET_RE.findall(body):
                    if name not in bound and any(re.search(r"\b%s\b" % re.escape(b), rhs) for b in bound):
                        bound.add(name)
                        grown = True
            reading += 1
            hits = 0
            for neg, recv, lit in _NEEDLE_RE.findall(body):
                if neg == "!" or not any(t in lit for t in _CODE_TOKENS):
                    continue
                if any(re.search(r"\b%s\b" % re.escape(b), recv) for b in bound):
                    hits += 1
            asserts += hits
            offenders += 1 if hits else 0
    return reading, asserts, offenders


_ok_added, _added_raw = run(["git", "-C", REPO, "log", "--format=%H", "--diff-filter=A", "--", _CONTROL_FILE])
_added = (_added_raw or "").split()
_added_sha = _added[-1] if _added else None
_before = _census(_added_sha + "^") if _added_sha else (None, None, None)
_now = _census("HEAD")
# The known positive is not "some": the control's first real-tree run named 3 tests carrying 5 asserts
# (sprint 627), so a scan that sees fewer has missed a shape and its numbers are withheld.
_PROBE_OK = bool(_added_sha) and _before[2] == 3 and _before[1] == 5 and _now[2] == 0
_HOW = ("text scan over keel-cli test regions (src from the first #[cfg(test)], tests/*.rs, the control's own file "
        "excluded): a test READS SOURCE when its body, or a module const it names, binds include_str!/read_to_string "
        "of a .rs path, or a `let` whose right side names such a binding (to a fixpoint); an assert is CODE-SHAPED when a non-negated .contains/.matches on a bound name carries a literal "
        "holding one of ; { } == != return let if fn => (). Probe (D0388): the parent of the commit adding the control "
        "must show the 3 tests / 5 asserts its first run named, and HEAD none - " + ("held" if _PROBE_OK else "FAILED, numbers withheld") + ".")
if not _PROBE_OK:
    _before, _now = (None, None, None), (None, None, None)
fact("sourceReadingTestsBefore", _before[0], "tests reading program source, before the rebinding", _HOW + " Tree: %s^." % (_added_sha or "?")[:7])
fact("codeShapedAssertsBefore", _before[1], "positive code-shaped asserts on bound source, before", _HOW)
fact("sourceBoundOffendersBefore", _before[2], "tests carrying such an assert, before", _HOW)
fact("sourceReadingTestsNow", _now[0], "tests reading program source at HEAD", _HOW)
fact("sourceBoundOffendersNow", _now[2], "tests carrying a code-shaped assert at HEAD", _HOW)
_probe_fns = len(re.findall(r"\bfn probe_", read(os.path.join(REPO, _CONTROL_FILE)) or ""))
fact("testsBindProbes", _probe_fns or None, "known-answer probes shipped with the control",
     "count of `fn probe_` in %s: the fixtures the discriminator is run against before the tree is read (D0388)." % _CONTROL_FILE)

# ================================================================ 17. what this range changed (D0282 / dcCommitDeltaView)
# `keel show commit-delta` is the model delta over a git range - items ADDED by type, items RETIRED by a
# #Supersede edge, Issues RESOLVED - reconciled against the diff's declaration count. The brief's "what this
# range changed" section is built from THIS fact, never typed: the range runs from the tree the page was last
# published against (`publishedAgainst` in .keel/decision-page.toml) to HEAD, so a reader of the refreshed page
# sees what the model gained since they last read it. The decision channel the item's DoD named as a second
# surface is disconnected (D0291); the page is the surface.
_page = read(os.path.join(REPO, ".keel", "decision-page.toml")) or ""
_m = re.search(r'^publishedAgainst\s*=\s*"([0-9a-fA-F]+)"', _page, re.M)
_delta_from = _m.group(1) if _m else None
_DELTA_HOW = ("`keel show commit-delta . --range %s..HEAD` (D0282): items of a delta type - Need, SystemRequirement, "
              "Requirement, Decision, Issue, action - present at HEAD and absent at the range start, #Supersede and "
              "#Resolves edges new in the range; `reconciled` is the view's per-type count against the "
              "`+part <name> : <Type>` / `+action <name>;` lines git diff adds NET of the same name removed (a move)."
              % (_delta_from or "?"))
if _delta_from is None:
    fact("commitDelta", None, "model delta since the last publish",
         "no `publishedAgainst` in .keel/decision-page.toml, so the range has no start - " + _DELTA_HOW)
else:
    ok, out = run([KEEL, "show", "commit-delta", ".", "--range", "%s..HEAD" % _delta_from], timeout=180)
    _delta = None
    if ok:
        try:
            _delta = json.loads(out)
        except ValueError:
            ok, out = False, "commit-delta emitted non-JSON: %s" % out[:200]
    if _delta is None:
        fact("commitDelta", None, "model delta since the last publish", "commit-delta failed: %s - %s" % (out, _DELTA_HOW))
    else:
        fact("commitDelta", {
            "range": _delta.get("range"),
            "empty": _delta.get("empty"),
            "added": _delta.get("added", []),
            "superseded": _delta.get("superseded", []),
            "resolved": _delta.get("resolved", []),
            "reconciled": (_delta.get("reconciliation") or {}).get("matches"),
        }, "model delta since the last publish", _DELTA_HOW)
        fact("commitDeltaAdded", len(_delta.get("added", [])), "items added since the last publish", _DELTA_HOW)
        fact("commitDeltaSuperseded", len(_delta.get("superseded", [])), "items retired since the last publish", _DELTA_HOW)
        fact("commitDeltaResolved", len(_delta.get("resolved", [])), "Issues resolved since the last publish", _DELTA_HOW)

# ================================================================ 18. family call-site census (D0452 / D0453)
# The two safety Decisions fold the gating and channel families into routers and rewrite every call site in
# the SAME commit that removes the names. A page that says how many call sites move, and where, computes the
# number here with the census method each Decision's research comment names: the family's MEMBERS come from
# the CliCommand facts in .engine/cli/commands.sysml, the CALL SITES from the bounded regex of section 2
# restricted to those members over `git ls-files` (target/ is untracked, so excluded), every occurrence
# assigned to exactly ONE area by its path. Which members move and which stay is the Decision's text and
# stays in the builder; this section only counts.
_AREAS = [   # (area, predicate) - first match wins, so .engine/decisions is split out before .engine
    ("hooksAndCi", lambda p: p.startswith(".githooks/") or p.startswith(".github/workflows/")),
    ("decisions", lambda p: p.startswith(".engine/decisions/")),
    ("engine", lambda p: p.startswith(".engine/")),
    ("claude", lambda p: p.startswith(".claude/")),
    ("rustSource", lambda p: p.startswith("keel-cli/src/")),
    ("rustTests", lambda p: p.startswith("keel-cli/tests/")),
    ("rootDocs", lambda p: "/" not in p and p.endswith(".md")),
    ("tracking", lambda p: p.startswith(".tracking/")),
    ("other", lambda p: True),
]
_NOT_REWRITTEN = ("decisions", "tracking")   # recorded history: never rewritten (D0129), reported apart
_FAMILY_HOW = ("members: the `name` of EVERY non-lens CliCommand fact in .engine/cli/commands.sysml whose `family` "
               "is \"%s\", deprecated ones included - cliLiveTopLevelByFamily (live only) would drop a deprecated "
               "verb from a members table without a trace; memberFacts carries each name with its CliEffect and "
               "CliStability. sites: git ls-files, then for each tracked TEXT file count regex "
               r"`(?<![\w.-])(keel\.exe|keelw|keel)[ \t]+<member>(?![\w-])` (longest member first, so a hyphenated "
               "name never collapses into its prefix; literal \\n/\\r/\\t normalised to a space, as in section 2), "
               "OCCURRENCES not lines, each file assigned to ONE area by path, first match wins: hooksAndCi = "
               ".githooks/ + .github/workflows/; decisions = .engine/decisions/; engine = the rest of .engine/; "
               "claude = .claude/; rustSource = keel-cli/src/; rustTests = keel-cli/tests/; rootDocs = top-level "
               "*.md; tracking = .tracking/; other = everything else tracked. `sites[area][member]` and "
               "`files[area][member]` (distinct files) are the grid; `rewritten` sums every area except decisions "
               "and tracking, which the Decision does not rewrite and `notRewritten` sums. target/ is untracked "
               "and therefore absent. The verb-to-router mapping is NOT here: it is the Decision's text.")


def _family_census(family, members, tracked):
    verbs = sorted(members, key=len, reverse=True)
    rx = re.compile(r"(?<![\w.-])(?:keel\.exe|keelw|keel)[ \t]+(" + "|".join(re.escape(v) for v in verbs) + r")(?![\w-])")
    sites = {a: {v: 0 for v in members} for a, _ in _AREAS}
    files = {a: {v: 0 for v in members} for a, _ in _AREAS}
    for rel in tracked:
        text = read(os.path.join(REPO, rel.replace("/", os.sep)))
        if text is None or "\0" in text[:4096]:
            continue
        hits = rx.findall(ESCAPE_RE.sub(" ", text))
        if not hits:
            continue
        area = next(a for a, pred in _AREAS if pred(rel))
        for v in set(hits):
            files[area][v] += 1
        for v in hits:
            sites[area][v] += 1
    total_by_area = {a: sum(sites[a].values()) for a, _ in _AREAS}
    return {
        "family": family,
        "members": sorted(members),
        "sites": sites,
        "files": files,
        "totalByArea": total_by_area,
        "rewritten": sum(n for a, n in total_by_area.items() if a not in _NOT_REWRITTEN),
        "notRewritten": sum(n for a, n in total_by_area.items() if a in _NOT_REWRITTEN),
    }


# members are EVERY CliCommand fact of the family, deprecated ones included: the Decision moves the names
# the facts file holds. Each member carries its effect and stability so the page can say which of the
# moving names the surface already calls deprecated.
_by_family = None
if cli_src is not None:
    _by_family = {}
    for _b in re.finditer(r"part\s+cli\w*\s*:\s*CliCommand\s*\{([^\n]*)", cli_src):
        _b = _b.group(1)
        _nm = re.search(r'name\s*=\s*"([^"]+)"', _b)
        _fm = re.search(r'family\s*=\s*"([^"]+)"', _b)
        _ef = re.search(r"CliEffect::(\w+)", _b)
        _st = re.search(r"CliStability::(\w+)", _b)
        if _nm and _fm and _ef and _st and not re.search(r'invocation\s*=\s*"show\s', _b):
            _by_family.setdefault(_fm.group(1), []).append(
                {"name": _nm.group(1), "effect": _ef.group(1), "stability": _st.group(1)})
ok, out = run(["git", "ls-files"])
for _fam, _fact_name in (("gating", "familyCensusGating"), ("channel", "familyCensusChannel")):
    if not ok:
        fact(_fact_name, None, "call sites by area and member", _FAMILY_HOW % _fam + " `git ls-files` failed: " + out)
    elif not _by_family or _fam not in _by_family:
        fact(_fact_name, None, "call sites by area and member",
             _FAMILY_HOW % _fam + " commands.sysml is unreadable or has no %s family, so the members are unknown" % _fam)
    else:
        _members = [r["name"] for r in _by_family[_fam]]
        _c = _family_census(_fam, _members, [p for p in out.splitlines() if p.strip()])
        _c["memberFacts"] = sorted(_by_family[_fam], key=lambda r: r["name"])
        fact(_fact_name, _c, "call sites by area and member", _FAMILY_HOW % _fam)

# the dispatch count the Decisions say falls (by ten, by four) and the implement gate reads back
_HARD_HOW = ("`keel show hardening .` (D0169/D0434), field helpCoverage.dispatched: the number of match arms "
             "in keel-cli/src/main.rs that dispatch a top-level command; the read-back the two Decisions' "
             "implement gates name.")
ok, out = run([KEEL, "show", "hardening", "."], timeout=120)
_hard = as_json(out) if ok else None
_hc = (_hard or {}).get("helpCoverage") or {}
fact("cliDispatchArms", _hc.get("dispatched") if _hc.get("available") else None, "dispatch arms",
     _HARD_HOW if _hc.get("available") else _HARD_HOW + " hardening did not answer: " + out[:200])


# the count d0457 puts to the human: every remaining top-level family by member, effect and stability, the
# distance to D0273's under-25 clause, and the fold arithmetic against d0399's baseline commit - each from the
# facts file at the two commits, none typed from a Decision.
_D0399_BASE = "649b433"
_CENSUS_HOW = ("keel-cli/src/cli_facts.rs, every `CliFact {{ name, family, effect, stability }}` whose family is not "
               "`lens` (the 51 lenses sit under show), grouped by family with the member list; byEffect counts the "
               "same set; topLevel is the set's size and namesToUnder25 = topLevel - 24, the names that must go (armsToUnder25 the same from the help count, which has `help`) "
               "for D0273's clause to hold; baseline* comes from `git show {base}:keel-cli/src/cli_facts.rs` read the "
               "same way (d0399's counts were taken at {base}), removed = baseline - current, added = current - "
               "baseline; dispatchArms is cliDispatchArms (hardening), which is topLevel + `help`.")
_FACT_RX = re.compile(r'CliFact \{ name: "([^"]+)", family: "([^"]+)", effect: "([^"]+)", stability: "([^"]+)"')


def _top_level_facts(src):
    return [(n, f, e, s) for n, f, e, s in _FACT_RX.findall(src) if f != "lens"]


_cli_facts_home = _mh("cli_facts")
try:
    if _cli_facts_home is None:
        raise OSError("module cli_facts is in no workspace crate (scripts/module_home.py over Cargo.toml [workspace] members)")
    _cur_src = open(_cli_facts_home, encoding="utf-8").read()
except OSError as _e:
    _cur_src = None
    fact("cliFamilyCensus", None, "top-level verbs by family", _CENSUS_HOW.format(base=_D0399_BASE) +
         " cli_facts.rs unreadable: %s" % _e)
if _cur_src is not None:
    _cur = _top_level_facts(_cur_src)
    _fams = {}
    _eff = {}
    for _n, _f, _e, _s in _cur:
        _fams.setdefault(_f, []).append({"name": _n, "effect": _e, "stability": _s})
        _eff[_e] = _eff.get(_e, 0) + 1
    ok, _base_src = run(["git", "show", "%s:keel-cli/src/cli_facts.rs" % _D0399_BASE])
    _base = _top_level_facts(_base_src) if ok else None
    _cur_names = {n for n, _, _, _ in _cur}
    _base_names = {n for n, _, _, _ in _base} if _base is not None else None
    fact("cliFamilyCensus", {
        "topLevel": len(_cur),
        "namesToUnder25": len(_cur) - 24,
        "armsToUnder25": (_hc.get("dispatched") - 24) if _hc.get("available") else None,
        "dispatchArms": _hc.get("dispatched") if _hc.get("available") else None,
        "families": [{"family": f, "count": len(ms), "members": sorted(ms, key=lambda r: r["name"])}
                     for f, ms in sorted(_fams.items(), key=lambda kv: (-len(kv[1]), kv[0]))],
        "byEffect": _eff,
        "baselineCommit": _D0399_BASE,
        "baselineTopLevel": len(_base) if _base is not None else None,
        "removedSinceBaseline": sorted(_base_names - _cur_names) if _base_names is not None else None,
        "addedSinceBaseline": sorted(_cur_names - _base_names) if _base_names is not None else None,
    }, "top-level verbs by family", _CENSUS_HOW.format(base=_D0399_BASE) +
        ("" if ok else " (`git show` failed, baseline rows null: %s)" % _base_src[:120]))


# ================================================================ 19. sub-verb control actions (D0454 / sprint681)
# D0454 derives one control action per sub-verb of a routing write command from the fact's own invocation.
# The page that asks for a word on it says how many actions the rule adds, which routers it opens, and how
# much of the new set the analysis has covered - all read from `cs` (section 9, this binary's structure)
# and the census view, never from the Decision's text. "Sub-verb of" is the data prefix the derivation
# writes (control_structure.rs), so an action is a sub-verb action iff its data carries it; the router it
# belongs to is the backticked command in that phrase.
_SUB_HOW = ("`keel show control-structure .`: actions whose `data` contains `sub-verb of` are the D0454 sub-verb "
            "actions; the router is the `keel <command>` inside the backticks of that phrase. subVerbActions = their "
            "count; routers = distinct routers with their per-router counts; actionsTotal = every action; "
            "actionsBeforeRule = actionsTotal - subVerbActions + len(routers), i.e. what the structure listed when "
            "each router was one action (the sprint-680 read-back of 42 is the check); bareRoutersPresent = router "
            "action names (cmd + CamelCase command) that still appear - the rule says none may.")
_acts = cs.get("actions") or []
_sub_rx = re.compile(r"sub-verb of `keel ([a-z][a-z-]*)`")
_routers = {}
for _a in _acts:
    _mm = _sub_rx.search(_a.get("data") or "")
    if _mm:
        _routers[_mm.group(1)] = _routers.get(_mm.group(1), 0) + 1
_sub_n = sum(_routers.values())
_camel = lambda s: "cmd" + "".join(w[:1].upper() + w[1:] for w in s.split("-"))
_bare = sorted(_camel(r) for r in _routers if any(_a.get("name") == _camel(r) for _a in _acts))
fact("subVerbActions", {
    "actionsTotal": len(_acts) or None,
    "subVerbActions": _sub_n,
    "routers": dict(sorted(_routers.items())),
    "actionsBeforeRule": (len(_acts) - _sub_n + len(_routers)) if _acts else None,
    "bareRoutersPresent": _bare,
}, "control actions derived per sub-verb", _SUB_HOW)

# how much of the sub-verb set the analysis has covered: the ANALYSED: lines of every recorded stpa-self run
# hold the names; the stpa-currency guard's open count is section 11's stpaActionsUnanalysed.
_UCAS_PATH = os.path.join(REPO, ".tracking", "architecture", "engine-ucas.sysml")
_ucas_src = read(_UCAS_PATH) or ""
_analysed = set()
for _line in re.findall(r"ANALYSED:([^\"]*)", _ucas_src):
    _analysed.update(re.findall(r"\bcmd[A-Z]\w*", _line))
_sub_names = [_a.get("name") for _a in _acts if _sub_rx.search(_a.get("data") or "")]
fact("subVerbActionsAnalysed", {
    "runsRecorded": len(re.findall(r"verification stpaRun\d+ : Test", _ucas_src)),
    "analysed": sorted(n for n in _sub_names if n in _analysed),
    "open": sorted(n for n in _sub_names if n not in _analysed),
}, "sub-verb actions a recorded stpa-self run names",
     "engine-ucas.sysml: every `ANALYSED:` procedureText of a `verification stpaRunN : Test`, its cmd* names collected; "
     "a sub-verb action (subVerbActions) is analysed iff one of those lines names it. runsRecorded counts the run "
     "records. The guard stpa-currency computes the same set the other way round (section 11).")

# the UCA census after the run, from the view that holds it (D0428): how many UCAs exist, how many stand on a
# control, how many on an Issue that names the gap.
ok, out = run([KEEL, "show", "control-census", "."], timeout=120)
_cc = as_json(out) if ok else None
_us = (_cc or {}).get("ucaSummary") or {}
_ul = (_cc or {}).get("ucas") or []
fact("ucaCensus", {
    "total": _us.get("ucas"),
    "observed": _us.get("observed"),
    "codeRead": _us.get("codeRead"),
    "onAnIssue": sum(1 for u in _ul if u.get("issues")),
    "onAControl": sum(1 for u in _ul if u.get("boundControls")),
} if _us else None, "UCAs and what each stands on",
     "`keel show control-census .` (D0426/D0428): ucaSummary.ucas / observed / codeRead as the view computes them; "
     "onAnIssue = ucas rows whose `issues` list is non-empty, onAControl = rows whose `boundControls` is non-empty "
     "(a row can be both)." + ("" if _us else " The view did not answer: " + (out or "")[:200]))


# ================================================================ 20. the nineteenth publish's four asks
# Three process-change ratifications and one fork. Each number here is read from the record or the tree
# that holds it - the guard's own print, the skill files, the Decision's RESEARCH line - never retyped.

# --- D0463: the direction-cited guard's own line at HEAD (scanned / violations / the counted history)
ok, out = run([KEEL, "gate", "guard", "direction-cited", "."], timeout=120)
_dc = re.search(r"\[guard:direction-cited\] (PASS|FAIL|WARN)[^\d]*(\d+) scanned, (\d+) warning\(s\) \+ (\d+) counted-history line\(s\), (\d+) violation\(s\)", out or "")
_dh = re.search(r"HISTORY\s+(\d+) citing Decision\(s\) recorded before (\d{4}-\d{2}-\d{2})[^\d]*(\d+) citing DoD\(s\) carry no createdAt", out or "")
DC_HOW = ("`keel gate guard direction-cited .` (D0463, guard 73): the verdict line's scanned / warning / counted-history / "
          "violation counts, and the HISTORY line's two counts (citing Decisions before the cutoff, citing DoDs with no "
          "createdAt) - the guard's own print, quoted." + ("" if _dc else " The guard did not print a verdict line: " + (out or "")[-200:]))
fact("directionCited", {
    "verdict": _dc.group(1), "scanned": int(_dc.group(2)), "violations": int(_dc.group(5)),
    "historyDecisions": int(_dh.group(1)) if _dh else None, "cutoff": _dh.group(2) if _dh else None,
    "historyDoDs": int(_dh.group(3)) if _dh else None,
} if _dc else None, "the guard's counts at HEAD", DC_HOW)

# --- D0460: what the two skills say today about judgedAgainst and HEAD - the lines the Decision rewrites
_sk = {}
for _rel in ("test-result", "sprint-standup"):
    _t = read(os.path.join(REPO, ".engine", "skills", _rel, "SKILL.md")) or ""
    _sk[_rel] = [ln.strip() for ln in _t.splitlines() if "judgedAgainst" in ln and ("HEAD" in ln or "≠" in ln or "!=" in ln)]
fact("skillHeadEqualityLines", {k: len(v) for k, v in _sk.items()},
     "skill lines that tie judgedAgainst to HEAD",
     "lines of .engine/skills/test-result/SKILL.md and .engine/skills/sprint-standup/SKILL.md containing `judgedAgainst` "
     "and one of `HEAD`, `≠`, `!=` - the wording D0460 replaces (the recording instruction, which keeps HEAD, is one of them).")

# --- D0461: retros in the tree that name a Decision as written (D0NNN) - the class the widened needle reads
_retro_upper = 0
_retro_total = 0
for _fn in os.listdir(os.path.join(REPO, ".tracking", "delivery")):
    if not _fn.endswith(".sysml"):
        continue
    _t = read(os.path.join(REPO, ".tracking", "delivery", _fn)) or ""
    # a record starts at a LINE beginning `verification` or `part` (the guard's own reading); the retro gates are
    # the verifications whose name says Retro; the chunk runs to the next record's line
    _chunks = re.split(r"\n(?=\s*(?:verification|part)\s)", _t)
    for _c in _chunks:
        if re.match(r"\s*verification\s+\w*Retro\w*\s*:\s*Test", _c):
            _retro_total += 1
            if re.search(r"(?<![A-Za-z0-9])D0\d{3}(?![A-Za-z0-9])", _c):
                _retro_upper += 1
fact("retrosNamingDecisionUpper", {"retros": _retro_total, "namingD0NNN": _retro_upper},
     "retro gate Tests, and how many name a Decision as D0NNN",
     "over .tracking/delivery/*.sysml: every record starting at a line `verification <name>Retro<...> : Test` (the "
     "retro gate, read to the next `verification`/`part` line as guards.rs retro_texts does), and those whose text "
     "carries `D0` + three digits at a word boundary - the form named_items did not read before D0461.")

# --- D0464: the sweep the fork stands on, quoted from the Decision's own RESEARCH line; the constant from the source
_d0464 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0464-"):
        _d0464 = read(os.path.join(DEC_DIR, _fn))
_rl = re.search(r"// RESEARCH: (.*)", _d0464)
_research = _rl.group(1) if _rl else ""
_arm1 = re.search(r"One-hop arm, 50 cases: (.*?)\. Two-hop arm", _research)
_arm2 = re.search(r"Two-hop arm, 50 cases: (.*?)\. Verdict lines", _research)


def _arm(text):
    rows = {}
    if not text:
        return rows
    # "DOMINANCE=0 hits 45/50 median 2 top-3 28/45 mean rows 12; 1.1 45/50 median 4 top-3 21/45; ..."
    for seg in text.split(";"):
        m = re.search(r"(?:DOMINANCE=)?([0-9.]+) (?:hits )?(\d+)/50 median (\d+) top-3 (\d+)/(\d+)", seg.strip())
        if m:
            rows[m.group(1)] = {"hits": int(m.group(2)), "median": int(m.group(3)), "top3": int(m.group(4))}
    # "1.3, 1.35, 1.4, 1.45 each 45/50 median 3 top-3 27/45"
    for m in re.finditer(r"((?:[0-9.]+, )+[0-9.]+) each (\d+)/50 median (\d+) top-3 (\d+)/(\d+)", text):
        for s in m.group(1).split(", "):
            rows[s] = {"hits": int(m.group(2)), "median": int(m.group(3)), "top3": int(m.group(4))}
    return rows


_hop1, _hop2 = _arm(_arm1.group(1) if _arm1 else ""), _arm(_arm2.group(1) if _arm2 else "")
_hand = {}
for m in re.finditer(r"at ([0-9.]+)(?: and at ([0-9.]+))? (?:the rebase question's d0129 arrives at position (\d+), )?(?:it does not arrive, )?injection ON (\d)/8, bar (MET|NOT MET) \((\d)/7", _research):
    for s in (m.group(1), m.group(2)):
        if s:
            _hand[s] = {"on": int(m.group(4)), "reachable": int(m.group(6)), "bar": m.group(5), "d0129Position": int(m.group(3)) if m.group(3) else None}
_ties = len(re.findall(r"COULD NOT CHOOSE on KEEL_RECALL_DOMINANCE", _research))
_kn = read(_mh("view/knowledge") or "") or ""
_const = re.search(r"const DOMINANCE: f64 = ([0-9.]+);", _kn)
_bar = re.search(r'recallRanksLinkedRecordsAsWellAsGrepDoesDoD : Test \{[^}]*?procedureText = "(.*?)"', read(os.path.join(REPO, ".tracking", "backlog.sysml")) or "", re.DOTALL)
SW_HOW = ("regex over the `// RESEARCH:` line of .engine/decisions/0464-*.sysml - the sweep that Decision records, one row per "
          "DOMINANCE setting per arm (hits/50, median position, top-3), the hand-set readings (injection ON n/8, bar MET or "
          "NOT MET, n/7 reachable, d0129's position where it arrived), and the count of COULD NOT CHOOSE tie lines; "
          "`constant` = regex `const DOMINANCE: f64 = N;` over view/knowledge.rs (module_home resolves the crate) (the value in force); "
          "`barText` = the procedureText of recallRanksLinkedRecordsAsWellAsGrepDoesDoD in .tracking/backlog.sysml. "
          "Quoted from the record and the source, never retyped; re-runnable with .engine/tools/recall_bench.py --sweep.")
fact("dominanceSweep", {
    "hop1": _hop1, "hop2": _hop2, "handSet": _hand, "tieLines": _ties,
    "constant": float(_const.group(1)) if _const else None,
    "barText": _bar.group(1) if _bar else None,
} if _hop1 and _hop2 and _hand else None, "the D0464 sweep, the constant in force, the bar's text", SW_HOW)

# ================================================================ 21. the twentieth publish's asks
# The keystone's second source (D0465), the sample the weighted rule reads at (D0466), the retro discharge
# preference (D0467), and the retro note (st122) that bears on D0461. Every number is read from the source,
# the record or the Decision's own RESEARCH line - never retyped.

# --- D0465: the working-tree guards.rs - does the second authorising source exist, and how many keystone tests hold it
_gr = _crate_text("keel-guards")  # sprint 733: the guards are a crate, its text is every file of it
_kv = re.search(r"fn keystone_violations\((.*?)\) -> ", _gr)
fact("keystoneCharterPath", {
    "acceptedChartersFn": bool(re.search(r"^\s*fn accepted_charters\(", _gr, re.M)),
    "charterTargetsFn": bool(re.search(r"^\s*fn charter_targets\(", _gr, re.M)),
    "isAuthorisingCharterFn": bool(re.search(r"^\s*fn is_authorising_charter\(", _gr, re.M)),
    "keystoneTakesCharters": bool(_kv and re.search(r"\bcharters\s*:", _kv.group(1))),
    "keystoneTests": len(re.findall(r"#\[test\]\s*\n\s*fn keystone_\w+", _gr)),
} if _gr else None, "the second source in the guard source",
     "regex over the WORKING TREE keel-cli/src/guards.rs: `^\\s*fn accepted_charters(`, `^\\s*fn charter_targets(`, "
     "`^\\s*fn is_authorising_charter(` (line-anchored, present or not); keystoneTakesCharters = the parameter list of "
     "`fn keystone_violations(...)` names `charters:`; keystoneTests = count of `#[test]` immediately followed by "
     "`fn keystone_...`. The tree is HEAD plus the uncommitted guards.rs edit (treeUncommitted).")

# --- D0465: the marked Decisions and how each stands - the population the second source can read
_MARK_RE = re.compile(r"^\s*#(?:ProspectiveChange|SafetyChange)\s+part\s+(d\d{4})\s*:\s*Decision", re.M)
_marked = {}
for _fn in sorted(os.listdir(DEC_DIR)):
    if not _fn.endswith(".sysml"):
        continue
    _t = read(os.path.join(DEC_DIR, _fn)) or ""
    for _dn in _MARK_RE.findall(_t):
        # mirrors guards.rs acceptance_kind: a passing `part dNNNNAcceptR... : TestResult {...}` segment, and the
        # `verification dNNNNAccept : Test {...}` segment's text marked AUTO-ACCEPTED or not
        _pass = any("VerdictKind::pass" in _t[m.end():].split("}", 1)[0]
                    for m in re.finditer(r"part " + _dn + r"AcceptR\w*\s*:\s*TestResult\s*\{", _t))
        _acc = re.search(r"verification " + _dn + r"Accept\s*:\s*Test\s*\{", _t)
        _auto = bool(_acc and "AUTO-ACCEPTED" in _t[_acc.end():].split("}", 1)[0])
        _marked[_dn] = "human" if _pass and not _auto else "auto" if _pass else "none"
_MC_HOW = ("over .engine/decisions/*.sysml: a marked Decision is a line `#ProspectiveChange part dNNNN : Decision` or "
           "`#SafetyChange part dNNNN : Decision` (line-anchored); its acceptance mirrors guards.rs acceptance_kind - "
           "humanAccepted = a `part dNNNNAcceptR<n> : TestResult {` segment carrying VerdictKind::pass whose "
           "`verification dNNNNAccept : Test {` segment does NOT contain AUTO-ACCEPTED; autoAccepted = the same with "
           "AUTO-ACCEPTED; noAcceptResult = no passing AcceptR segment (proposed, held or rejected). Retired Decisions "
           "are counted like any other; the marker is what the keystone reads.")
fact("markedDecisionCensus", {
    "marked": len(_marked),
    "humanAccepted": sum(1 for k in _marked.values() if k == "human"),
    "autoAccepted": sum(1 for k in _marked.values() if k == "auto"),
    "noAcceptResult": sum(1 for k in _marked.values() if k == "none"),
} if _marked else None, "marked Decisions by how they stand", _MC_HOW)

# --- D0465: the #CharteredBy edges in the delivery records, and how many name a marked Decision
_CH_RE = re.compile(r"^\s*#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+(d\d{4})\s*;", re.M)
_ch_targets = []
for _fn in sorted(os.listdir(os.path.join(REPO, ".tracking", "delivery"))):
    if _fn.endswith(".sysml"):
        _ch_targets += _CH_RE.findall(read(os.path.join(REPO, ".tracking", "delivery", _fn)) or "")
fact("charterEdgesToMarked", {
    "edges": len(_ch_targets),
    "toMarked": sum(1 for d in _ch_targets if d in _marked),
    "toHumanAcceptedMarked": sum(1 for d in _ch_targets if _marked.get(d) == "human"),
    "toUnacceptedMarked": sum(1 for d in _ch_targets if _marked.get(d) == "none"),
} if _ch_targets else None, "charter edges and their targets",
     "over .tracking/delivery/*.sysml: every line matching `#CharteredBy dependency from <story> to dNNNN;` (the "
     "line-anchored form guards.rs charter_targets reads; an edge quoted inside a string is on no line of its own); "
     "toMarked / toHumanAcceptedMarked / toUnacceptedMarked classify the target by markedDecisionCensus above.")

# --- D0465: the live pair, quoted from sprint 693's implement gate
_s693 = read(os.path.join(REPO, ".tracking", "delivery", "sprint693_acceptedCharterAuthorisesTheLockedEdit.sysml")) or ""
_ig = re.search(r"ImplementGate\s*:\s*Test\s*\{[^}]*?procedureText\s*=\s*\"(.*?)\";", _s693, re.DOTALL)
_lp = re.search(r"LIVE: (.*?FAIL[^.]*\.)", _ig.group(1)) if _ig else None
fact("keystoneLivePair", _lp.group(1) if _lp else None, "the guard's own two verdicts",
     ".tracking/delivery/sprint693_acceptedCharterAuthorisesTheLockedEdit.sysml: the implement gate's procedureText, "
     "the substring from `LIVE: ` to the end of the sentence containing FAIL (regex `LIVE: (.*?FAIL[^.]*\\.)`) - the "
     "PASS with a human-accepted charter and the FAIL with the same sprint chartered by a proposed Decision, quoted."
     + ("" if _lp else " Not found: " + ("no implement gate procedureText" if not _ig else "no LIVE...FAIL sentence in it")))

# --- D0466: the two sweeps, quoted from the Decision's RESEARCH line, and the weighted rule read at each
_d0466 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0466-"):
        _d0466 = read(os.path.join(DEC_DIR, _fn))
_rl6 = re.search(r"// RESEARCH: (.*)", _d0466)
_research6 = _rl6.group(1) if _rl6 else ""
_blk50 = re.search(r"Default sample, 50 cases, tree (\w+), one-hop then two-hop: (.*?)\. Verdict lines", _research6)
_blk100 = re.search(r"100 cases \(--cases 100\), tree (\w+) \([^)]*\), one-hop then two-hop: (.*?)\. Verdict lines", _research6)
_ROW_RE = re.compile(
    r"([0-9.]+) -> (?:hits )?(\d+)/(\d+) median(?: position)? (\d+) top-3 (\d+)/(\d+)(?: (?:mean )?rows (\d+))?, "
    r"then (\d+)/(\d+) median(?: position)? (\d+) top-3 (\d+)/(\d+)(?: (?:mean )?rows (\d+))?")


def _sample(block):
    rows = {}
    for m in _ROW_RE.finditer(block or ""):
        rows[m.group(1)] = {
            "hop1": {"hits": int(m.group(2)), "n": int(m.group(3)), "median": int(m.group(4)),
                     "top3": int(m.group(5)), "top3Of": int(m.group(6)),
                     "rows": int(m.group(7)) if m.group(7) else None},
            "hop2": {"hits": int(m.group(8)), "n": int(m.group(9)), "median": int(m.group(10)),
                     "top3": int(m.group(11)), "top3Of": int(m.group(12)),
                     "rows": int(m.group(13)) if m.group(13) else None},
        }
    return rows


def _weighted_rule(rows):
    """D0464 option B as accepted: the highest two-hop hit count among settings whose one-hop hits and mean rows
    hold against setting 0 and whose one-hop median moves by at most one - a move of one only where the two-hop
    arm gains at least three hits. Rows are compared only where both readings state them."""
    if "0" not in rows:
        return {"admissible": [], "refused": list(rows), "winner": None}
    off = rows["0"]
    adm, why = [], {}
    for s, r in rows.items():
        h1, h2 = r["hop1"], off["hop1"]
        dmed = h1["median"] - h2["median"]
        gain = r["hop2"]["hits"] - off["hop2"]["hits"]
        hits_ok = h1["hits"] >= h2["hits"]
        rows_ok = h1["rows"] is None or h2["rows"] is None or h1["rows"] == h2["rows"]
        med_ok = dmed == 0 or (abs(dmed) == 1 and gain >= 3)
        why[s] = {"medianMove": dmed, "twoHopGain": gain, "hitsHeld": hits_ok, "rowsHeld": rows_ok, "medianWithinRule": med_ok}
        if hits_ok and rows_ok and med_ok:
            adm.append(s)
    best = max((rows[s]["hop2"]["hits"] for s in adm), default=None)
    winners = [s for s in adm if rows[s]["hop2"]["hits"] == best]
    return {"admissible": sorted(adm, key=float), "refused": sorted((s for s in rows if s not in adm), key=float),
            "winner": winners[0] if len(winners) == 1 else None, "tied": winners if len(winners) > 1 else [],
            "readings": why}


_r50, _r100 = _sample(_blk50.group(2) if _blk50 else ""), _sample(_blk100.group(2) if _blk100 else "")
_hand6 = {}
for m in re.finditer(r"([0-9.]+) bar (MET|NOT MET) (\d)/(\d), (?:the rebase question's )?d0129 (?:at position (\d+)|absent)", _research6):
    _hand6[m.group(1)] = {"bar": m.group(2), "reachable": int(m.group(3)), "of": int(m.group(4)),
                          "d0129Position": int(m.group(5)) if m.group(5) else None}
_kn6 = read(_mh("view/knowledge") or "") or ""
_const6 = re.search(r"const DOMINANCE: f64 = ([0-9.]+);", _kn6)
SS_HOW = ("regex over the `// RESEARCH:` line of .engine/decisions/0466-*.sysml: the 50-case block is the text between "
          "`Default sample, 50 cases, tree <sha>, one-hop then two-hop: ` and `. Verdict lines`; the 100-case block the "
          "text between `100 cases (--cases 100), tree <sha> (...), one-hop then two-hop: ` and `. Verdict lines`; in each, "
          "one row per `<setting> -> [hits ]h/N median[ position] m top-3 t/h[ [mean ]rows r], then h/N median m top-3 t/h[ rows r]` "
          "(rows absent where the line states none). handSet = each `<setting> bar MET|NOT MET k/n, d0129 at position p|absent`. "
          "ruleAt50 / ruleAt100 apply D0464 option B IN CODE to those rows: admissible when one-hop hits >= setting 0's, "
          "one-hop mean rows equal where both stated, and the one-hop median moves by 0, or by exactly 1 where the two-hop arm "
          "gains >= 3 hits; winner = the admissible setting with the most two-hop hits (null on a tie). "
          "`constant` = regex `const DOMINANCE: f64 = N;` over view/knowledge.rs (module_home resolves the crate). Re-runnable with "
          ".engine/tools/recall_bench.py --sweep [--cases 100].")
fact("sampleSplit", {
    "at50": {"tree": _blk50.group(1) if _blk50 else None, "rows": _r50, "rule": _weighted_rule(_r50)},
    "at100": {"tree": _blk100.group(1) if _blk100 else None, "rows": _r100, "rule": _weighted_rule(_r100)},
    "handSet": _hand6,
    "constant": float(_const6.group(1)) if _const6 else None,
} if _r50 and _r100 and _hand6 else None, "the two sweeps and the rule read at each", SS_HOW)

# --- D0461 / D0467: the human's retro note, verbatim from the Statement that holds it
_st = read(os.path.join(REPO, ".tracking", "intake", "intake-2026-09-13.sysml")) or ""
_st122 = re.search(r"part st122\s*:\s*Statement\s*\{(.*?)\n\s*\}", _st, re.DOTALL)
_st_text = re.search(r':>>\s*text\s*=\s*"(.*?)";', _st122.group(1), re.DOTALL) if _st122 else None
_st_by = re.search(r':>>\s*saidBy\s*=\s*"([^"]*)";\s*:>>\s*saidAt\s*=\s*"([^"]*)";', _st122.group(1)) if _st122 else None
fact("retroNoteStatement", {
    "text": _st_text.group(1).encode("utf-8").decode("unicode_escape") if _st_text else None,
    "saidBy": _st_by.group(1) if _st_by else None, "saidAt": _st_by.group(2) if _st_by else None,
} if _st_text else None, "their words on retros, verbatim",
     ".tracking/intake/intake-2026-09-13.sysml: the `text`, `saidBy` and `saidAt` fields of `part st122 : Statement` "
     "(the note written on the nineteenth-publish copy-out), quoted; the SysML `\\n` escapes are decoded, nothing else changes.")

# --- D0467: what retros in the tree discharge themselves with - an Issue, a task, or a Decision only
_rd = {"retros": 0, "namingIssue": 0, "namingTask": 0, "namingDecision": 0, "decisionOnly": 0, "namingNone": 0}
for _fn in os.listdir(os.path.join(REPO, ".tracking", "delivery")):
    if not _fn.endswith(".sysml"):
        continue
    _t = read(os.path.join(REPO, ".tracking", "delivery", _fn)) or ""
    for _c in re.split(r"\n(?=\s*(?:verification|part)\s)", _t):
        if not re.match(r"\s*verification\s+\w*Retro\w*\s*:\s*Test", _c):
            continue
        _rd["retros"] += 1
        _iss = bool(re.search(r"(?<![A-Za-z0-9])issue\d+(?![A-Za-z0-9])", _c))
        _tsk = bool(re.search(r"(?<![A-Za-z0-9])dc[A-Z][A-Za-z0-9]+", _c))
        _dec = bool(re.search(r"(?<![A-Za-z0-9])[dD]0\d{3}(?![A-Za-z0-9])", _c))
        _rd["namingIssue"] += _iss
        _rd["namingTask"] += _tsk
        _rd["namingDecision"] += _dec
        _rd["decisionOnly"] += (_dec and not _iss and not _tsk)
        _rd["namingNone"] += (not _dec and not _iss and not _tsk)
fact("retroDischargeCensus", _rd if _rd["retros"] else None, "retro gates by the item form they name",
     "over .tracking/delivery/*.sysml: every record starting at a line `verification <name>Retro<...> : Test` (read to the "
     "next `verification`/`part` line, as guards.rs retro_texts does); namingIssue = text carries `issue` + digits at a word "
     "boundary; namingTask = `dc` + an uppercase letter at a word boundary (the task form named_items reads); namingDecision = "
     "`d0`/`D0` + three digits at a word boundary (the retro template's own citation of the retro rule counts, as the guard "
     "counts it); decisionOnly = namingDecision and NEITHER of the other two; namingNone = none of the three. A retro can "
     "count in both namingIssue and namingTask.")

# ================================================================ 22. the twenty-first publish's ask
# The two host-cost indicators and their thresholds (D0468): the live values and trigger state from the indicators
# lens, the two numbers from the triggers contract, the last-25 stop-fire distribution and its critical path from the
# ledger, the anchors from the Decision's own text, and the tracked Issue the trigger surfaces. Nothing here is retyped
# from the Decision's RESEARCH line: the ledger is re-read, so a fire that landed after the Decision moves the number.

# --- D0468: the indicators lens - the two live values and whether each trigger is crossed
_IND_HOW = ("`" + KEEL + " show indicators .` - JSON; `indicators[]` rows with `indicator` in {hookLatencyIndicator, "
            "gitFactsSizeIndicator}: `latest` parsed as a float, `trigger.threshold` quoted, `trigger.crossed` as read; "
            "`triggered` = the indicator's name is in the top-level `triggered[]` list.")
ok, out = run([KEEL, "show", "indicators", "."], timeout=120)
_ind = as_json(out) if ok else None
_host = {}
if _ind and isinstance(_ind.get("indicators"), list):
    _trig_names = {t.get("indicator") for t in _ind.get("triggered") or []}
    for _row in _ind["indicators"]:
        if _row.get("indicator") in ("hookLatencyIndicator", "gitFactsSizeIndicator"):
            _tr = _row.get("trigger") or {}
            try:
                _lat = float(_row["latest"]) if _row.get("latest") is not None else None
            except (TypeError, ValueError):
                _lat = None
            _host[_row["indicator"]] = {"latest": _lat, "threshold": _tr.get("threshold"),
                                        "crossed": _tr.get("crossed"), "triggered": _row["indicator"] in _trig_names}
fact("hostCostIndicators", _host if len(_host) == 2 else None, "the two indicators as the lens reads them",
     _IND_HOW + ("" if len(_host) == 2 else " Not both present: " + (out[:200] if ok else out)))

# --- D0468: the two thresholds as the contract declares them
_trg = read(os.path.join(REPO, ".engine", "contracts", "indicator-triggers.toml")) or ""
_thr = {}
for _sec in re.finditer(r"^\[(\w+)\]\s*\n(.*?)(?=^\[|\Z)", _trg, re.M | re.DOTALL):
    if _sec.group(1) in ("hookLatencyIndicator", "gitFactsSizeIndicator"):
        _ab = re.search(r"^above\s*=\s*([0-9.]+)", _sec.group(2), re.M)
        _sf = re.search(r'^surfaces\s*=\s*"(.*?)"', _sec.group(2), re.M)
        _thr[_sec.group(1)] = {"above": float(_ab.group(1)) if _ab else None,
                               "surfaces": _sf.group(1) if _sf else None}
fact("hostCostThresholds", _thr if len(_thr) == 2 else None, "the declared triggers",
     "regex over .engine/contracts/indicator-triggers.toml: in the `[hookLatencyIndicator]` and `[gitFactsSizeIndicator]` "
     "sections (a section runs from its `[name]` line to the next `[`), `above = N` parsed as a float and the `surfaces` "
     "string quoted. Comment lines are not read - the anchors come from the Decision (hostCostAnchors).")

# --- D0468: the last 25 stop fires - the distribution the hook indicator binds, re-read from the ledger
_SLOW_MS = 3000
_ledger = read(os.path.join(REPO, ".keel", "metrics", "hooks.jsonl")) or ""
_stops = []
for _ln in _ledger.splitlines():
    _e = as_json(_ln.strip()) if _ln.strip() else None
    if isinstance(_e, dict) and _e.get("event") == "stop" and isinstance(_e.get("ms"), (int, float)):
        _stops.append(_e)
_last = _stops[-25:]
_ms = sorted(int(e["ms"]) for e in _last)
_slow = [e for e in _last if e["ms"] >= _SLOW_MS]
_crit = {}
for _e in _slow:
    _names = [str(p.get("name", "")) for p in (_e.get("phases") or []) if isinstance(p, dict)]
    _cp = next((n for n in _names if n.endswith("(critical path)")), None)
    _key = _cp.replace(" (critical path)", "") if _cp else "none attributed"
    _crit[_key] = _crit.get(_key, 0) + 1
_ds_ms = [int(p["ms"]) for e in _slow for p in (e.get("phases") or [])
          if isinstance(p, dict) and p.get("name") == "guard:decision-scaffolding (critical path)" and isinstance(p.get("ms"), (int, float))]


def _rank(sorted_ms, q):
    """Nearest-rank percentile: the value at ceil(q * n), 1-based - the rule D0468 names for the indicator."""
    import math
    return sorted_ms[max(0, math.ceil(q * len(sorted_ms)) - 1)] if sorted_ms else None


fact("stopFireTail", {
    "fires": len(_last), "ledgerStopFires": len(_stops),
    "p90": _rank(_ms, 0.9), "median": _rank(_ms, 0.5), "min": _ms[0] if _ms else None, "max": _ms[-1] if _ms else None,
    "slowFireMs": _SLOW_MS, "pastReceipt": len(_slow), "fromReceipt": len(_last) - len(_slow),
    "receiptMs": [int(e["ms"]) for e in _last if e["ms"] < _SLOW_MS],
    "slowMs": sorted(int(e["ms"]) for e in _slow),
    "criticalPath": _crit,
    "decisionScaffoldingLed": _crit.get("guard:decision-scaffolding", 0),
    "decisionScaffoldingMs": {"min": min(_ds_ms) if _ds_ms else None, "max": max(_ds_ms) if _ds_ms else None},
    "aboveTenSeconds": sum(1 for m in _ms if m > 10000),
    "lastTs": _last[-1].get("ts") if _last else None,
} if len(_last) == 25 else None, "the last 25 stop-hook fires",
     ".keel/metrics/hooks.jsonl, one JSON object per line: the last 25 lines with `event` == \"stop\" and a numeric `ms`. "
     f"p90 / median by nearest rank (the value at ceil(q*25), the rule the indicator binds); pastReceipt = fires with ms >= {_SLOW_MS} "
     "(D0414 SLOW_FIRE_MS - a fire that answers from the green receipt runs no guard); fromReceipt = the rest. criticalPath = "
     "for each slow fire the one `phases[].name` ending `(critical path)`, counted by name without the suffix; "
     "decisionScaffoldingLed = that count for guard:decision-scaffolding; decisionScaffoldingMs = min/max of that phase's ms "
     "over the slow fires; aboveTenSeconds = fires with ms > 10000. Read at run time, so a fire after the Decision's RESEARCH "
     "line moves these numbers and the page carries the ledger's, not the Decision's.")

# --- D0468: the cache file as it sits on disk
_gf = os.path.join(REPO, ".keel", "cache", "git-facts.toml")
fact("gitFactsCacheBytes", os.path.getsize(_gf) if os.path.exists(_gf) else 0, "bytes",
     "os.path.getsize(.keel/cache/git-facts.toml), 0 when absent - the rule the indicator's metric states.")

# --- D0468: the anchors, quoted from the Decision's own text, and the Decision's fields
_d0468 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0468-"):
        _d0468 = read(os.path.join(DEC_DIR, _fn)) or ""


def _field(name):
    m = re.search(r':>>\s*' + name + r'\s*=\s*"(.*?)"\s*;', _d0468, re.DOTALL)
    return m.group(1) if m else None


_rat = _field("rationale") or ""
_ctx = _field("context") or ""
_dec8 = _field("decision") or ""
_a_med = re.search(r"known positive was a (\d+) ms MEDIAN", _rat)
_a_single = re.search(r"an (\d+) ms single fire", _rat)
_a_slow = re.search(r"one fire slow at (\d+) ms", _rat)
_a_trig = re.search(r"triggers on the tree that declares it \((\d+) > (\d+)\)", _rat)
_a_all = re.search(r"\((\d+) fires, p99 ([\d ]+)\)", _rat)
_rl8 = re.search(r"// RESEARCH: (.*)", _d0468)
_res8 = _rl8.group(1) if _rl8 else ""
_a_cache = re.search(r"git-facts\.toml ([\d ]+) bytes after", _res8)
_a_found = re.search(r"([\d.]+) MB with ([\d.]+) MB dead", _res8)
_a_slowset = re.search(r"(\d+) of the (\d+) fires past the receipt", _res8)
_a_p90 = re.search(r"p90 (\d+) ms, median (\d+)", _res8)
fact("hostCostAnchors", {
    "issue442MedianMs": int(_a_med.group(1)) if _a_med else None,
    "issue441SingleFireMs": int(_a_single.group(1)) if _a_single else None,
    "slowFireMs": int(_a_slow.group(1)) if _a_slow else None,
    "atDeclaration": {"p90": int(_a_trig.group(1)), "threshold": int(_a_trig.group(2))} if _a_trig else None,
    "allTime": {"fires": int(_a_all.group(1)), "p99": int(_a_all.group(2).replace(" ", ""))} if _a_all else None,
    "researchP90": int(_a_p90.group(1)) if _a_p90 else None,
    "researchMedian": int(_a_p90.group(2)) if _a_p90 else None,
    "researchScaffoldingLed": {"led": int(_a_slowset.group(1)), "of": int(_a_slowset.group(2))} if _a_slowset else None,
    "cacheBytesAfterCut": int(_a_cache.group(1).replace(" ", "")) if _a_cache else None,
    "issue440Mb": float(_a_found.group(1)) if _a_found else None,
    "issue440DeadMb": float(_a_found.group(2)) if _a_found else None,
    "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0468) or [None, None])[1],
    "createdAt": _field("createdAt"),
    "marker": "#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+d0468", _d0468, re.M) else None,
    "notAFork": _dec8.lstrip().startswith("NOT A FORK"),
    "gatesNothing": "gates nothing" in _dec8,
    "keystonePath": "keystone-locked path" in _ctx,
} if _d0468 else None, "the Decision's own anchors and fields",
     "regex over .engine/decisions/0468-*.sysml: from the `rationale` field `known positive was a N ms MEDIAN`, `an N ms single "
     "fire`, `one fire slow at N ms`, `triggers on the tree that declares it (N > M)`, `(N fires, p99 N)`; from the "
     "`// RESEARCH:` line `p90 N ms, median N`, `N of the M fires past the receipt`, `git-facts.toml N bytes after`, `N MB with "
     "N MB dead`; status from `DecisionStatus::x`; marker = a line `#ProspectiveChange part d0468`; notAFork = the `decision` "
     "field opens NOT A FORK; gatesNothing / keystonePath = those phrases in the decision / context fields. Digit groups "
     "written with spaces are joined.")

# --- D0468: the Issue the hook trigger surfaces, and its resolver edge
_iss = read(os.path.join(REPO, ".tracking", "issues-claudeFable5.sysml")) or ""
_i520 = re.search(r"part issue520\s*:\s*Issue\s*\{(.*?)\n\s*\}", _iss, re.DOTALL)
_i520_title = re.search(r':>>\s*title\s*=\s*"(.*?)";', _i520.group(1), re.DOTALL) if _i520 else None
_i520_res = re.search(r"#Resolves\s+dependency\s+from\s+(\w+)\s+to\s+issue520\s*;", _iss)
fact("hookCriticalPathIssue", {
    "title": _i520_title.group(1) if _i520_title else None,
    "resolver": _i520_res.group(1) if _i520_res else None,
    "namedBySurfaces": "issue520" in (_thr.get("hookLatencyIndicator") or {}).get("surfaces", "") if _thr else None,
} if _i520_title else None, "the tracked Issue the trigger points at",
     ".tracking/issues-claudeFable5.sysml: the `title` of `part issue520 : Issue`, the `from` of `#Resolves dependency from "
     "<task> to issue520;`, and whether the hookLatencyIndicator `surfaces` line in indicator-triggers.toml names issue520.")

# ================================================================ 23. the twenty-second publish's ask
# The measured-cost token (D0469): the pending set as the authority lens computes it, the Decision's own fields read
# from its file, the census of marked Decisions the guard clause would read on acceptance - mirrored in python from
# guards.rs unmeasured_path_decisions so the page states a live consequence it computed, not one it copied - the
# MEASURED figures parsed from the rationale, the 528 s origin from the Issue's text, and the sprint record.

# --- D0469: the pending set from the authority queue - the assessment the surfacing skill reads (D0359)
_AQ_HOW = ("`" + KEEL + " show authority-queue .` - JSON; `awaiting[]` rows whose `kind` is decisionAcceptance, their `item` "
           "in the lens's order. Rows of other kinds (findingDisposition) are counted, not listed.")
ok, out = run([KEEL, "show", "authority-queue", "."], timeout=120)
_aq = as_json(out) if ok else None
if _aq and isinstance(_aq.get("awaiting"), list):
    _aq_rows = [a for a in _aq["awaiting"] if isinstance(a, dict)]
    fact("authorityQueuePending", {
        "decisions": [a.get("item") for a in _aq_rows if a.get("kind") == "decisionAcceptance"],
        "otherKinds": len([a for a in _aq_rows if a.get("kind") != "decisionAcceptance"]),
        "asOf": _aq.get("asOf"),
    }, "Decisions awaiting the human's acceptance", _AQ_HOW)
else:
    fact("authorityQueuePending", None, "Decisions awaiting the human's acceptance",
         _AQ_HOW + " Command failed or not JSON: " + (out[:200] if ok else out))

# --- D0469: the Decision's own fields, read from its file the way the guard reads them
_d0469 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0469-"):
        _d0469 = read(os.path.join(DEC_DIR, _fn)) or ""
_PATH_WORDS = ("land", "push", "refuse", "gate")


def _guard_field(text, dname, key):
    """guards.rs unmeasured_path_decisions' field read: the body from `part dNNNN : Decision` to the first `\\n    }`,
    the value from `key = "` to the next double quote."""
    s = text.find("part " + dname + " : Decision")
    if s < 0:
        return ""
    body = text[s:]
    e = body.find("\n    }")
    body = body if e < 0 else body[:e]
    k = body.find(key + ' = "')
    if k < 0:
        return ""
    v = body[k + len(key) + 4:]
    q = v.find('"')
    return v if q < 0 else v[:q]


def _path_words(decision_text):
    """The whole words of PATH_WORDS in the lower-cased text split on every non-alphanumeric character."""
    return sorted({w for w in re.split(r"[^a-z0-9]", decision_text.lower()) if w in _PATH_WORDS})


def _acceptance_kind(text, dname):
    """guards.rs acceptance_kind: None with no passing AcceptR segment; 'auto' when the Accept Test says AUTO-ACCEPTED."""
    if not any("VerdictKind::pass" in text[m.end():].split("}", 1)[0]
               for m in re.finditer(r"part " + dname + r"AcceptR", text)):
        return None
    _acc = text.find("verification " + dname + "Accept : Test")
    return "auto" if _acc >= 0 and "AUTO-ACCEPTED" in text[_acc:].split("}", 1)[0] else "human"


_dec9 = _guard_field(_d0469, "d0469", "decision")
_rat9 = _guard_field(_d0469, "d0469", "rationale")
_con9 = _guard_field(_d0469, "d0469", "consequences")
_ctx9 = _guard_field(_d0469, "d0469", "context")
fact("measuredCostDecision", {
    "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0469) or [None, None])[1],
    "createdAt": _guard_field(_d0469, "d0469", "createdAt") or None,
    "marker": "#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+d0469\s*:", _d0469, re.M) else None,
    "acceptance": _acceptance_kind(_d0469, "d0469"),
    "notAFork": _dec9.lstrip().startswith("NOT A FORK"),
    "decision": _dec9, "rationale": _rat9, "consequences": _con9, "context": _ctx9,
    "carriesMeasuredToken": "MEASURED:" in _rat9,
    "ownPathWords": _path_words(_dec9),
    "isWarning": "WARNS" in _dec9 or "WARNING" in _rat9,
    "namesIssue444": "issue444" in _ctx9,
    "dischargesObligation": (re.search(r"discharges (obligation\w+)", _con9) or [None, None])[1],
    "processChangeWords": bool(re.search(r"process-change", _dec9 + _rat9 + _con9 + _ctx9)),
} if _d0469 else None, "the Decision's fields as the guard reads them",
     "regex over .engine/decisions/0469-*.sysml: status from `DecisionStatus::x`; marker = a line `#ProspectiveChange part "
     "d0469 :`; acceptance mirrors guards.rs acceptance_kind (None = no passing AcceptR segment); decision / rationale / "
     "consequences / context read as guards.rs unmeasured_path_decisions reads a field - from `key = \"` to the next quote "
     "inside the body that runs from `part d0469 : Decision` to the first `\\n    }`; carriesMeasuredToken = the literal "
     "`MEASURED:` in the rationale; ownPathWords = the four PATH_WORDS found as whole words in the lower-cased decision text "
     "split on every non-alphanumeric character (the guard's own split); isWarning = `WARNS` in the decision or `WARNING` "
     "in the rationale; dischargesObligation = the word after `discharges ` in consequences.")

# --- D0469: the marked Decisions the clause reads, and the set it would name on acceptance - the guard mirrored
_pw_rows = []
_MARK_ANY = re.compile(r"^\s*#(ProspectiveChange|SafetyChange)\s+part\s+(d\d{4})\s*:\s*Decision", re.M)
for _fn in sorted(os.listdir(DEC_DIR)):
    if not _fn.endswith(".sysml"):
        continue
    _t = read(os.path.join(DEC_DIR, _fn)) or ""
    for _mk, _dn in _MARK_ANY.findall(_t):
        _ak = _acceptance_kind(_t, _dn)
        if _ak is not None:
            continue                       # accepted, by a human or by consent: the human's word already, not read
        _dtxt = _guard_field(_t, _dn, "decision")
        _rtxt = _guard_field(_t, _dn, "rationale")
        _w = _path_words(_dtxt)
        _pw_rows.append({
            "decision": _dn, "marker": "#" + _mk,
            "status": (re.search(r"part " + _dn + r"\s*:\s*Decision.*?status\s*=\s*DecisionStatus::(\w+)", _t, re.DOTALL) or [None, None])[1],
            "pathWords": _w, "carriesMeasuredToken": "MEASURED:" in _rtxt,
            "wouldWarn": _mk == "ProspectiveChange" and bool(_w) and "MEASURED:" not in _rtxt,
        })
fact("pathWordCensus", {
    "unaccepted": len(_pw_rows),
    "prospective": sum(1 for r in _pw_rows if r["marker"] == "#ProspectiveChange"),
    "safety": sum(1 for r in _pw_rows if r["marker"] == "#SafetyChange"),
    "byStatus": {s: sum(1 for r in _pw_rows if r["status"] == s) for s in sorted({r["status"] for r in _pw_rows}, key=str)},
    "namingAWord": [r["decision"] for r in _pw_rows if r["pathWords"]],
    "withToken": [r["decision"] for r in _pw_rows if r["carriesMeasuredToken"]],
    "wouldWarn": sorted(r["decision"] for r in _pw_rows if r["wouldWarn"]),
    "rows": _pw_rows,
    "armed": _acceptance_kind(_d0469, "d0469") == "human",
} if _pw_rows else None, "unaccepted marked Decisions and what the clause would name",
     "over .engine/decisions/*.sysml, mirroring guards.rs unmeasured_path_decisions + acceptance_kind: every line "
     "`#ProspectiveChange part dNNNN : Decision` or `#SafetyChange part dNNNN : Decision`; a Decision with a passing "
     "`part dNNNNAcceptR` segment is dropped (accepted by a human or by consent); for the rest the `decision` field is "
     "lower-cased, split on every non-alphanumeric character and the whole words land/push/refuse/gate kept; "
     "carriesMeasuredToken = `MEASURED:` in the `rationale` field; wouldWarn = #ProspectiveChange (the guard reads that "
     "marker only) AND at least one word AND no token. A REJECTED Decision has no passing AcceptR and is read like a "
     "proposed one, as the guard reads it. armed = d0469 itself carries a HUMAN acceptance (D0337: the clause is inert "
     "until then).")

# --- D0469: the MEASURED figures, parsed from the rationale - the cost of the inert clause on this host
_m_before = re.search(r"before the clause ([\d.]+) s median \(([\d.]+) / ([\d.]+) / ([\d.]+)\)", _rat9)
_m_with = re.search(r"holding it ([\d.]+) s median \(([\d.]+) / ([\d.]+) / ([\d.]+)\)", _rat9)
_m_tree = re.search(r"working tree at (\w+) plus this change, host (\w+), (\d{4}-\d{2}-\d{2}), (\w+) runs each", _rat9)
_m_files = re.search(r"\((\d+) files on this tree\)", _rat9)
fact("measuredCostFigures", {
    "run": "keel gate guard --no-receipt" if "keel gate guard --no-receipt" in _rat9 else None,
    "tree": _m_tree.group(1) if _m_tree else None,
    "host": _m_tree.group(2) if _m_tree else None,
    "date": _m_tree.group(3) if _m_tree else None,
    "runsEach": _m_tree.group(4) if _m_tree else None,
    "beforeMedianS": float(_m_before.group(1)) if _m_before else None,
    "beforeRunsS": [float(_m_before.group(i)) for i in (2, 3, 4)] if _m_before else None,
    "withMedianS": float(_m_with.group(1)) if _m_with else None,
    "withRunsS": [float(_m_with.group(i)) for i in (2, 3, 4)] if _m_with else None,
    "markedFilesRead": int(_m_files.group(1)) if _m_files else None,
    "armedPathMeasured": "not measurable before the acceptance" not in _rat9,
} if _m_before and _m_with else None, "the Decision's own MEASURED: figures",
     "regex over the `rationale` field of .engine/decisions/0469-*.sysml after its `MEASURED:` token: `before the clause N s "
     "median (a / b / c)`, `holding it N s median (a / b / c)`, `working tree at <sha> plus this change, host <id>, <date>, "
     "<word> runs each`, `(N files on this tree)`; armedPathMeasured = False when the rationale says the armed path is "
     "`not measurable before the acceptance`. Nothing is re-timed here: the figures are the Decision's, quoted.")

# --- D0469: the origin - the Issue whose text holds the 528 s, and the Decision it corrected
_iss_o = read(os.path.join(REPO, ".tracking", "issues-claudeOpus5.sysml")) or ""
_i444 = re.search(r"part issue444\s*:\s*Issue\s*\{(.*?)\n\s*\}", _iss_o, re.DOTALL)
_i444_body = _i444.group(1) if _i444 else ""
_i444_title = re.search(r':>>\s*title\s*=\s*"(.*?)";', _i444_body, re.DOTALL)
_i444_desc = re.search(r':>>\s*description\s*=\s*"(.*?)";', _i444_body, re.DOTALL)
_i444_d = _i444_desc.group(1) if _i444_desc else ""
_o_wall = re.search(r"named (\d+) integration binaries plus the lib: (\d+) s wall", _i444_d)
_o_suite = re.search(r"full suite's (\d+) s at (\w+)", _i444_d)
_o_split = re.search(r"compile (\d+) s, tests (\d+) s, (\d+) passed", _i444_d)
_o_fixture = re.search(r"runs in about (\w+) seconds", _i444_d)
_o_pct = re.search(r"(\d+) percent of the suite", _i444_title.group(1) if _i444_title else "")
_i444_res = re.search(r"#Resolves\s+dependency\s+from\s+(\w+)\s+to\s+issue444\s*;", _iss_o)
_d0421 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0421-"):
        _d0421 = read(os.path.join(DEC_DIR, _fn)) or ""
fact("landGateFirstLiveSet", {
    "title": _i444_title.group(1) if _i444_title else None,
    "liveSetSeconds": int(_o_wall.group(2)) if _o_wall else None,
    "liveSetBinaries": int(_o_wall.group(1)) if _o_wall else None,
    "compileSeconds": int(_o_split.group(1)) if _o_split else None,
    "testSeconds": int(_o_split.group(2)) if _o_split else None,
    "testsPassed": int(_o_split.group(3)) if _o_split else None,
    "fullSuiteSeconds": int(_o_suite.group(1)) if _o_suite else None,
    "fullSuiteTree": _o_suite.group(2) if _o_suite else None,
    "fixtureWordSeconds": _o_fixture.group(1) if _o_fixture else None,
    "percentOfSuite": int(_o_pct.group(1)) if _o_pct else None,
    "resolver": _i444_res.group(1) if _i444_res else None,
    "d0421Status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0421) or [None, None])[1],
    "d0421Acceptance": _acceptance_kind(_d0421, "d0421") if _d0421 else None,
    "d0421PathWords": _path_words(_guard_field(_d0421, "d0421", "decision")) if _d0421 else None,
    "d0421CarriesToken": "MEASURED:" in _guard_field(_d0421, "d0421", "rationale") if _d0421 else None,
    "d0421CarriesMeasuredWord": "MEASURED" in _guard_field(_d0421, "d0421", "rationale") if _d0421 else None,
} if _i444_title and _o_wall else None, "the Issue's figures and the corrected Decision's standing",
     ".tracking/issues-claudeOpus5.sysml: `part issue444 : Issue { ... }` - title; from description `named N integration "
     "binaries plus the lib: N s wall`, `compile N s, tests N s, N passed`, `full suite's N s at <sha>`, `runs in about <word> "
     "seconds`; from the title `N percent of the suite`; resolver = the `from` of `#Resolves dependency from <task> to "
     "issue444;`. d0421* from .engine/decisions/0421-*.sysml read the way pathWordCensus reads a Decision: status, "
     "acceptance kind, the path words in its decision text, and whether its rationale carries the literal `MEASURED:` "
     "(d0421CarriesToken) or the bare word MEASURED (d0421CarriesMeasuredWord) - the corrected text writes the word without "
     "the colon; accepted, so the guard does not read it either way.")

# --- D0469: the sprint record that delivered the clause
_s697p = os.path.join(REPO, ".tracking", "delivery", "sprint697_gateDecisionCarriesItsMeasuredCost.sysml")
_s697 = read(_s697p) if os.path.exists(_s697p) else None
if _s697 is not None:
    _res = re.findall(r"part \w+\s*:\s*TestResult\s*\{[^}]*?outcome\s*=\s*VerdictKind::(\w+)", _s697)
    _trio = re.search(r"unmeasured_path_tests -> (\d+) passed (\d+) failed", _s697)
    _mirror = re.search(r"on acceptance the guard would name (d\d{4}) \((\w+)\) and no other proposed marked Decision", _s697)
    fact("measuredCostSprint", {
        "exists": True, "results": len(_res),
        "byOutcome": {o: _res.count(o) for o in sorted(set(_res))},
        "ranReceipts": len(re.findall(r"// RAN:", _s697)),
        "trioPassed": int(_trio.group(1)) if _trio else None, "trioFailed": int(_trio.group(2)) if _trio else None,
        "liveMirror": {"names": _mirror.group(1), "word": _mirror.group(2)} if _mirror else None,
        "chartersD0469": bool(re.search(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+d0469\s*;", _s697)),
    }, "the delivery record",
         ".tracking/delivery/sprint697_gateDecisionCarriesItsMeasuredCost.sysml: results = `part x : TestResult {` segments and "
         "their `VerdictKind::x`; ranReceipts = `// RAN:` lines; from the receipts `unmeasured_path_tests -> N passed N failed` "
         "and `on acceptance the guard would name dNNNN (word) and no other proposed marked Decision`; chartersD0469 = a "
         "`#CharteredBy dependency from <story> to d0469;` line.")
else:
    fact("measuredCostSprint", {"exists": False}, "the delivery record", "the file " + _s697p + " does not exist")

# ================================================================ 24. the reject verdict's human-judgment layers (D0470)
# --- D0470: the Decision's own fields, read from its file the way the guard reads them
_d0470 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0470-"):
        _d0470 = read(os.path.join(DEC_DIR, _fn)) or ""
_dec0 = _guard_field(_d0470, "d0470", "decision")
_rat0 = _guard_field(_d0470, "d0470", "rationale")
_con0 = _guard_field(_d0470, "d0470", "consequences")
_ctx0 = _guard_field(_d0470, "d0470", "context")
fact("rejectVerdictDecision", {
    "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0470) or [None, None])[1],
    "createdAt": _guard_field(_d0470, "d0470", "createdAt") or None,
    "marker": "#SafetyChange" if re.search(r"^\s*#SafetyChange\s+part\s+d0470\s*:", _d0470, re.M) else None,
    "acceptance": _acceptance_kind(_d0470, "d0470"),
    "notAFork": "NOT A FORK" in _dec0,
    "decision": _dec0, "rationale": _rat0, "consequences": _con0, "context": _ctx0,
    "namesIssue526": "issue526" in _ctx0 or "issue526" in _con0,
    "namesTheTwoRejected": "d0376" in _ctx0 and "d0463" in _ctx0,
    "safetyChangeWords": bool(re.search(r"safety-change", _dec0 + _rat0 + _con0 + _ctx0)),
} if _d0470 else None, "the Decision's fields as the guard reads them",
     "regex over .engine/decisions/0470-*.sysml: status from `DecisionStatus::x`; marker = a line `#SafetyChange part d0470 :`; "
     "acceptance mirrors guards.rs acceptance_kind (None = no passing AcceptR segment); the four fields read as guards.rs "
     "unmeasured_path_decisions reads a field; notAFork = the literal `NOT A FORK` in the decision text; namesIssue526 / "
     "namesTheTwoRejected = those names in the context (or consequences); safetyChangeWords = `safety-change` anywhere in the four.")

# --- D0470: the three layers in source - the refusal inside each verdict's lock, and the two command lists
_wr = read(os.path.join(REPO, "members", "keel-write", "src", "write.rs")) or ""
_cs = read(_mh("view/control_structure") or "") or ""
_gr = _crate_text("keel-guards")  # sprint 733: the guards are a crate, its text is every file of it


def _fn_body(text, name):
    """The text from `fn name(` to the next line that is exactly `}` at column 0."""
    s = text.find("fn " + name + "(")
    if s < 0:
        return ""
    e = text.find("\n}\n", s)
    return text[s:] if e < 0 else text[s:e]


def _refuses_before_read(body):
    """True when refuse_ai_judgment( sits before the first read_to_string( inside the function body."""
    r, f = body.find("refuse_ai_judgment("), body.find("read_to_string(")
    return r >= 0 and (f < 0 or r < f)


def _const_list(text, name):
    m = re.search(r"const " + name + r"\s*:\s*\[&str;\s*(\d+)\]\s*=\s*\[([^\]]*)\]", text)
    if not m:
        return None
    return {"declared": int(m.group(1)), "members": re.findall(r'"([^"]+)"', m.group(2))}


_acc_body, _rej_body = _fn_body(_wr, "accept_decision_locked"), _fn_body(_wr, "reject_decision_locked")
_wl, _cl = _const_list(_wr, "HUMAN_ONLY_WRITE_COMMANDS"), _const_list(_cs, "HUMAN_AUTHORITY_COMMANDS")
_loop = re.search(r'\[\s*\("confirmationAuthenticityRule",\s*"accepted",\s*"acceptance"\),\s*\("rejectionAuthenticityRule",\s*"rejected",\s*"rejection"\),?\s*\]', _gr)
fact("rejectVerdictLayers", {
    "acceptRefusesBeforeRead": _refuses_before_read(_acc_body),
    "rejectRefusesBeforeRead": _refuses_before_read(_rej_body),
    "rejectRefusalWhat": (re.search(r'refuse_ai_judgment\(path, judged_by, "([^"]+)"\)', _rej_body) or [None, None])[1],
    "writeList": _wl, "authorityList": _cl,
    "listsAgree": bool(_wl and _cl and _wl["members"] == _cl["members"]),
    "guardReadsBothRules": bool(_loop),
    "refuseAiJudgmentCallers": len(re.findall(r"^\s*refuse_ai_judgment\(path, judged_by,", _wr, re.M)),
} if _wr and _cs and _gr else None, "the write path, the command lists and the guard as the source reads",
     "members/keel-write/src/write.rs: the body of `fn accept_decision_locked(` / `fn reject_decision_locked(` up to the next `\\n}\\n`; a "
     "layer refuses when `refuse_ai_judgment(` precedes the first `read_to_string(` in that body; rejectRefusalWhat = the "
     "third argument of that call; writeList = `const HUMAN_ONLY_WRITE_COMMANDS: [&str; N] = [..]` (declared N and the quoted "
     "members); authorityList = the same over `const HUMAN_AUTHORITY_COMMANDS` in view/control_structure.rs; "
     "guardReadsBothRules = guards.rs holds the two-tuple array pairing confirmationAuthenticityRule/accepted with "
     "rejectionAuthenticityRule/rejected; refuseAiJudgmentCallers = call sites `refuse_ai_judgment(path, judged_by,` in write.rs.")

# --- D0470: the two authenticity rules as declared, and as the rules gate evaluates them on this tree
_rules = read(os.path.join(REPO, ".engine", "rules", "rules.sysml")) or ""


def _rule_decl(name):
    s = _rules.find("part " + name + " : ElementRule")
    if s < 0:
        return None
    body = _rules[s:_rules.find("\n    }", s)]
    g = lambda k: (re.search(k + r'\s*=\s*"?([^";\n]+)"?;', body) or [None, None])[1]
    return {"predicate": g("predicate"), "appliesWhen": g("appliesWhen"), "severity": g("severity"),
            "onViolation": g("onViolation"),
            "justifiedBy": (re.search(r"#JustifiedBy dependency from " + name + r" to (d\d{4});", _rules) or [None, None])[1]}


ok, out = run([KEEL, "gate", "rules", "."], timeout=180)
_rj = as_json(out) if ok else None
_eval = {}
if isinstance(_rj, dict):
    for r in _rj.get("rules", []) if isinstance(_rj.get("rules"), list) else []:
        if r.get("rule") in ("confirmationAuthenticityRule", "rejectionAuthenticityRule"):
            _eval[r["rule"]] = {"evaluated": r.get("evaluated"), "violations": len(r.get("violations") or []),
                                "severity": r.get("severity"), "scope": r.get("scope")}
fact("authenticityRules", {
    "confirmation": {"declared": _rule_decl("confirmationAuthenticityRule"), "live": _eval.get("confirmationAuthenticityRule")},
    "rejection": {"declared": _rule_decl("rejectionAuthenticityRule"), "live": _eval.get("rejectionAuthenticityRule")},
} if _rules else None, "the two rules, declared and evaluated",
     "declared: .engine/rules/rules.sysml, the `part <name> : ElementRule {` body's predicate / appliesWhen / severity / "
     "onViolation and the `#JustifiedBy dependency from <name> to dNNNN;` edge; live: `keel gate rules .` JSON, the rows "
     "named confirmationAuthenticityRule and rejectionAuthenticityRule (evaluated, violation count, severity, scope)"
     + ("" if _eval else "; the rules gate did not return JSON rows: " + (out or "")[:200]))

# --- D0470: every rejected Decision in the tree, and who judged each rejection - the rule's population
_actors = read(os.path.join(REPO, ".tracking", "actors.sysml")) or ""
_persons = set(re.findall(r"part (\w+) : Person\b", _actors))
_ai = set(re.findall(r"part (\w+) : Actor\b", _actors))
_rejected = []
for _fn in sorted(os.listdir(DEC_DIR)):
    if not _fn.endswith(".sysml"):
        continue
    _t = read(os.path.join(DEC_DIR, _fn)) or ""
    _m = re.search(r"part (d\d{4}) : Decision\b", _t)
    if not _m:
        continue
    _dn = _m.group(1)
    if not re.search(r"part " + _dn + r" : Decision\s*\{[^}]*?status\s*=\s*DecisionStatus::rejected", _t, re.S):
        continue
    _judges = re.findall(r"part " + _dn + r"RejectR\d+ : TestResult\s*\{[^}]*?judgedBy\s*=\s*\"([^\"]+)\"", _t, re.S)
    _rejected.append({"decision": _dn, "rejectResults": len(_judges),
                      "judgedBy": sorted(set(_judges)),
                      "allHuman": bool(_judges) and all(j in _persons for j in _judges),
                      "anyAi": any(j in _ai for j in _judges)})
fact("rejectedDecisionCensus", {
    "rejected": len(_rejected), "rows": _rejected,
    "allHumanJudged": all(r["allHuman"] for r in _rejected),
    "persons": sorted(_persons), "aiActors": sorted(_ai),
}, "the rule's population",
     "every .engine/decisions/*.sysml whose `part dNNNN : Decision {` body carries `status = DecisionStatus::rejected` (the "
     "status member itself, not prose naming it); judgedBy = the `judgedBy = \"x\"` of each `part dNNNNRejectR<n> : TestResult` "
     "segment; a judge is human when .tracking/actors.sysml declares `part x : Person`, AI when `part x : Actor`.")

# --- D0470: the Issue the Decision resolves, and the sprint record that delivered the layers
_iss = read(os.path.join(REPO, ".tracking", "issues-claudeFable5.sysml")) or ""
_i526 = re.search(r"part issue526 : Issue\s*\{(.*?)\n    \}", _iss, re.S)
_i526b = _i526.group(1) if _i526 else ""
fact("rejectVerdictIssue", {
    "exists": bool(_i526),
    "severity": (re.search(r"severity\s*=\s*Severity::(\w+)", _i526b) or [None, None])[1],
    "createdAt": (re.search(r'createdAt\s*=\s*"([^"]+)"', _i526b) or [None, None])[1],
    "resolver": (re.search(r"#Resolves dependency from (\w+) to issue526;", _iss) or [None, None])[1],
    "title": (re.search(r'title\s*=\s*"([^"]+)"', _i526b) or [None, None])[1],
} if _iss else None, "the origin Issue",
     ".tracking/issues-claudeFable5.sysml: the `part issue526 : Issue {` body's severity / createdAt / title and the "
     "`#Resolves dependency from <task> to issue526;` edge.")

_s702p = os.path.join(REPO, ".tracking", "delivery", "sprint702_rejectIsHumanJudgedWhereAnAcceptanceIs.sysml")
_s702 = read(_s702p) if os.path.exists(_s702p) else None
if _s702 is not None:
    _res = re.findall(r"part \w+\s*:\s*TestResult\s*\{[^}]*?outcome\s*=\s*VerdictKind::(\w+)", _s702)
    _pair = re.search(r"reject_decision_refuses accept_decision_refuses human_authority_commands -> positive \(refusal removed\): FAILED ([^;]+); negative \(restored\): PASS ok\. (\d+) passed; (\d+) failed", _s702)
    fact("rejectVerdictSprint", {
        "exists": True, "results": len(_res),
        "byOutcome": {o: _res.count(o) for o in sorted(set(_res))},
        "ranReceipts": len(re.findall(r"// RAN:", _s702)),
        "probePair": {"positiveMessage": _pair.group(1).strip(), "negativePassed": int(_pair.group(2)),
                      "negativeFailed": int(_pair.group(3))} if _pair else None,
        "chartersD0470": bool(re.search(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+d0470\s*;", _s702)),
        "namesIssue529": "issue529" in _s702, "namesIssue530": "issue530" in _s702,
    }, "the delivery record",
         ".tracking/delivery/sprint702_rejectIsHumanJudgedWhereAnAcceptanceIs.sysml: results = `part x : TestResult {` segments "
         "and their `VerdictKind::x`; ranReceipts = `// RAN:` lines; probePair parsed from the receipt line naming the three "
         "unit tests (the positive's panic message, the negative's passed/failed); chartersD0470 = a `#CharteredBy dependency "
         "from <story> to d0470;` line; the two retro Issues by name.")
else:
    fact("rejectVerdictSprint", {"exists": False}, "the delivery record", "the file " + _s702p + " does not exist")

# ================================================================ 25. the living-doc verb guard (D0471 / issue528)
# --- D0471: the Decision's own fields, read from its file the way the guard reads them
_d0471 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0471-"):
        _d0471 = read(os.path.join(DEC_DIR, _fn)) or ""
_dec1 = _guard_field(_d0471, "d0471", "decision")
_rat1 = _guard_field(_d0471, "d0471", "rationale")
_con1 = _guard_field(_d0471, "d0471", "consequences")
_ctx1 = _guard_field(_d0471, "d0471", "context")
fact("cliReferenceDecision", {
    "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0471) or [None, None])[1],
    "createdAt": _guard_field(_d0471, "d0471", "createdAt") or None,
    "marker": "#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+d0471\s*:", _d0471, re.M) else None,
    "acceptance": _acceptance_kind(_d0471, "d0471"),
    "notAFork": "NOT A FORK" in (_dec1 + _rat1),
    "decision": _dec1, "rationale": _rat1, "consequences": _con1, "context": _ctx1,
    "namesIssue528": "issue528" in _ctx1,
    "namesEightSites": "eight such lines" in _ctx1,
    "processChangeWords": bool(re.search(r"process-change", _dec1 + _rat1 + _con1 + _ctx1)),
} if _d0471 else None, "the Decision's fields as the guard reads them",
     "regex over .engine/decisions/0471-*.sysml: status from `DecisionStatus::x`; marker = a line `#ProspectiveChange part d0471 :`; "
     "acceptance mirrors guards.rs acceptance_kind (None = no passing AcceptR segment); the four fields read as guards.rs "
     "unmeasured_path_decisions reads a field; notAFork = the literal `NOT A FORK` in the decision or rationale; namesIssue528 "
     "and namesEightSites = those spans in the context; processChangeWords = `process-change` anywhere in the four.")

# --- D0471: the guard in source - its name in GUARD_NAMES, its dispatch arm, the shared walk, and the three declaration surfaces
_gr = _crate_text("keel-guards")  # sprint 733: the guards are a crate, its text is every file of it
_gl = read(_mh("guard_names") or "") or ""  # sprint 732: the list is keel-schema's, the arms stay in guards.rs
_gn = re.search(r"pub const GUARD_NAMES: \[&str; (\d+)\] =\s*\[([^\]]*)\]", _gl, re.S)
_gnames = re.findall(r'"([a-z0-9-]+)"', _gn.group(2)) if _gn else []
_gmd = read(os.path.join(REPO, ".engine", "docs", "guards.md")) or ""
_gcs = read(os.path.join(REPO, ".engine", "rules", "guard-constraints.sysml")) or ""
_cmap = read(os.path.join(REPO, ".tracking", "architecture", "control-map.sysml")) or ""
fact("cliReferenceGuardSource", {
    "guardCount": int(_gn.group(1)) if _gn else None,
    "guardCountMatchesList": bool(_gn) and int(_gn.group(1)) == len(_gnames),
    "inGuardNames": "cli-reference" in _gnames,
    "dispatchArm": bool(re.search(r'"cli-reference"\s*=>\s*Some\(cli_reference\(root\)\)|\("cli-reference",\s*cli_reference\)', _gr)),
    "sharedWalk": {"toolReference": bool(re.search(r"pub fn tool_reference\(root: &Path\) -> GuardReport \{[^}]*?living_doc_files\(root\)", _gr, re.S)),
                   "cliReference": bool(re.search(r"pub fn cli_reference\(root: &Path\) -> GuardReport \{[^}]*?living_doc_files\(root\)", _gr, re.S))},
    "readsDispatch": {"hasCommand": bool(re.search(r"(crate|keel_schema)::cli_surface::has_command\(&r\.verb\)", _gr)),
                      "hasLens": bool(re.search(r"(crate|keel_schema)::cli_surface::has_lens\(", _gr)),
                      "subVerbsOf": bool(re.search(r"(crate|keel_schema)::cli_facts::sub_verbs_of\(f\.invocation\)", _gr))},
    "unitTests": len(re.findall(r"^\s*#\[test\]\s*\n\s*fn (\w+)", _gr[_gr.find("mod cli_reference_tests"):_gr.find("\n}\n", _gr.find("mod cli_reference_tests")) + 3], re.M)) if "mod cli_reference_tests" in _gr else 0,
    "catalogueRow": bool(re.search(r"^\| `cli-reference` \| HARD \(D0471 / issue528\)", _gmd, re.M)),
    "constraintDecl": bool(re.search(r"constraint def cliReference;\s*// guard 74 \(D0471/issue528\)", _gcs)),
    "controlMapPart": bool(re.search(r"part gCliReference : SystemSafetyConstraint \{", _cmap)),
    "controlMapHazard": (re.search(r"dependency from gCliReference to (ehz\d+);", _cmap) or [None, None])[1],
} if _gr else None, "the guard as source declares it",
     "the keel-guards crate's text (keel-cli/src/guards.rs until sprint 733; GUARD_NAMES from keel-schema since 732): `pub const GUARD_NAMES: [&str; N] = [...]` (N and the quoted names), the dispatch arm "
     "`\"cli-reference\" => Some(cli_reference(root))` or, since sprint 733, the family-table entry `(\"cli-reference\", cli_reference)`, both `pub fn tool_reference` and `pub fn cli_reference` bodies calling "
     "`living_doc_files(root)`, the three dispatch reads by their literal call text (`crate::` while the guards were in keel-cli, `keel_schema::` since sprint 733), `#[test] fn` count inside `mod "
     "cli_reference_tests`; .engine/docs/guards.md row `| `cli-reference` | HARD (D0471 / issue528)`; "
     ".engine/rules/guard-constraints.sysml `constraint def cliReference; // guard 74 (D0471/issue528)`; "
     ".tracking/architecture/control-map.sysml `part gCliReference : SystemSafetyConstraint {` and its `dependency from "
     "gCliReference to ehzN;` edge.")

# --- D0471: the guard run live on this tree, and the three gate lines a verifier follows, run as written
ok, out = run([KEEL, "gate", "guard", "cli-reference", "--no-receipt", "."], timeout=300)
_last = (out or "").strip().splitlines()[-1] if (out or "").strip() else ""
_m = re.search(r"\[guard:cli-reference\] (PASS|FAIL) \W+ (\d+) scanned, (\d+) warning\(s\), (\d+) violation\(s\)", _last)
fact("cliReferenceLive", {
    "verdict": _m.group(1) if _m else None, "scanned": int(_m.group(2)) if _m else None,
    "violations": int(_m.group(4)) if _m else None, "line": _last,
} if _m else None, "the guard on this tree",
     "`keel gate guard cli-reference --no-receipt .`: the last line `[guard:cli-reference] PASS|FAIL - N scanned, N warning(s), "
     "N violation(s)`; scanned = command references examined (code always, prose only with a flag or root)"
     + ("" if _m else "; the guard did not print its summary line: " + (out or "")[-300:]))

_tv = read(os.path.join(REPO, ".engine", "skills", "test-verify", "SKILL.md")) or ""
_tv_lines = _tv.splitlines()
_gate_lines = [l for l in _tv_lines if re.match(r"^KEEL gate (validate|check-engine|guard --no-receipt) \.", l)]
_retired_lines = [l for l in _tv_lines if re.match(r"^KEEL (validate|check-engine|guard) ", l)]
_ran = {}
for _verb in (["gate", "validate", "."], ["gate", "check-engine", "."]):
    _rc, _out = run_rc([KEEL] + _verb, timeout=300)
    _ran[" ".join(_verb)] = {"exit0": _rc == 0, "lastLine": (_out or "").strip().splitlines()[-1][:160] if (_out or "").strip() else ""}
_mirror_ok, _mirror_out = run([KEEL, "sync-claude", "--check", "."], timeout=120)
fact("cliReferenceProcedure", {
    "testVerifyGateLines": len(_gate_lines), "testVerifyRetiredLines": len(_retired_lines),
    "gateLinesRun": _ran,
    "claudeMirrorClean": _mirror_ok,
    "knowledgeGraphMemory": {"showWhy": "`keel show why`" in (read(os.path.join(REPO, ".engine", "skills", "knowledge-graph-memory", "SKILL.md")) or ""),
                             "showKnowledge": "`keel show knowledge question-coverage`" in (read(os.path.join(REPO, ".engine", "skills", "knowledge-graph-memory", "SKILL.md")) or "")},
}, "the verifier's procedure as written today, and run",
     ".engine/skills/test-verify/SKILL.md: lines beginning `KEEL gate validate .` / `KEEL gate check-engine .` / `KEEL gate guard "
     "--no-receipt .` (the current spelling) and lines beginning `KEEL validate ` / `KEEL check-engine ` / `KEEL guard ` (the retired "
     "one); gateLinesRun = `keel gate validate .` and `keel gate check-engine .` run here, their exit and last line; "
     "claudeMirrorClean = `keel sync-claude --check .` exit 0; the knowledge-graph-memory skill's two backticked spellings by "
     "literal search.")

# --- D0471: the origin Issue, the retro Issue, and the sprint record
_iss = read(os.path.join(REPO, ".tracking", "issues-claudeFable5.sysml")) or ""
_i528 = re.search(r"part issue528 : Issue\s*\{(.*?)\n    \}", _iss, re.S)
_i528b = _i528.group(1) if _i528 else ""
fact("cliReferenceIssue", {
    "exists": bool(_i528),
    "severity": (re.search(r"severity\s*=\s*Severity::(\w+)", _i528b) or [None, None])[1],
    "createdAt": (re.search(r'createdAt\s*=\s*"([^"]+)"', _i528b) or [None, None])[1],
    "resolver": (re.search(r"#Resolves dependency from (\w+) to issue528;", _iss) or [None, None])[1],
    "title": (re.search(r'title\s*=\s*"([^"]+)"', _i528b) or [None, None])[1],
    "descriptionSaysTen": "Ten sites" in _i528b,
    "retroIssue531": bool(re.search(r"part issue531 : Issue\b", _iss)),
    "retroIssue531Resolver": (re.search(r"#Resolves dependency from (\w+) to issue531;", _iss) or [None, None])[1],
} if _iss else None, "the origin Issue and the retro Issue",
     ".tracking/issues-claudeFable5.sysml: the `part issue528 : Issue {` body's severity / createdAt / title, whether its "
     "description opens `Ten sites`, and the `#Resolves dependency from <task> to issue528;` edge; issue531 by `part issue531 : "
     "Issue` and its own #Resolves edge.")

_s703p = os.path.join(REPO, ".tracking", "delivery", "sprint703_livingDocsNameOnlyDeclaredCliVerbs.sysml")
_s703 = read(_s703p) if os.path.exists(_s703p) else None
if _s703 is not None:
    _res = re.findall(r"part \w+\s*:\s*TestResult\s*\{[^}]*?outcome\s*=\s*VerdictKind::(\w+)", _s703)
    _before = re.search(r"before the corrections FAILED naming all eight", _s703)
    _pair = re.search(r"cli_reference_tests[^\n]*?(\d+) passed; (\d+) failed", _s703)
    fact("cliReferenceSprint", {
        "exists": True, "results": len(_res),
        "byOutcome": {o: _res.count(o) for o in sorted(set(_res))},
        "ranReceipts": len(re.findall(r"// RAN:", _s703)),
        "dodStatesBeforeFailure": bool(_before),
        "probePair": {"passed": int(_pair.group(1)), "failed": int(_pair.group(2))} if _pair else None,
        "chartersD0471": bool(re.search(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+d0471\s*;", _s703)),
        "namesIssue531": "issue531" in _s703,
        "statesOvercount": "overcounted by two" in _s703,
    }, "the delivery record",
         ".tracking/delivery/sprint703_livingDocsNameOnlyDeclaredCliVerbs.sysml: results = `part x : TestResult {` segments and "
         "their `VerdictKind::x`; ranReceipts = `// RAN:` lines; dodStatesBeforeFailure = the DoD's span `before the corrections "
         "FAILED naming all eight`; probePair = a receipt line naming cli_reference_tests with `N passed; N failed`; chartersD0471 "
         "= a `#CharteredBy dependency from <story> to d0471;` line; issue531 and `overcounted by two` by literal search.")
else:
    fact("cliReferenceSprint", {"exists": False}, "the delivery record", "the file " + _s703p + " does not exist")

# ================================================================ 26. the two doc guards read one surface (D0472 / issue533)
# --- D0472: the Decision's own fields, read from its file the way the guard reads them
_d0472 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0472-"):
        _d0472 = read(os.path.join(DEC_DIR, _fn)) or ""
_dec2 = _guard_field(_d0472, "d0472", "decision")
_rat2 = _guard_field(_d0472, "d0472", "rationale")
_con2 = _guard_field(_d0472, "d0472", "consequences")
_ctx2 = _guard_field(_d0472, "d0472", "context")
fact("sharedWalkDecision", {
    "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0472) or [None, None])[1],
    "createdAt": _guard_field(_d0472, "d0472", "createdAt") or None,
    "marker": "#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+d0472\s*:", _d0472, re.M) else None,
    "acceptance": _acceptance_kind(_d0472, "d0472"),
    "notAFork": "NOT A FORK" in (_dec2 + _rat2),
    "measuredToken": "MEASURED:" in _rat2,
    "dependsOnD0471": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0472\s+to\s+d0471\s*;", _d0472)),
    "decision": _dec2, "rationale": _rat2, "consequences": _con2, "context": _ctx2,
    "namesAba396e": "aba396e" in _ctx2,
    "namesIssue533": "issue533" in _con2,
    "namesBuilderRefusal": "the source does not share living_doc_files" in _ctx2,
    "pathWordsInDecision": sorted({w for w in re.findall(r"[a-z]+", _dec2.lower()) if w in ("land", "push", "refuse", "gate")}),
} if _d0472 else None, "the Decision's fields as the guard reads them",
     "regex over .engine/decisions/0472-*.sysml: status from `DecisionStatus::x`; marker = a line `#ProspectiveChange part d0472 :`; "
     "acceptance mirrors guards.rs acceptance_kind (None = no passing AcceptR segment); the four fields read as guards.rs "
     "unmeasured_path_decisions reads a field; notAFork / MEASURED: by literal search; dependsOnD0471 = a `#DependsOn dependency "
     "from d0472 to d0471;` line; namesAba396e / namesIssue533 / the builder's refusal sentence by literal search in the field "
     "named; pathWordsInDecision = the D0469 PATH_WORDS present as whole lower-case words in the decision text.")

# --- D0472: the relation in source - tool_reference's first statement, one definition of the walk, and who calls it
_gr = read(_mh("surface", crate="keel-guards") or "") or ""  # sprint 733: the walk and both guards live in the surface family
_grl = _gr.splitlines()
def _lineno(pattern):
    for _i, _l in enumerate(_grl, 1):
        if re.search(pattern, _l):
            return _i
    return None
_tr_line = _lineno(r"^pub fn tool_reference\(root: &Path\) -> GuardReport \{")
_tr_body = ""
if _tr_line:
    for _l in _grl[_tr_line:]:
        if re.match(r"^(pub(\(crate\))? )?fn ", _l):
            break
        _tr_body += _l + "\n"
fact("sharedWalkSource", {
    "toolReferenceLine": _tr_line,
    "cliReferenceLine": _lineno(r"^pub fn cli_reference\(root: &Path\) -> GuardReport \{"),
    "livingDocFilesLine": _lineno(r"^(?:pub\(crate\) )?fn living_doc_files\(root: &Path\)"),
    "livingDocFilesDefinitions": len(re.findall(r"^(?:pub\(crate\) )?fn living_doc_files\(", _gr, re.M)),
    "callers": len(re.findall(r"living_doc_files\(root\)", _gr)),
    "toolReferenceFirstStatement": (_grl[_tr_line].strip() if _tr_line and _tr_line < len(_grl) else None),
    "toolReferenceCallsSharedWalk": bool(_tr_line) and _grl[_tr_line].strip() == "let files = living_doc_files(root);",
    "toolReferenceHasInlineWalk": "fn walk(" in _tr_body,
    "toolReferenceBodyLines": _tr_body.count("\n"),
} if _gr else None, "the guard source",
     "members/keel-guards/src/surface.rs (keel-cli/src/guards.rs until sprint 733) read as lines: the 1-based line of `pub fn tool_reference(...) {`, `pub fn cli_reference(...) {` and "
     "`fn living_doc_files(...)` (pub(crate) since sprint 733); definitions = lines beginning `[pub(crate) ]fn living_doc_files(`; callers = occurrences of "
     "`living_doc_files(root)` (two = both guards, since the definition's own line does not carry `(root)` followed by `)`); "
     "toolReferenceFirstStatement = the stripped line after the fn header, compared to `let files = living_doc_files(root);`; "
     "toolReferenceHasInlineWalk = `fn walk(` anywhere in the body up to the next top-level `fn`.")

# --- D0472: the guard whose walk moved, run live
ok, out = run([KEEL, "gate", "guard", "tool-reference", "--no-receipt", "."], timeout=300)
_last = (out or "").strip().splitlines()[-1] if (out or "").strip() else ""
_m = re.search(r"\[guard:tool-reference\] (PASS|FAIL) \W+ (\d+) scanned, (\d+) warning\(s\), (\d+) violation\(s\)", _last)
fact("toolReferenceLive", {
    "verdict": _m.group(1) if _m else None, "scanned": int(_m.group(2)) if _m else None,
    "violations": int(_m.group(4)) if _m else None, "line": _last,
} if _m else None, "the guard on this tree",
     "`keel gate guard tool-reference --no-receipt .`: the last line `[guard:tool-reference] PASS|FAIL - N scanned, N warning(s), "
     "N violation(s)` parsed; scanned counts the living-doc files that carry a `.engine/tools/` path.")

# --- issue533: the finding, and where its resolver sits
_i533 = re.search(r"part issue533 : Issue\s*\{(.*?)\n\s*\}", _iss or "", re.S)
_i533b = _i533.group(1) if _i533 else ""
_bl = read(os.path.join(REPO, ".tracking", "backlog.sysml")) or ""
_nw = re.search(r"action def NextWork \{(.*?)\n    \}", _bl, re.S)
_nw_actions = re.findall(r"^\s{8}action (\w+);", _nw.group(1), re.M) if _nw else []
fact("sharedWalkIssue", {
    "exists": bool(_i533),
    "severity": (re.search(r"severity\s*=\s*Severity::(\w+)", _i533b) or [None, None])[1],
    "createdAt": (re.search(r'createdAt\s*=\s*"([^"]+)"', _i533b) or [None, None])[1],
    "resolver": (re.search(r"#Resolves dependency from (\w+) to issue533;", _iss or "") or [None, None])[1],
    "title": (re.search(r'title\s*=\s*"([^"]+)"', _i533b) or [None, None])[1],
    "namesBuilder": "build_2026_09_13_cli_reference.py" in _i533b,
    "namesD0472": "D0472" in _i533b,
    "resolverPosition": (_nw_actions.index("dcDeliveredClaimsAreReadBackFromSource") + 1) if "dcDeliveredClaimsAreReadBackFromSource" in _nw_actions else None,
    "nextWorkItems": len(_nw_actions),
    "resolverDodNamesIssue": bool(re.search(r"dcDeliveredClaimsAreReadBackFromSourceDoD[^\n]*Resolves issue533\.", _bl)),
} if _iss else None, "the finding Issue and its resolver",
     ".tracking/issues-claudeFable5.sysml: the `part issue533 : Issue {` body's severity / createdAt / title, the builder's file "
     "name and `D0472` by literal search, and the `#Resolves dependency from <task> to issue533;` edge; .tracking/backlog.sysml: "
     "the resolver's 1-based position among `action x;` lines inside `action def NextWork` (declaration order IS priority, D0052) "
     "and whether its DoD line carries `Resolves issue533.` (guard issues, D0304).")

# ================================================================ 27. the recorder's report is refused on a non-record write or a record written twice (D0473 / issue532)
# --- D0473: the Decision's own fields, read from its file the way the guard reads them
_d0473 = ""
for _fn in os.listdir(DEC_DIR):
    if _fn.startswith("0473-"):
        _d0473 = read(os.path.join(DEC_DIR, _fn)) or ""
_dec3 = _guard_field(_d0473, "d0473", "decision")
_rat3 = _guard_field(_d0473, "d0473", "rationale")
_con3 = _guard_field(_d0473, "d0473", "consequences")
_ctx3 = _guard_field(_d0473, "d0473", "context")
fact("recorderRefusalDecision", {
    "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0473) or [None, None])[1],
    "createdAt": _guard_field(_d0473, "d0473", "createdAt") or None,
    "marker": "#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+d0473\s*:", _d0473, re.M) else None,
    "acceptance": _acceptance_kind(_d0473, "d0473"),
    "notAFork": "NOT A FORK" in (_dec3 + _rat3),
    "measuredToken": "MEASURED:" in _rat3,
    "decision": _dec3, "rationale": _rat3, "consequences": _con3, "context": _ctx3,
    "namesIssue532": "issue532" in _ctx3,
    "namesTextpatch": "textpatch" in _ctx3,
    "namesLineNine": "naming line 9" in _rat3,
    "namesReminder": "D0047" in _ctx3,
    "pathWordsInDecision": sorted({w for w in re.findall(r"[a-z]+", _dec3.lower()) if w in ("land", "push", "refuse", "gate")}),
} if _d0473 else None, "the Decision's fields as the guard reads them",
     "regex over .engine/decisions/0473-*.sysml: status from `DecisionStatus::x`; marker = a line `#ProspectiveChange part d0473 :`; "
     "acceptance mirrors guards.rs acceptance_kind (None = no passing AcceptR segment); the four fields read as guards.rs "
     "unmeasured_path_decisions reads a field; notAFork / MEASURED: / issue532 / textpatch / `naming line 9` / D0047 by literal "
     "search in the field named; pathWordsInDecision = the D0469 PATH_WORDS present as whole lower-case words in the decision text "
     "(`gate --fast` puts `gate` there, which is why the rationale must carry MEASURED:).")

# --- D0473: the checker as source - the refusal sentences it can emit, the pair table, the fixtures on disk, and the .claude copy
_ckp = os.path.join(REPO, ".engine", "skills", "delegated-ceremony", "references", "check_report.py")
_ck = read(_ckp) or ""
_ck_claude = read(os.path.join(REPO, ".claude", "skills", "delegated-ceremony", "references", "check_report.py")) or ""
_fx_dir = os.path.join(REPO, ".engine", "skills", "delegated-ceremony", "references", "fixtures")
_pairs_src = re.search(r"^PAIRS = \[(.*?)^\]", _ck, re.S | re.M)
# a PAIRS row is `(fixture, expectation)` until D0492, `(fixture, expectation, owed)` after it; both shapes are read
_pairs = re.findall(r'\("([^"]+)",\s*(None|"[^"]*")(?:,\s*(?:None|\d+))?\)', _pairs_src.group(1)) if _pairs_src else []
_refusal_sentences = re.findall(r'found\.append\(f?"(?:line \{n\}: )?([^`"{]+)', _ck)
fact("recorderCheckerSource", {
    "refusalKinds": len(_refusal_sentences),
    "refusalSentences": [x.strip(" -") for x in _refusal_sentences],
    "pairs": len(_pairs),
    "positives": len([p for p in _pairs if p[1] != "None"]),
    "negatives": len([p for p in _pairs if p[1] == "None"]),
    "fixturesOnDisk": len([p for p in _pairs if os.path.exists(os.path.join(_fx_dir, p[0]))]),
    "sprint703Fixtures": len([p for p in _pairs if "sprint703" in p[0]]),
    "isRecordWriteFn": bool(re.search(r"^def is_record_write\(command\):", _ck, re.M)),
    "recordKeyPattern": bool(re.search(r"^RECORD_KEY = re\.compile\(", _ck, re.M)),
    "claudeCopyIdentical": _ck == _ck_claude and bool(_ck),
    "lines": _ck.count("\n"),
} if _ck else None, "the checker's source",
     ".engine/skills/delegated-ceremony/references/check_report.py: refusalKinds = `found.append(` calls in refusals(); pairs = "
     "the `(fixture, expectation[, owed])` tuples inside `PAIRS = [...]`, positives carry a quoted expectation and negatives `None`; "
     "fixturesOnDisk = those whose file exists under references/fixtures/; isRecordWriteFn / RECORD_KEY = the def and the compiled "
     "pattern the two new refusals read; claudeCopyIdentical = byte equality with .claude/skills/.../check_report.py (sync-claude).")

# --- D0473: the checker run live - the pair table, then each sprint-703 fixture on its own
_probe_rc, out = run_rc([sys.executable, _ckp, "--probe", "--root", "."], timeout=120)
_probe_lines = (out or "").strip().splitlines()
_per_fixture = {}
for _name, _expect in _pairs:
    _frc, _fout = run_rc([sys.executable, _ckp, os.path.join(_fx_dir, _name), "--root", "."], timeout=120)
    _flines = [l for l in (_fout or "").strip().splitlines() if l.strip()]
    _per_fixture[_name] = {"exit0": _frc == 0, "refusals": len([l for l in _flines if l.startswith("  line ") or l.startswith("line ")]),
                           "firstLine": _flines[0] if _flines else "", "lastLine": _flines[-1] if _flines else ""}
fact("recorderCheckerLive", {
    "probeExit0": _probe_rc == 0,
    "probeLastLine": _probe_lines[-1] if _probe_lines else "",
    "pairsHolding": len([l for l in _probe_lines if l.startswith("probe: known-") and ("-> PASS" in l or "-> REFUSED naming" in l)]),
    "perFixture": _per_fixture,
} if _probe_lines else None, "the checker on this tree",
     "`python check_report.py --probe --root .`: exit code and the last line (`probe: every pair holds.`); pairsHolding = probe lines "
     "reading `-> PASS` (a negative) or `-> REFUSED naming` (a positive); then the checker run once per PAIRS fixture, its exit and "
     "first/last output lines - a positive exits 1 with `REFUSED:` first, a negative exits 0 with the `check_report: pass` line.")

# --- issue532: the finding, and where its resolver sits (the def that holds it, read - not assumed to be NextWork)
_rri_eb = re.search(r"action def EngineBuild \{(.*?)^    \}", _bl, re.S | re.M)
_rri_eb_actions = re.findall(r"^\s{8}action (\w+);", _rri_eb.group(1), re.M) if _rri_eb else []
_rri_def, _rri_actions = next(((d, a) for d, a in (("NextWork", _nw_actions), ("EngineBuild", _rri_eb_actions))
                               if "dcRecorderReportRefusesNonRecordWrites" in a), (None, []))
_i532 = re.search(r"part issue532 : Issue\s*\{(.*?)\n\s*\}", _iss or "", re.S)
_i532b = _i532.group(1) if _i532 else ""
fact("recorderRefusalIssue", {
    "exists": bool(_i532),
    "severity": (re.search(r"severity\s*=\s*Severity::(\w+)", _i532b) or [None, None])[1],
    "createdAt": (re.search(r'createdAt\s*=\s*"([^"]+)"', _i532b) or [None, None])[1],
    "resolver": (re.search(r"#Resolves dependency from (\w+) to issue532;", _iss or "") or [None, None])[1],
    "title": (re.search(r'title\s*=\s*"([^"]+)"', _i532b) or [None, None])[1],
    "namesTextpatch": "textpatch" in _i532b,
    "namesThreeMore": "three more times" in _i532b,
    "namesCheckerLines": "check_report.py lines 50-71" in _i532b,
    "resolverPosition": (_rri_actions.index("dcRecorderReportRefusesNonRecordWrites") + 1) if "dcRecorderReportRefusesNonRecordWrites" in _rri_actions else None,
    "resolverDef": _rri_def,
    "defItems": len(_rri_actions),
    "resolverDodNamesIssue": bool(re.search(r"dcRecorderReportRefusesNonRecordWritesDoD[^\n]*Resolves issue532\.", _bl)),
} if _iss else None, "the finding Issue and its resolver",
     ".tracking/issues-claudeFable5.sysml: the `part issue532 : Issue {` body's severity / createdAt / title, textpatch, `three more "
     "times` and the checker's line span by literal search, and the `#Resolves dependency from <task> to issue532;` edge; "
     ".tracking/backlog.sysml: the resolver's 1-based position among `action x;` lines inside the def that holds it - NextWork "
     "or EngineBuild, named in resolverDef (declaration "
     "order IS priority, D0052) and whether its DoD line carries `Resolves issue532.` (guard issues, D0304).")

# --- sprint 704: the delivery record the recorder wrote through the API, and the doc surfaces D0473 amended
_s704p = os.path.join(REPO, ".tracking", "delivery", "sprint704_recorderReportRefusesNonRecordWrites.sysml")
_s704 = read(_s704p) or ""
_proc = read(os.path.join(REPO, ".engine", "processes", "delegated-ceremony.sysml")) or ""
_skill = read(os.path.join(REPO, ".engine", "skills", "delegated-ceremony", "SKILL.md")) or ""
_cmd = read(os.path.join(REPO, "CLAUDE.md")) or ""
if _s704:
    _res4 = re.findall(r"part \w+\s*:\s*TestResult\s*\{[^}]*?outcome\s*=\s*VerdictKind::(\w+)", _s704)
    _shas = sorted(set(re.findall(r'judgedAgainst\s*=\s*"([^"]+)"', _s704)))
    fact("recorderRefusalSprint", {
        "exists": True, "results": len(_res4),
        "byOutcome": {o: _res4.count(o) for o in sorted(set(_res4))},
        "judgedAgainst": _shas,
        "gateResults": len(re.findall(r"part \w+GateR\d*\s*:\s*TestResult", _s704)),
        "chartersD0473": bool(re.search(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+d0473\s*;", _s704)),
        "evidenceNamesProbe": _s704.count("PROBE PAIR:"),
        "evidenceNamesOldReportRefused": _s704.count("REFUSED: write outside the record API"),
        "processNamesD0473": _proc.count("D0473"),
        "processNamesOneWrite": "One write per owed record" in _proc,
        "skillRuleFive": "(5) one write per owed record" in _skill,
        "skillNamesFiveFixtures": "five" in _skill and "fixtures" in _skill,
        "claudeMdNamesD0473": "D0473" in _cmd,
    }, "the delivery record and the amended surfaces",
         ".tracking/delivery/sprint704_recorderReportRefusesNonRecordWrites.sysml: results = `part x : TestResult {` segments and their "
         "`VerdictKind::x`; judgedAgainst = the distinct `judgedAgainst = \"sha\"` values; gateResults = `part xGateRn : TestResult`; "
         "chartersD0473 = a `#CharteredBy dependency from <story> to d0473;` line; evidence spans by literal count. "
         ".engine/processes/delegated-ceremony.sysml counts `D0473` and carries `One write per owed record`; the recorder brief in "
         ".engine/skills/delegated-ceremony/SKILL.md carries rule `(5) one write per owed record`; CLAUDE.md names D0473 (doc-sync).")
else:
    fact("recorderRefusalSprint", {"exists": False}, "the delivery record", "the file " + _s704p + " does not exist")

# ================================================================ 28. the twenty-sixth publish's three asks
# D0477 (the CLI facts in force), D0483 (the layering guard, a plan held before its sprint) and D0484 (the build skill's
# layout paragraph). Each Decision's fields are read from its file the way the guard reads them; the source, the live
# guard, the manifests, the skill paragraph, the Issues and the sprint records are read from the tree - nothing typed.


def _dec_file(prefix):
    for _fn in os.listdir(DEC_DIR):
        if _fn.startswith(prefix):
            return read(os.path.join(DEC_DIR, _fn)) or ""
    return ""


def _decision_facts(text, dname):
    """The fields every tab states: status, marker, acceptance, the four fields, the D0469 path words and token."""
    _d = _guard_field(text, dname, "decision")
    return {
        "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", text) or [None, None])[1],
        "createdAt": _guard_field(text, dname, "createdAt") or None,
        "createdBy": _guard_field(text, dname, "createdBy") or None,
        "marker": ("#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+" + dname + r"\s*:", text, re.M)
                   else "#SafetyChange" if re.search(r"^\s*#SafetyChange\s+part\s+" + dname + r"\s*:", text, re.M) else None),
        "acceptance": _acceptance_kind(text, dname),
        "notAFork": "NOT A FORK" in _d,
        "measuredToken": "MEASURED:" in (_guard_field(text, dname, "rationale") or ""),
        "pathWordsInDecision": _path_words(_d),
        "title": _guard_field(text, dname, "title"),
        "decision": _d, "rationale": _guard_field(text, dname, "rationale"),
        "consequences": _guard_field(text, dname, "consequences"), "context": _guard_field(text, dname, "context"),
    }


_DEC_HOW = ("regex over the Decision's file: status from `DecisionStatus::x`; marker = a line `#ProspectiveChange part dNNNN :` "
            "(or #SafetyChange); acceptance mirrors guards.rs acceptance_kind (None = no passing AcceptR segment); the fields read "
            "as guards.rs unmeasured_path_decisions reads a field; notAFork = the phrase in the decision field; measuredToken = "
            "`MEASURED:` in the rationale; pathWordsInDecision = the D0469 PATH_WORDS present as whole lower-case words.")

# --- D0477: the Decision, and the names it carries
_d0477 = _dec_file("0477-")
_f477 = _decision_facts(_d0477, "d0477")
_f477.update({
    "namesIssue547": "issue547" in (_f477["context"] or ""),
    "namesIssue548": "issue548" in (_f477["context"] or ""),
    "namesSprint712": "sprint 712" in (_f477["context"] or ""),
    "namesD0108": "D0108" in (_f477["decision"] or ""),
    "namesD0271": "D0271" in (_f477["decision"] or ""),
    "namesTwoPrecedents": "74c1923" in (_f477["context"] or "") and "sprint 711" in (_f477["context"] or ""),
    "namesOneDrift": "86 facts diffed, one drift" in (_f477["context"] or ""),
    "probedBeforeEnabled": "one invocation drift, zero duplicate names" in (_f477["rationale"] or ""),
    "namesRemovalPath": "Removal path:" in (_f477["consequences"] or ""),
})
fact("cliInForceDecision", _f477 if _d0477 else None, "the Decision's fields as the guard reads them",
     _DEC_HOW + " Names by literal search: issue547 / issue548 / `sprint 712` / 74c1923 + `sprint 711` / `86 facts diffed, one drift` "
     "in context; D0108 / D0271 in decision; `one invocation drift, zero duplicate names` in rationale; `Removal path:` in consequences.")

# --- D0477: the reader and the comparison in source, and the shared retired set
_gr = _crate_text("keel-guards")  # sprint 733: the guards are a crate, its text is every file of it
_lib = read(_mh("lib", crate="keel-cli") or "") or ""
# the reader moved from guards.rs to members/keel-schema/src/cli_facts.rs in sprint 717 (D0479: down into a leaf); the page reads it where it is
_cf = read(os.path.join(REPO, "members", "keel-schema", "src", "cli_facts.rs")) or ""
_pcf = _fn_body(_cf, "parse_cli_facts") if _cf else ""
_csv = _fn_body(_gr, "cli_surface_violations") if _gr else ""
_corpus = read(_mh("corpus", crate="keel-model") or "") or ""  # sprint 733: supersede_edges descended into keel-model
_se = _fn_body(_corpus, "supersede_edges") if _corpus else ""
_tests_mod = re.search(r"^mod cli_surface_declared_tests \{(.*?)(?=^// FILE: |\Z)", _gr, re.S | re.M)  # one file of the crate text
_tests_body = _tests_mod.group(1) if _tests_mod else ""
_test_names = re.findall(r"^\s*fn (\w+)\(\)", _tests_body, re.M)
_gn = re.search(r"pub const GUARD_NAMES: \[&str; (\d+)\] =\s*\[([^\]]*)\]", read(_mh("guard_names") or "") or "", re.S)  # sprint 732: the list is keel-schema's
_gnames = re.findall(r'"([^"]+)"', _gn.group(2)) if _gn else []
fact("cliInForceSource", {
    "authoredFactHasPart": bool(re.search(r"pub struct AuthoredCliFact \{[^}]*pub part: String", _cf, re.S)),
    "parserCollectsRetired": 'strip_prefix("#Supersede dependency from ")' in _pcf and "retired.contains(&part)" in _pcf,
    "comparisonReadsInvocation": '("invocation", f.invocation.as_str(), m.invocation)' in _csv,
    "duplicateInForceIsAViolation": "is declared by two CliCommand facts in force" in _csv,
    "supersedeEdgesScansEngineCli": '.join(".engine").join("cli")' in _se,
    "testModuleTests": len(_test_names),
    "liveTestPresent": "the_live_facts_mirror_and_dispatch_agree" in _test_names,
    "supersessionTest": next((t for t in _test_names if "superseded" in t), None),
    "invocationDriftTest": next((t for t in _test_names if "invocation" in t and "drift" in t), None),
    "guardInNames": "cli-surface-declared" in _gnames,
    "guardCount": int(_gn.group(1)) if _gn else None,
    "guardCountMatchesList": bool(_gn) and int(_gn.group(1)) == len(_gnames),
} if _gr and _corpus else None, "the reader, the comparison and the shared retired set in source",
     "members/keel-schema/src/cli_facts.rs (guards.rs until sprint 717): `pub struct AuthoredCliFact {` carrying `pub part: String`; the body of `parse_cli_facts` carrying "
     "`strip_prefix(\"#Supersede dependency from \")` and `retired.contains(&part)`; the body of `cli_surface_violations` carrying "
     "the `(\"invocation\", f.invocation.as_str(), m.invocation)` tuple and the sentence `is declared by two CliCommand facts in "
     "force`; `fn x()` names inside `mod cli_surface_declared_tests`; GUARD_NAMES count and members (members/keel-schema/src/guard_names.rs since sprint 732). "
     "members/keel-model/src/corpus.rs (keel-cli/src/lib.rs until sprint 733): the body of `supersede_edges` joining `.engine/cli`.")

# --- D0477: the facts on disk - every CliCommand line, the #Supersede edges beside them, the set in force
_cmds = read(os.path.join(REPO, ".engine", "cli", "commands.sysml")) or ""
_cmd_lines = [l for l in _cmds.splitlines() if ": CliCommand {" in l]
_cmd_parts = [(l.strip().split()[1], (re.search(r':>>\s*name\s*=\s*"([^"]*)"', l) or [None, None])[1]) for l in _cmd_lines]
_sup = re.findall(r"^\s*#Supersede dependency from (\w+) to (\w+);", _cmds, re.M)
_retired = {t for _, t in _sup}
_in_force = [(p, nm) for p, nm in _cmd_parts if p not in _retired]
_names_in_force = [nm for _, nm in _in_force]
_dups = sorted({nm for nm in _names_in_force if _names_in_force.count(nm) > 1})
_acc_line = next((l for l in _cmd_lines if l.strip().startswith("part cliAccept2 ")), "")
_acc_inv = (re.search(r':>>\s*invocation\s*=\s*"([^"]*)"', _acc_line) or [None, None])[1]
_cf = read(_mh("cli_facts") or "") or ""
_cf_acc = re.search(r'name: "accept",[^}]*?invocation: "([^"]*)"', _cf, re.S)
fact("cliFactsInForce", {
    "cliCommandLines": len(_cmd_lines),
    "supersedeEdges": len(_sup),
    "retiredParts": sorted(_retired),
    "supersederParts": sorted(f for f, _ in _sup),
    "inForce": len(_in_force),
    "duplicateNamesInForce": _dups,
    "retiredNamesReDeclared": sorted({nm for p, nm in _cmd_parts if p in _retired}),
    "acceptInvocationAuthored": _acc_inv,
    "acceptInvocationMirror": _cf_acc.group(1) if _cf_acc else None,
    "acceptAgrees": bool(_acc_inv) and bool(_cf_acc) and _acc_inv == _cf_acc.group(1),
} if _cmds else None, "the CLI facts as authored, and the set in force",
     ".engine/cli/commands.sysml: lines carrying `: CliCommand {` (part name = the second token, name = its `name` attribute); "
     "`#Supersede dependency from X to Y;` lines; inForce = parts not a Y; duplicateNamesInForce = names two in-force parts share; "
     "the `invocation` attribute of `part cliAccept2`; cli_facts.rs: the `invocation:` of the `name: \"accept\"` entry.")

# --- D0477: the guard live on this tree
_rc, out = run_rc([KEEL, "gate", "guard", "cli-surface-declared", "--no-receipt", "."], timeout=300)
_last = (out or "").strip().splitlines()[-1] if (out or "").strip() else ""
_m = re.search(r"\[guard:cli-surface-declared\] (PASS|FAIL) \W+ (\d+) scanned, (\d+) warning\(s\), (\d+) violation\(s\)", _last)
fact("cliSurfaceLive", {
    "verdict": _m.group(1) if _m else None, "scanned": int(_m.group(2)) if _m else None,
    "warnings": int(_m.group(3)) if _m else None, "violations": int(_m.group(4)) if _m else None,
    "line": _last, "exit0": _rc == 0,
} if _m else None, "guard cli-surface-declared on this tree",
     "`" + KEEL + " gate guard cli-surface-declared --no-receipt .`: the last line `[guard:cli-surface-declared] PASS|FAIL - N scanned, "
     "N warning(s), N violation(s)`" + ("" if _m else " - did not match: " + _last[:200]))

# --- D0477: the two Issues, the resolver's DoD result, and sprint 712's record
_iss = read(os.path.join(REPO, ".tracking", "issues-claudeFable5.sysml")) or ""
_bl = read(os.path.join(REPO, ".tracking", "backlog.sysml")) or ""


def _issue_facts(num, resolver_expected):
    _i = re.search(r"part issue" + num + r" : Issue\s*\{(.*?)\n\s*\}", _iss, re.S)
    _b = _i.group(1) if _i else ""
    _res = (re.search(r"#Resolves dependency from (\w+) to issue" + num + ";", _iss) or [None, None])[1]
    return {
        "exists": bool(_i),
        "severity": (re.search(r"severity\s*=\s*Severity::(\w+)", _b) or [None, None])[1],
        "createdAt": (re.search(r'createdAt\s*=\s*"([^"]+)"', _b) or [None, None])[1],
        "title": (re.search(r'title\s*=\s*"([^"]+)"', _b) or [None, None])[1],
        "resolver": _res,
        "resolverAsExpected": _res == resolver_expected,
        "resolverDodNamesIssue": bool(re.search(resolver_expected + r"DoD[^\n]*Resolves issue" + num + r"[ .:]", _bl)),
    }


def _dod_results(action):
    _rs = re.findall(r"part " + action + r"DoDR\d+ : TestResult \{[^}]*?outcome = VerdictKind::(\w+);[^}]*?judgedAgainst = \"([^\"]+)\"", _bl)
    return [{"outcome": o, "judgedAgainst": s} for o, s in _rs]


fact("cliInForceIssues", {
    "issue547": _issue_facts("547", "dcCliFactSupersessionIsHonoured"),
    "issue548": _issue_facts("548", "dcCliFactSupersessionIsHonoured"),
    "resolverDodResults": _dod_results("dcCliFactSupersessionIsHonoured"),
} if _iss and _bl else None, "the two findings and the resolver's DoD result",
     ".tracking/issues-claudeFable5.sysml: each `part issueN : Issue {` body's severity / createdAt / title and its `#Resolves` edge; "
     ".tracking/backlog.sysml: the resolver's DoD line carrying `Resolves issueN`, and its `DoDRn : TestResult` outcomes and shas.")


def _sprint_facts(fname, story_charter):
    _p = os.path.join(REPO, ".tracking", "delivery", fname)
    _s = read(_p) or ""
    if not _s:
        return {"exists": False, "path": _p}
    _res = re.findall(r"part \w+\s*:\s*TestResult\s*\{[^}]*?outcome\s*=\s*VerdictKind::(\w+)", _s)
    return {
        "exists": True, "results": len(_res),
        "byOutcome": {o: _res.count(o) for o in sorted(set(_res))},
        "judgedAgainst": sorted(set(re.findall(r'judgedAgainst\s*=\s*"([^"]+)"', _s))),
        "gateResults": len(re.findall(r"part \w+GateR\d*\s*:\s*TestResult", _s)),
        "charter": (re.search(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+(\w+)\s*;", _s) or [None, None])[1],
        "chartersAsExpected": bool(re.search(r"#CharteredBy\s+dependency\s+from\s+\w+\s+to\s+" + story_charter + r"\s*;", _s)),
        "estimatedPoints": (re.search(r"estimatedPoints\s*=\s*(\d+)", _s) or [None, None])[1],
        "text": _s,
    }


_SPRINT_HOW = ("the delivery record: results = `part x : TestResult {` segments and their `VerdictKind::x`; judgedAgainst = the "
               "distinct shas; gateResults = `part xGateRn : TestResult`; charter = the `#CharteredBy dependency from <story> to "
               "<d>;` target; estimatedPoints from the Story.")
_s712 = _sprint_facts("sprint712_cliFactSupersessionIsHonoured.sysml", "d0477")
fact("cliInForceSprint", {k: v for k, v in _s712.items() if k != "text"} | ({
    "implementNamesTwelvePassed": "cli_surface -> 12 passed 0 failed" in _s712["text"],
    "namesOneAuthoredFact": "parses to ONE fact" in _s712["text"],
} if _s712.get("exists") else {}), "sprint 712's record", _SPRINT_HOW + " Literal spans `cli_surface -> 12 passed 0 failed` and `parses to ONE fact`.")

# --- D0483: the Decision, the human's words it derives from, the workspace graph today, the item that would deliver it
_d0483 = _dec_file("0483-")
_f483 = _decision_facts(_d0483, "d0483")
_f483.update({
    "derivedFromSt126": bool(re.search(r"#DerivedFrom dependency from d0483 to st126;", _d0483)),
    "namesD0479": "D0479" in (_f483["context"] or ""),
    "namesD0047": "D0047" in (_f483["rationale"] or ""),
    "namesD0209": "D0209" in (_f483["decision"] or ""),
    "namesContract": ".engine/contracts/workspace-layers.toml" in (_f483["decision"] or ""),
    "namesGuard": "workspace-layering" in (_f483["decision"] or ""),
    "knownPositive": "keel-git depending on keel-view fails naming both" in (_f483["decision"] or ""),
    "knownNegative": "the real workspace at the commit that adds the guard passes" in (_f483["decision"] or ""),
    "namesSprintItem": "dcWorkspaceLayeringIsGuarded" in (_f483["consequences"] or ""),
})
fact("layeringDecision", _f483 if _d0483 else None, "the Decision's fields as the guard reads them",
     _DEC_HOW + " derivedFromSt126 = a `#DerivedFrom dependency from d0483 to st126;` line; the names by literal search in the field named.")

_st_files = [os.path.join(REPO, ".tracking", "intake", f) for f in os.listdir(os.path.join(REPO, ".tracking", "intake"))] if os.path.isdir(os.path.join(REPO, ".tracking", "intake")) else []
_st126 = None
for _p in _st_files:
    _t = read(_p) or ""
    _m = re.search(r"part st126 : Statement\s*\{(.*?)\n\s*\}", _t, re.S)
    if _m:
        _b = _m.group(1)
        _st126 = {
            "file": os.path.relpath(_p, REPO).replace("\\", "/"),
            "text": (re.search(r':>>\s*text\s*=\s*"(.*?)";', _b, re.S) or [None, None])[1],
            "saidBy": (re.search(r'saidBy\s*=\s*"([^"]+)"', _b) or [None, None])[1],
            "saidAt": (re.search(r'saidAt\s*=\s*"([^"]+)"', _b) or [None, None])[1],
            "channel": (re.search(r"channel\s*=\s*StatementChannel::(\w+)", _b) or [None, None])[1],
            "title": (re.search(r':>>\s*title\s*=\s*"([^"]+)"', _b) or [None, None])[1],
        }
fact("modularityStatement", _st126, "the human's words the layering Decision derives from",
     ".tracking/intake/*.sysml: the `part st126 : Statement {` body's text (verbatim, D0236), saidBy, saidAt, channel, title.")

_root_toml = read(os.path.join(REPO, "Cargo.toml")) or ""
_members_m = re.search(r"members\s*=\s*\[(.*?)\]", _root_toml, re.S)
_members = re.findall(r'"([^"]+)"', _members_m.group(1)) if _members_m else []
_edges = []
for _mp in _members:
    _mt = read(os.path.join(REPO, _mp, "Cargo.toml")) or ""
    _crate = (re.search(r'^name\s*=\s*"([^"]+)"', _mt, re.M) or [None, os.path.basename(_mp)])[1]
    _deps_m = re.search(r"^\[dependencies\](.*?)(?=^\[|\Z)", _mt, re.S | re.M)
    for _dn in re.findall(r"^(keel-[\w-]+)\s*=", _deps_m.group(1) if _deps_m else "", re.M):
        _edges.append({"from": _crate, "to": _dn})
_leaf_members = [m for m in _members if m.startswith("members/")]
_contract_p = os.path.join(REPO, ".engine", "contracts", "workspace-layers.toml")
_layer_item = re.search(r"^\s*action dcWorkspaceLayeringIsGuarded;", _bl, re.M)
_layer_dod = (re.search(r"dcWorkspaceLayeringIsGuardedDoD[^\n]*procedureText = \"(.*?)\";", _bl) or [None, ""])[1]
_eb = re.search(r"action def EngineBuild \{(.*?)^    \}", _bl, re.S | re.M)
_eb_actions = re.findall(r"^\s{8}action (\w+);", _eb.group(1), re.M) if _eb else []
ok, out = run([KEEL, "show", "whats-next", "."], timeout=120)
_ready_names = [l.strip() for l in (out or "").splitlines() if l.strip()] if ok else []
fact("workspaceGraph", {
    "members": _members,
    "leafMembers": _leaf_members,
    "memberEdges": _edges,
    "edgesAmongLeaves": [e for e in _edges if e["from"] != "keel-cli" and e["to"] != "keel-parser"],
    "keelCliDependsOn": sorted(e["to"] for e in _edges if e["from"] == "keel-cli"),
    "layersContractExists": os.path.exists(_contract_p),
    "guardInNames": "workspace-layering" in _gnames,
    "guardCount": int(_gn.group(1)) if _gn else None,
    "item": {
        "exists": bool(_layer_item),
        "positionInEngineBuild": (_eb_actions.index("dcWorkspaceLayeringIsGuarded") + 1) if "dcWorkspaceLayeringIsGuarded" in _eb_actions else None,
        "engineBuildItems": len(_eb_actions),
        "charteredByD0483": bool(re.search(r"#CharteredBy dependency from dcWorkspaceLayeringIsGuarded to d0483;", _bl)),
        "dependsOnD0483": bool(re.search(r"#DependsOn dependency from dcWorkspaceLayeringIsGuarded to d0483;", _bl)),
        "dependsOnLeafSprint": bool(re.search(r"#DependsOn dependency from dcWorkspaceLayeringIsGuarded to dcWorkspaceHoldsTheLeafMembers;", _bl)),
        "dodSaysBlocked": "this item stays blocked on that acceptance" in _layer_dod,
        "dodNamesFourHomes": "all four homes" in _layer_dod,
        "readyToday": "dcWorkspaceLayeringIsGuarded" in _ready_names,
        "readyCount": len(_ready_names),
    },
} if _members else None, "the workspace as the manifests declare it, and the item that would add the guard",
     "Cargo.toml `members = [...]`; each member's Cargo.toml `[dependencies]` entries named keel-*, as (from crate name, to); "
     "leafMembers = paths under members/; layersContractExists = the file the Decision names is on disk; guardInNames = "
     "`workspace-layering` in guards.rs GUARD_NAMES; .tracking/backlog.sysml: the item's line, its 1-based position among `action x;` "
     "lines inside `action def EngineBuild`, its #CharteredBy / #DependsOn edges and two DoD phrases; readyToday = its name among the "
     "lines of `" + KEEL + " show whats-next .`.")

_i552 = _issue_facts("552", "dcReadyHonoursItemDependencies")
_i552["namesBlockedBy"] = "blocked_by" in ((re.search(r"part issue552 : Issue\s*\{(.*?)\n\s*\}", _iss, re.S) or [None, ""])[1])
_i552["resolverPosition"] = (_eb_actions.index("dcReadyHonoursItemDependencies") + 1) if "dcReadyHonoursItemDependencies" in _eb_actions else None
_i552["resolverReadyRank"] = (_ready_names.index("dcReadyHonoursItemDependencies") + 1) if "dcReadyHonoursItemDependencies" in _ready_names else None
_i549 = _issue_facts("549", "dcReadyHonoursItemDependencies")
_vm = read(os.path.join(REPO, "members", "keel-model", "src", "queries.rs")) or ""
_bb = _fn_body(_vm, "blocked_by") if _vm else ""
fact("charterBlockIssue", {
    "issue552": _i552, "issue549": _i549,
    "blockedByReadsKinds": sorted(set(re.findall(r'e\.kind == "(\w+)"', _bb))),
    "blockedByReadsCharter": "charteredby" in _bb,
    "resolverDodResults": _dod_results("dcReadyHonoursItemDependencies"),
} if _iss else None, "the finding the D0483 tab surfaced, the filter it names, and the resolver's DoD result",
     ".tracking/issues-claudeFable5.sysml: `part issue552 : Issue {` and `part issue549`, each with severity, resolver edge and whether the "
     "resolver's DoD names it; resolverPosition = the resolver's place in EngineBuild, resolverReadyRank = its line in whats-next (null once done); "
     "resolverDodResults = the resolver's `DoDRn : TestResult` outcomes and shas in .tracking/backlog.sysml; "
     "members/keel-model/src/queries.rs: the edge kinds `e.kind == \"x\"` inside `fn blocked_by`.")

# --- D0484: the Decision, the paragraph it governs against the manifest, the charter it layers on, sprint 714's record
_d0484 = _dec_file("0484-")
_f484 = _decision_facts(_d0484, "d0484")
_d0479 = _dec_file("0479-")
_f484.update({
    "dependsOnD0479": bool(re.search(r"#DependsOn dependency from d0484 to d0479;", _d0484)),
    "namesSprint714": "Sprint 714" in (_f484["context"] or ""),
    "namesLocked": "locked process definition" in (_f484["context"] or ""),
    "namesD0209": "D0209" in (_f484["context"] or ""),
    "saysNothingElseChanges": "Nothing else in the skill changes" in (_f484["decision"] or ""),
    "saysDescriptive": "The change is descriptive" in (_f484["rationale"] or ""),
    "namesPatternAlternative": "naming the pattern" in (_f484["consequences"] or ""),
    "d0479": {
        "status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0479) or [None, None])[1],
        "acceptance": _acceptance_kind(_d0479, "d0479"),
        "judgedBy": sorted(set(re.findall(r'judgedBy\s*=\s*"([^"]+)"', _d0479))),
        "marker": ("#ProspectiveChange" if re.search(r"^\s*#ProspectiveChange\s+part\s+d0479\s*:", _d0479, re.M)
                   else "#SafetyChange" if re.search(r"^\s*#SafetyChange\s+part\s+d0479\s*:", _d0479, re.M) else None),
    },
})
fact("buildSkillDecision", _f484 if _d0484 else None, "the Decision's fields as the guard reads them, and the charter it layers on",
     _DEC_HOW + " dependsOnD0479 = a `#DependsOn dependency from d0484 to d0479;` line; names by literal search in the field named; "
     "d0479 = the charter Decision's status, acceptance kind, judgedBy set and marker (None = unmarked) from .engine/decisions/0479-*.sysml.")

_skill_p = os.path.join(REPO, ".engine", "skills", "build", "SKILL.md")
_skill = read(_skill_p) or ""
_skill_claude = read(os.path.join(REPO, ".claude", "skills", "build", "SKILL.md")) or ""
_para_m = re.search(r"\*\*Workspace layout:\*\*(.*?)\n\n", _skill, re.S)
_para = _para_m.group(1) if _para_m else ""
_para_crates = re.findall(r"`(keel-[\w-]+)`", _para)
_manifest_crates = [m.rsplit("/", 1)[-1] for m in _members]
fact("buildSkillParagraph", {
    "exists": bool(_para_m),
    "text": _para.strip(),
    "cratesNamed": _para_crates,
    "manifestCrates": _manifest_crates,
    "everyManifestCrateNamed": all(c in _para_crates for c in _manifest_crates),
    "namedNotInManifest": [c for c in _para_crates if c not in _manifest_crates],
    "namesD0479": "D0479" in _para,
    "saysReExported": "re-exports" in _para,
    "namesDenySet": "deny(warnings" in _para,
    "claudeCopyIdentical": bool(_skill) and _skill == _skill_claude,
    "skillLockedWords": "process-change" in (_f484["rationale"] or ""),
    # the D0486 wording: the paragraph names the layering and points at Cargo.toml, and enumerates no leaf
    "namesLeafLayer": "leaf crates" in _para,
    "namesReadModelLayer": bool(re.search(r"read model\s+`keel-model`", _para)),
    "namesWriteLayer": bool(re.search(r"write layer\s+`keel-write`", _para)),
    "namesCargoMembersTable": "`[workspace] members` in `Cargo.toml` is the one list" in _para,
    "namesD0486": "D0486" in _para,
    "leafMembersEnumerated": [c for c in _para_crates if c in _manifest_crates and c not in ("keel-parser", "keel-cli", "keel-model", "keel-write")],
} if _skill else None, "the layout paragraph against the manifest, and against D0486's wording",
     ".engine/skills/build/SKILL.md: the paragraph from `**Workspace layout:**` to the next blank line; cratesNamed = its `keel-x` code "
     "spans; manifestCrates = the last path segment of each root Cargo.toml member; everyManifestCrateNamed = set inclusion; "
     "D0479 / `re-exports` / `deny(warnings` by literal search; claudeCopyIdentical = byte equality with .claude/skills/build/SKILL.md; "
     "the layering words by literal search (`leaf crates`, `read model `keel-model``, `write layer `keel-write``, the Cargo.toml "
     "sentence, D0486); leafMembersEnumerated = the code spans that are manifest members other than the parser, the cli and the two "
     "named layers (D0486 says this list is empty).")

_s714 = _sprint_facts("sprint714_workspaceHoldsTheLeafMembers.sysml", "d0479")
_i551 = _issue_facts("551", "dcRecordRefusesAnUnknownFlag")
_i551["resolverPosition"] = (_eb_actions.index("dcRecordRefusesAnUnknownFlag") + 1) if "dcRecordRefusesAnUnknownFlag" in _eb_actions else None
_i551["resolverReadyRank"] = (_ready_names.index("dcRecordRefusesAnUnknownFlag") + 1) if "dcRecordRefusesAnUnknownFlag" in _ready_names else None
_i550 = _issue_facts("550", "dcTouchedSetDescendsTheWorkspace")
fact("buildSkillSprint", {k: v for k, v in _s714.items() if k != "text"} | ({
    "retroNamesD0484": "D0484" in _s714["text"],
    "retroNamesIssue551": "issue551" in _s714["text"],
    "retroNamesIssue550": "issue550" in _s714["text"],
    "retroSaysEdgeByHand": "the edge was authored by hand" in _s714["text"],
    "helpByteIdentical": "byte-identical" in _s714["text"],
    "memberLibTests": (re.search(r"(\d+) tests over the non-cli members", _s714["text"]) or [None, None])[1],
    "touchedPassed": (re.search(r"keel suite --touched: (\d+) passed", _s714["text"]) or [None, None])[1],
    "issue551": _i551, "issue550": _i550,
} if _s714.get("exists") else {}), "sprint 714's record and the two findings it carried",
     _SPRINT_HOW + " Literal spans D0484 / issue551 / issue550 / `the edge was authored by hand` / `byte-identical`; `N tests over the "
     "non-cli members` and `keel suite --touched: N passed` from the evidence; the two Issues as for the others, with the resolver's "
     "EngineBuild position and whats-next rank.")

# --- D0486: the Decision that replaces D0484's enumerated clause, and sprint 718's record with its three findings
_d0486 = _dec_file("0486-")
_f486 = _decision_facts(_d0486, "d0486")
_f486.update({
    "supersedesClauseD0484": bool(re.search(r"#SupersedeClause dependency from d0486 to d0484;", _d0486)),
    "supersedesD0484Whole": bool(re.search(r"#Supersede dependency from d0486 to d0484;", _d0486)),
    "dependsOnD0479": bool(re.search(r"#DependsOn dependency from d0486 to d0479;", _d0486)),
    "namesD0484": "D0484" in (_f486["context"] or ""),
    "namesSprint718": "Sprint 718" in (_f486["context"] or ""),
    "namesThreeMembers": all(m in (_f486["context"] or "") for m in ("members/keel-fs", "members/keel-model", "members/keel-write")),
    "namesLocked": "locked process definition" in (_f486["context"] or ""),
    "namesD0209": "D0209 clause 2" in (_f486["context"] or ""),
    "saysLayering": "describes members/ by its D0479 layering" in (_f486["decision"] or ""),
    "saysCargoOneList": "names the workspace members table in Cargo.toml as the one list of crates" in (_f486["decision"] or ""),
    "saysEnumeratesNoMember": "It enumerates no member." in (_f486["decision"] or ""),
    "saysNothingElseChanges": "Nothing else in the skill changes" in (_f486["decision"] or ""),
    "namesD0105": "D0105" in (_f486["rationale"] or ""),
    "saysD0484PredictedDrift": "D0484 recorded that it would" in (_f486["rationale"] or ""),
    "saysOnlyANewLayer": "only a new LAYER does" in (_f486["consequences"] or ""),
    "saysKeepsD0484Rule": "keeping its rule that the skill describes the workspace as its members" in (_f486["consequences"] or ""),
    "d0484": {"status": _f484["status"], "acceptance": _f484["acceptance"], "marker": _f484["marker"]},
    "d0479": _f484["d0479"],
})
fact("buildSkillLayeringDecision", _f486 if _d0486 else None,
     "the Decision's fields as the guard reads them, the clause it reverses and the charter it layers on",
     _DEC_HOW + " supersedesClauseD0484 / supersedesD0484Whole / dependsOnD0479 = the literal edge lines in the file (D0398: never both); "
     "names by literal search in the field named; d0484 = D0484's own status, acceptance kind and marker; d0479 as for buildSkillDecision.")

_s718 = _sprint_facts("sprint718_modelAndWriteAreMembers.sysml", "d0479")
_i556 = _issue_facts("556", "dcArchDriftDropsSupersededElements")
_i557 = _issue_facts("557", "dcOneRepoRootHelper")
_i557["resolverPosition"] = (_eb_actions.index("dcOneRepoRootHelper") + 1) if "dcOneRepoRootHelper" in _eb_actions else None
_i557["resolverReadyRank"] = (_ready_names.index("dcOneRepoRootHelper") + 1) if "dcOneRepoRootHelper" in _ready_names else None
_i558 = _issue_facts("558", "dcRecordIssueRefusesANonResolverKind")
fact("buildSkillLayeringSprint", {k: v for k, v in _s718.items() if k != "text"} | ({
    "retroNamesD0486": "D0486" in _s718["text"],
    "retroNamesIssue556": "issue556" in _s718["text"],
    "retroNamesIssue557": "issue557" in _s718["text"],
    "retroNamesIssue558": "issue558" in _s718["text"],
    "retroSaysD0484WentStale": "went stale on the very next extraction" in _s718["text"],
    "helpByteIdentical": "byte-identical" in _s718["text"],
    "touchedPassed": (re.search(r"touched (\d+) passed 0 failed", _s718["text"]) or [None, None])[1],
    "touchedSeconds": (re.search(r"touched \d+ passed 0 failed in (\d+) s", _s718["text"]) or [None, None])[1],
    "deliveredItems": re.findall(r"dc\w+", (re.search(r"DELIVERED BACKLOG ITEMS: ([^.]*)\.", _s718["text"]) or [None, ""])[1]),
    "issue556": _i556, "issue557": _i557, "issue558": _i558,
    "issue556ResolverDod": _dod_results("dcArchDriftDropsSupersededElements"),
    "issue558ResolverDod": _dod_results("dcRecordIssueRefusesANonResolverKind"),
} if _s718.get("exists") else {}), "sprint 718's record and the three findings it carried",
     _SPRINT_HOW + " Literal spans D0486 / issue556 / issue557 / issue558 / `went stale on the very next extraction` / `byte-identical`; "
     "`touched N passed 0 failed in S s` from the evidence; deliveredItems = the dc-names in the Story's `DELIVERED BACKLOG ITEMS:` "
     "sentence; the three Issues as for the others (issue557's resolver with its EngineBuild position and whats-next rank); the two "
     "delivered resolvers' `DoDRn : TestResult` outcomes and shas in .tracking/backlog.sysml.")

# --- D0485: the Decision, the module graph it makes the check, the compositions it moved, sprint 717's record and its finding
_d0485 = _dec_file("0485-")
_f485 = _decision_facts(_d0485, "d0485")
_f485.update({
    "derivedFromSt126": bool(re.search(r"#DerivedFrom dependency from d0485 to st126;", _d0485)),
    "namesD0479": "D0479" in (_f485["context"] or ""),
    "namesModgraph": "scripts/modgraph.py" in (_f485["context"] or ""),
    "namesReferenceCount": (re.search(r"read (\d+) references across (\w+) back-edges", _f485["context"] or "") or [None, None, None])[1],
    "saysViewRunsNoGuard": (_f485["decision"] or "").startswith("A view runs no guard"),
    "namesReadinessSignature": "view::readiness(root, task_suspect, invariant_violations)" in (_f485["decision"] or ""),
    "namesNoTraitObject": "No trait object hides an edge" in (_f485["decision"] or ""),
    "namesForbiddenPairsInScript": "five forbidden pairs listed in it" in (_f485["decision"] or ""),
    "namesD0209": "D0209 clause 2" in (_f485["rationale"] or ""),
    "namesGuardCountUnchanged": "guard count is the same before and after" in (_f485["rationale"] or ""),
    "namesNextItem": "dcWorkspaceLayeringIsGuarded" in (_f485["consequences"] or ""),
    "d0479": _f484["d0479"],
})
fact("viewNoGuardDecision", _f485 if _d0485 else None, "the Decision's fields as the guard reads them, and the charter it applies",
     _DEC_HOW + " derivedFromSt126 = a `#DerivedFrom dependency from d0485 to st126;` line; names by literal search in the field named; "
     "namesReferenceCount = the integer in `read N references across` in the context; d0479 as for buildSkillDecision.")

# the instrument, run live, and the source shape it reports on
_mg_p = os.path.join(REPO, "scripts", "modgraph.py")
_mg = read(_mg_p) or ""
_mg_pairs = re.findall(r'\("(\w+)",\s*"(\w+)"\)', (re.search(r"FORBIDDEN\s*=\s*\[(.*?)\]", _mg, re.S) or [None, ""])[1])
_rc_mg, _out_mg = run_rc([sys.executable, _mg_p, "--check"])
_mg_counts = re.search(r"\((\d+) modules, (\d+) edges\)", _out_mg)


def _module_home(name):
    """The file for a module wherever the workspace holds it (scripts/module_home.py, issue559); None if nowhere."""
    return _mh(name)


_view_mod = read(_module_home("view/mod") or "") or ""
_guards_rs = _crate_text("keel-guards")  # sprint 733: the crate's every file
_main_rs = read(_mh("main", crate="keel-cli") or "") or ""
_leaves = ["textscan", "ident", "done", "evidence", "suspect", "gitfacts", "binding"]
_ups = ["reports", "priority"]
_instr = read(os.path.join(REPO, ".tracking", "architecture", "engine-instruments.sysml")) or ""
fact("viewNoGuardGraph", {
    "scriptExists": bool(_mg),
    "forbiddenPairs": [f"{a} -> {b}" for a, b in _mg_pairs],
    "checkExit0": _rc_mg == 0,
    "checkLastLine": (_out_mg.strip().splitlines() or [""])[-1][:200],
    "modules": int(_mg_counts.group(1)) if _mg_counts else None,
    "edges": int(_mg_counts.group(2)) if _mg_counts else None,
    "viewHasReadinessTakingViolations": "pub fn readiness(root: &Path, task_suspect: Vec<String>, invariant_violations: Vec<String>)" in _view_mod,
    "viewNamesGuards": "crate::guards" in re.sub(r"//[^\n]*", "", _view_mod),
    "guardsHasComputeReadiness": "pub fn compute_readiness(" in _guards_rs,
    "guardsHasAssuredReport": "pub fn assured_report(" in _guards_rs,
    "leafModulesPresent": [m for m in _leaves if _module_home(m)],
    "leafModuleHomes": {m: os.path.relpath(_module_home(m), REPO).replace(os.sep, "/") for m in _leaves if _module_home(m)},
    "compositionsPresent": [m for m in _ups if _module_home(m)],
    "dynInNewModules": sum((read(_module_home(m)) or "").count("dyn ") for m in _leaves + _ups if _module_home(m)),
    "mainCallsMovedSymbols": all(s in _main_rs for s in ["keel_cli::priority::priority", "reports::report", "guards::assured_report"]),
    "sensorDeclared": "snModGraph : Sensor" in _instr and 'mechanism = "scripts/modgraph.py"' in _instr,
} if _mg else None, "the module graph check run live, and the shape of the source it reports on",
     "scripts/modgraph.py: FORBIDDEN pairs parsed from its `FORBIDDEN = [...]` list; `--check` run here, exit code and last line kept, "
     "modules/edges from its `(N modules, M edges)` span; view/mod.rs read where it lives (keel-cli/src, else members/keel-model/src "
     "after sprint 718) for the `readiness` signature and `crate::guards` with // comments removed, `compute_readiness` and `assured_report` "
     "in keel-cli/src/guards.rs, the leaf and composition files on disk in either home (leafModuleHomes says which), "
     "`dyn ` counted over them, the three moved call sites in main.rs; .tracking/architecture/engine-instruments.sysml for `snModGraph : Sensor` "
     "with the script as its mechanism.")

_s717 = _sprint_facts("sprint717_guardsViewOrientCycleIsBroken.sysml", "d0479")
_i555 = _issue_facts("555", "dcLockedSurfaceEditIsNamedAtWriteTime")
_i555["resolverPosition"] = (_eb_actions.index("dcLockedSurfaceEditIsNamedAtWriteTime") + 1) if "dcLockedSurfaceEditIsNamedAtWriteTime" in _eb_actions else None
_i555["resolverReadyRank"] = (_ready_names.index("dcLockedSurfaceEditIsNamedAtWriteTime") + 1) if "dcLockedSurfaceEditIsNamedAtWriteTime" in _ready_names else None
_hook = re.search(r"fn hook_pre_write\(.*?\n\}\n", _main_rs, re.S)
_i555["hookAsksIsLockedPath"] = "is_locked_path" in (_hook.group(0) if _hook else "")
fact("viewNoGuardSprint", {k: v for k, v in _s717.items() if k != "text"} | ({
    "retroNamesIssue555": "issue555" in _s717["text"],
    "retroNamesResolver": "dcLockedSurfaceEditIsNamedAtWriteTime" in _s717["text"],
    "retroSaysScanHeld": "Avoidable-issue scan held" in _s717["text"],
    "retroDropsDocCommentCandidate": "modgraph DOES strip doc comments" in _s717["text"],
    "helpByteIdentical": "identical, 14593 bytes" in _s717["text"] or "help-old.txt help-new.txt identical" in _s717["text"],
    "guardLineUnchanged": "guards: 75 (66 hard-blocking, 9 warning-only)" in _s717["text"],
    "touchedPassed": (re.search(r"keel suite --touched pass - (\d+) passed", _s717["text"]) or [None, None])[1],
    "ladderSeconds": (re.search(r"every rung green in (\d+) s", _s717["text"]) or [None, None])[1],
    "dodResults": _dod_results("dcGuardsViewOrientCycleIsBroken"),
    "issue555": _i555,
} if _s717.get("exists") else {}), "sprint 717's record, the item's DoD result and the one finding it carried",
     _SPRINT_HOW + " Literal spans issue555 / dcLockedSurfaceEditIsNamedAtWriteTime / `Avoidable-issue scan held` / `modgraph DOES strip doc "
     "comments` / the cmp and guards lines; `keel suite --touched pass - N passed` and `every rung green in N s` from the evidence; the "
     "Issue as for the others, with the resolver's EngineBuild position and whats-next rank; hookAsksIsLockedPath = `is_locked_path` "
     "inside `fn hook_pre_write` in keel-cli/src/main.rs (the finding is that it is absent).")

# --- st126 / ProjectBusinessModularMembers: the Needs the Business gate asks the human to confirm, quoted from the record
def _fields(body):
    out = {}
    for _m in re.finditer(r':>>\s*(\w+)\s*=\s*(?:"((?:[^"\\]|\\.)*)"|([A-Za-z]+::[A-Za-z]+))\s*;', body):
        out[_m.group(1)] = _m.group(2) if _m.group(2) is not None else _m.group(3)
    return out


_mm = read(os.path.join(REPO, ".tracking", "business", "modular-members.sysml")) or ""
_mm_needs = []
for _m in re.finditer(r"requirement\s+(\w+)\s*:\s*Need\s*\{(.*?)\n\s*\}", _mm, re.DOTALL):
    _f = _fields(_m.group(2))
    _mm_needs.append({"name": _m.group(1), "title": _f.get("title"), "statement": _f.get("statement"),
                      "priority": (_f.get("priority") or "").split("::")[-1] or None,
                      "source": (_f.get("source") or "").split("::")[-1] or None,
                      "derivedFrom": re.findall(r"#DerivedFrom dependency from %s to (\w+);" % _m.group(1), _mm)})
_mm_gate = re.search(r"verification\s+(\w+)\s*:\s*Test\s*\{(.*?)\n\s*\}", _mm, re.DOTALL)
_mm_gate_name = _mm_gate.group(1) if _mm_gate else None
_mm_gate_has_result = bool(_mm_gate and re.search(r":>>\s*verifiedBy\s*=\s*%s|judgedAgainst.*%s" % (_mm_gate_name, _mm_gate_name), _mm))
_in = read(os.path.join(REPO, ".tracking", "intake", "intake-2026-09-14.sysml")) or ""
_mm_stories = []
for _m in re.finditer(r"part\s+(us\d+)\s*:\s*UserStory\s*\{(.*?)\n\s*\}", _in, re.DOTALL):
    _f = _fields(_m.group(2))
    if re.search(r"#DerivedFrom dependency from %s to st126;" % _m.group(1), _in):
        _mm_stories.append({"name": _m.group(1), "title": _f.get("title"), "asA": _f.get("asA"), "iWant": _f.get("iWant"),
                            "soThat": _f.get("soThat"), "implication": (_f.get("implication") or "").split("::")[-1] or None})
_st126 = re.search(r"part st126\s*:\s*Statement\s*\{(.*?)\n\s*\}", _in, re.DOTALL)
_st126_f = _fields(_st126.group(1)) if _st126 else {}
fact("modularMembersNeeds", {
    "needs": _mm_needs, "stories": _mm_stories,
    "gate": {"name": _mm_gate_name, "method": (_fields(_mm_gate.group(2)).get("method") or "").split("::")[-1] if _mm_gate else None,
             "hasResult": _mm_gate_has_result},
    "statement": {"name": "st126", "saidBy": _st126_f.get("saidBy"), "saidAt": _st126_f.get("saidAt"), "text": _st126_f.get("text")},
} if _mm_needs and _mm_stories and _st126 else None, "the Needs awaiting the human's word, their stories and the statement",
     ".tracking/business/modular-members.sysml: every `requirement <name> : Need` block's title, statement, priority and source, "
     "plus the `#DerivedFrom dependency from <need> to <story>;` lines naming it; the one `verification ... : Test` block is the "
     "Business gate and `hasResult` is whether any TestResult in the file names it. .tracking/intake/intake-2026-09-14.sysml: "
     "every UserStory with a `#DerivedFrom ... to st126;` edge, and st126's own fields, quoted.")

# --- the thirtieth queue: the process-hook chain (D0488 -> D0489 -> D0490), the keel-issues Decision-scope fork (D0487),
# the starved-cat deny (D0491), and the Architecture read-back they were recorded beside (sprint 721)
_req = read(os.path.join(REPO, ".tracking", "requirements", "modular-members-requirements.sysml")) or ""


def _chain_link(prefix, dname):
    _t = _dec_file(prefix)
    _f = _decision_facts(_t, dname)
    _f.update({
        "derivedFromSt126": bool(re.search(r"#DerivedFrom dependency from %s to st126;" % dname, _t)),
        "dependsOn": re.findall(r"#DependsOn dependency from %s to (\w+);" % dname, _t),
        "requirementsDerived": sorted(set(re.findall(r"#DerivedFrom dependency from (\w+) to %s;" % dname, _req))),
        "measuredMs": [int(x) for x in re.findall(r"cost (\d+)-(\d+) ms", _f["rationale"] or "")[0]] if re.search(r"cost (\d+)-(\d+) ms", _f["rationale"] or "") else None,
    })
    return _f if _t else None


_chain = {"d0488": _chain_link("0488-", "d0488"), "d0489": _chain_link("0489-", "d0489"), "d0490": _chain_link("0490-", "d0490")}
fact("processHookChain", _chain if all(_chain.values()) else None,
     "the three one-clause process-change Decisions, each held, and what depends on what",
     _DEC_HOW + " dependsOn = the `#DependsOn dependency from dNNNN to dMMMM;` lines in the Decision's own file; requirementsDerived = "
     "the SystemRequirements / SubsystemRequirements in .tracking/requirements/modular-members-requirements.sysml carrying a "
     "`#DerivedFrom ... to dNNNN;` edge; measuredMs = the `cost A-B ms` range in the MEASURED: sentence of the rationale.")

# what the chain is about: where step order is enforced today
_cursor = read(_module_home("cursor") or "") or ""
_guards_src = _crate_text("keel-guards")  # sprint 733: the crate's every file
_precommit = read(os.path.join(REPO, ".githooks", "pre-commit")) or ""
_ci = read(os.path.join(REPO, ".github", "workflows", "ci.yml")) or ""
_act = read(os.path.join(REPO, ".engine", "contracts", "activation.toml")) or ""
_act_active = re.findall(r'"([\w-]+)"', (re.search(r"\[processes\].*?active\s*=\s*\[(.*?)\]", _act, re.S) or [None, ""])[1])
_proc_dir = os.path.join(REPO, ".engine", "processes")
_proc_files = sorted(f for f in os.listdir(_proc_dir) if f.endswith(".sysml")) if os.path.isdir(_proc_dir) else []
_bound = {f[:-6]: len(re.findall(r"checkedBy\s*=", read(os.path.join(_proc_dir, f)) or "")) for f in _proc_files}
_bound = {k: v for k, v in _bound.items() if v}
_check_kinds = sorted(set(re.findall(r'checkedBy\s*=\s*"([^"]+)"', "".join(read(os.path.join(_proc_dir, f)) or "" for f in _proc_files))))
fact("processCursorCensus", {
    "processFiles": len(_proc_files), "adopted": _act_active, "adoptedCount": len(_act_active),
    "processesWithBoundSteps": _bound, "boundStepKinds": _check_kinds,
    "gatePrefixedBindings": len([k for k in _check_kinds if k.startswith("gate:")]),
    "projectProcessDirExists": os.path.isdir(os.path.join(REPO, ".tracking", "processes")),
    "resolverProcessDirs": sorted(set(re.findall(r'root\.join\("(\.\w+)"\)\.join\("processes"\)', _cursor))),
    "cursorStoredNowhere": "never stored" in _cursor.split("\nuse ")[0],
    "guardsReadingCursor": len(re.findall(r"\bcursor\b", _guards_src)),
    "preCommitRunsAdvance": "advance" in _precommit, "ciRunsAdvance": "advance" in _ci,
    "preCommitGateCalls": len(re.findall(r"gate (validate|guard)", _precommit)), "ciGateCalls": len(re.findall(r"gate validate", _ci)),
} if _proc_files and _cursor else None, "where a process's step order is enforced today, and where it is not",
     ".engine/processes/*.sysml counted; activation.toml `[processes] active` quoted; processesWithBoundSteps = files with at least one "
     "`checkedBy =` and how many; boundStepKinds = the distinct checkedBy strings; resolverWalksEngineOnly = keel-cli/src/cursor.rs "
     "names .engine and never a tracking path; cursorStoredNowhere = its head comment says computed / never stored; guardsReadingCursor = "
     "whole-word `cursor` in guards.rs; preCommitRunsAdvance / ciRunsAdvance = the word `advance` in .githooks/pre-commit / ci.yml; "
     "the gate-call counts = `gate validate|guard` occurrences in those two files; resolverProcessDirs = the `root.join(\"<dir>\")"
     ".join(\"processes\")` walks in cursor.rs; cursorStoredNowhere = `never stored` in its module comment.")

# D0487: the fork on where a project's own Decision lives when it runs keel-issues alone
_d0487 = _dec_file("0487-")
_f487 = _decision_facts(_d0487, "d0487")
_dec_text = _f487["decision"] or ""
_opts = re.split(r"OPTION ([A-C])\b", _dec_text)
_options = []
for _k in range(1, len(_opts) - 1, 2):
    _body = _opts[_k + 1]
    _options.append({"letter": _opts[_k], "recommended": "recommended" in _body[:200].lower(), "hasCost": "COST:" in _body,
                     "text": _body.strip()[:700]})
_d0480 = _dec_file("0480-")
fact("decisionScopeFork", {
    **_f487,
    "options": _options, "optionCount": len(_options), "costsStated": sum(1 for o in _options if o["hasCost"]),
    "dependsOn": re.findall(r"#DependsOn dependency from d0487 to (\w+);", _d0487),
    "derivedFromSt126": bool(re.search(r"#DerivedFrom dependency from d0487 to st126;", _d0487)),
    "requirementsDerived": sorted(set(re.findall(r"#DerivedFrom dependency from (\w+) to d0487;", _req))),
    "d0480Title": _guard_field(_d0480, "d0480", "title"), "d0480Status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0480) or [None, None])[1],
    "resolverKindAllowsDecision": bool(re.search(r"Decision", read(os.path.join(REPO, ".engine", "schema", "core", "relationships.sysml")) or "")),
} if _d0487 and _options else None, "the fork's options as written, with their costs, and the Decision it narrows",
     _DEC_HOW + " options = the decision field split at `OPTION A|B|C`, each with `recommended` in its first 200 characters and a "
     "`COST:` sentence or not; d0480 fields from its file; resolverKindAllowsDecision = the word Decision in the core relationships schema.")

# D0491 and issue564: the starved-cat deny, the control that was missing, and the probe rows the hook ledgered
_d0491 = _dec_file("0491-")
_f491 = _decision_facts(_d0491, "d0491")
_sc = read(_module_home("shellcheck") or "") or ""
_pb = _fn_body(_main_rs, "hook_pre_bash")
_cm = read(os.path.join(REPO, ".tracking", "architecture", "control-map.sysml")) or ""
_ledger = read(os.path.join(REPO, ".keel", "metrics", "hooks.jsonl")) or ""
_starved_rows = [l for l in _ledger.splitlines() if '"control":"stdin-starved-write"' in l]
_iss_f = read(os.path.join(REPO, ".tracking", "issues-claudeFable5.sysml")) or ""
_i564 = re.search(r"part issue564 : Issue\s*\{(.*?)\n\s*\}", _iss_f, re.S)
_st127 = read(os.path.join(REPO, ".tracking", "intake", "intake-2026-09-15.sysml")) or ""
_st127_m = re.search(r"part st127\s*:\s*Statement\s*\{(.*?)\n\s*\}", _st127, re.S)
fact("starvedCatControl", {
    **_f491,
    "derivedFromSt127": bool(re.search(r"#DerivedFrom dependency from d0491 to st127;", _d0491)),
    "statement": _fields(_st127_m.group(1)) if _st127_m else None,
    "issue564": {"exists": bool(_i564), "resolver": (re.search(r"#Resolves dependency from (\w+) to issue564;", _iss_f) or [None, None])[1],
                 "severity": (re.search(r"severity\s*=\s*Severity::(\w+)", _i564.group(1)) or [None, None])[1] if _i564 else None},
    "detectorPresent": "pub fn stdin_starved_write" in _sc, "detectorTest": "a_cat_with_nothing_feeding_it_is_named_and_fed_shapes_are_not" in _sc,
    "preBashDenies": len(re.findall(r'"permissionDecision": "deny"', _pb)) - (1 if "strict-bash-verdict" in _pb else 0),
    "preBashDenyControls": re.findall(r'hook_refuse\(\s*"([\w-]+)"', _pb),
    "backslashDenyNamesEditTool": "Edit tool" in _pb,
    "controlMapRow": "ctlStdinStarvedWrite" in _cm and "dependency from ctlStdinStarvedWrite to ehz4;" in _cm,
    "ledgerRows": {"deny": sum(1 for l in _starved_rows if '"decision":"deny"' in l), "total": len(_starved_rows)},
} if _d0491 and _sc else None, "the second pre-bash deny: its Decision, Issue, detector, control-map row and first ledger rows",
     _DEC_HOW + " statement = st127's fields from .tracking/intake/intake-2026-09-15.sysml; issue564 from .tracking/issues-claudeFable5.sysml "
     "with its #Resolves edge; detectorPresent/Test by literal search in keel-cli/src/shellcheck.rs; preBashDenies = `permissionDecision: deny` "
     "emissions in main.rs hook_pre_bash less the strict-profile one; preBashDenyControls = the hook_refuse control names in that body in "
     "order; controlMapRow = the part and its EHZ4 edge in control-map.sysml; ledgerRows = .keel/metrics/hooks.jsonl lines with "
     "control=stdin-starved-write and how many are denies (machine-local).")

# the Architecture read-back the four Decisions were recorded beside: the three tiers, the eight SRs, the honest allocation gap
ok, out = run([KEEL, "show", "tier-satisfaction", "."], timeout=120)
_ts = as_json(out) if ok else None
_mm_sr = re.findall(r"part\s+(sr\w+)\s*:\s*SystemRequirement", _req)
_mm_ssr = re.findall(r"requirement\s+(ssr\w+)\s*:\s*SubsystemRequirement", _req)
_mm_need_names = [n["name"] for n in _mm_needs]
_alloc = read(os.path.join(REPO, ".tracking", "architecture", "allocations.sysml")) or ""
_alloc_new = [s for s in _mm_sr if re.search(r"allocate %s to \w+;" % s, _alloc)]
_arch_gate = re.search(r"verification\s+modularMembersArchitectureGate\s*:\s*Test", _req)
if _ts:
    _tiers = {t["tier"]: t for t in _ts["tiers"]}
    fact("architectureReadBack", {
        "tiers": {k: {"total": t["total"], "satisfied": t["satisfied"], "gaps": len(t["gaps"])} for k, t in _tiers.items()},
        "needsInGaps": [n for n in _mm_need_names if any(n in t["gaps"] for t in _ts["tiers"])],
        "systemRequirements": _mm_sr, "subsystemRequirements": _mm_ssr,
        "srInVerifyGaps": [s for s in _mm_sr if s in next(t["gaps"] for t in _ts["tiers"] if t["tier"] == "SystemRequirement" and "alloc" not in t["relation"])],
        "srAllocated": _alloc_new,
        "satisfyEdges": len(re.findall(r"\bsatisfy\s+n\w+\s+by\s+sr\w+;", _req)),
        "gateDeclared": bool(_arch_gate),
    }, "the tiers as the lens reads them, and the eight requirements' place in each",
         "`keel show tier-satisfaction .` JSON: per tier total / satisfied / gaps; needsInGaps = the modular-members Needs among any tier's "
         "gaps; the SR and SSR names from modular-members-requirements.sysml by declaration; srInVerifyGaps = those in the SystemRequirement "
         "verified-by tier's gaps; srAllocated = those with an `allocate srX to ...;` line in allocations.sysml; satisfyEdges = "
         "`satisfy nX by srY;` lines in the requirements file.")
else:
    fact("architectureReadBack", None, "the tiers as the lens reads them", "tier-satisfaction lens failed: " + out)

# --- the authority-queue lens: which obligation kinds it enumerates; a confirmation gate awaiting a human is not among them
ok, out = run([KEEL, "show", "authority-queue", "."], timeout=120)
_aq_kinds = sorted(set(re.findall(r'"kind":\s*"([A-Za-z]+)"', out))) if ok else None
fact("authorityQueueKinds", {"kinds": _aq_kinds, "listsConfirmationGates": bool(_aq_kinds and any("onfirmation" in k for k in _aq_kinds))} if _aq_kinds is not None else None,
     "obligation kinds the lens enumerates",
     "`keel show authority-queue .`: the distinct values of every `\"kind\"` field in the JSON; `listsConfirmationGates` is whether any "
     "kind names a confirmation - a `method=confirmation` Test with no result (a Business gate) is a human obligation the lens does "
     "not enumerate when this is false, so the decision page built from it has to carry that ask by hand." + ("" if ok else " lens failed: " + out))

# ================================================================ 31. the recorder's report accounts for every owed record (D0492 / issue568)
# The Decision's fields as the guard reads them; the checker's two new refusals, its --owed flag and the eight-row pair
# table in source; the checker run live over the three sprint 723 fixtures, with and without --owed; the Issue and its
# resolver; sprint 724's record; the brief and the process step that name the count. Nothing typed.
_d0492 = _dec_file("0492-")
_f492 = _decision_facts(_d0492, "d0492")
_ck92 = read(_ckp) or ""
_ck92_claude = read(os.path.join(REPO, ".claude", "skills", "delegated-ceremony", "references", "check_report.py")) or ""
_pairs92_src = re.search(r"^PAIRS = \[(.*?)^\]", _ck92, re.S | re.M)
_pairs92 = re.findall(r'\("([^"]+)",\s*(None|"[^"]*"),\s*(None|\d+)\)', _pairs92_src.group(1)) if _pairs92_src else []
_ref92_m = re.search(r"^def refusals\(.*?(?=^def |\Z)", _ck92, re.S | re.M)  # the Python function, to the next def
_ref92 = _ref92_m.group(0) if _ref92_m else ""


def _ck_run(fixture, owed=None):
    _args = [sys.executable, _ckp, os.path.join(_fx_dir, fixture), "--root", "."] + (["--owed", str(owed)] if owed is not None else [])
    _rc, _o = run_rc(_args, timeout=120)
    _ls = [l for l in (_o or "").strip().splitlines() if l.strip()]
    return {"exit": _rc, "lines": len(_ls), "firstLine": _ls[0] if _ls else "", "namedRefusal": next((l.strip() for l in _ls[1:] if l.startswith("  ")), "")}


_probe92_rc, _probe92_out = run_rc([sys.executable, _ckp, "--probe", "--root", "."], timeout=120)
_probe92_lines = (_probe92_out or "").strip().splitlines()
_s724 = _sprint_facts("sprint724_recorderReportAccountsForEveryOwedRecord.sysml", "d0492")
_proc92 = read(os.path.join(REPO, ".engine", "processes", "delegated-ceremony.sysml")) or ""
_skill92 = read(os.path.join(REPO, ".engine", "skills", "delegated-ceremony", "SKILL.md")) or ""
_skill92_claude = read(os.path.join(REPO, ".claude", "skills", "delegated-ceremony", "SKILL.md")) or ""
_i568 = _issue_facts("568", "dcRecorderReportAccountsForEveryOwedRecord")
_nw_eb = re.search(r"action def EngineBuild \{(.*?)^    \}", _bl, re.S | re.M)
_eb_actions = re.findall(r"^\s{8}action (\w+);", _nw_eb.group(1), re.M) if _nw_eb else []
fact("recorderOwedControl", {
    **_f492,
    "namesIssue568": "issue568" in (_f492["context"] or ""),
    "namesFirstReportPassed": "check_report.py passed it" in (_f492["context"] or ""),
    "namesReminder": "D0047" in (_f492["context"] or ""),
    "namesBeforeAfter": "exit 0" in (_f492["rationale"] or "") and "exit 1" in (_f492["rationale"] or ""),
    "notAForkInRationale": "NOT A FORK:" in (_f492["rationale"] or ""),
    "measuredPhraseInRationale": "MEASURED on this Windows 11 host:" in (_f492["rationale"] or ""),
    "singleProbeRow": bool(re.search(r"^def probe\(root, only=None\):", _ck92, re.M)),
    "source": {
        "owedFlagParsed": '"--owed" in argv' in _ck92,
        "refusalsTakeOwed": bool(re.search(r"^def refusals\(report_text, declared, owed=None\):", _ck92, re.M)),
        "nothingWrittenRefusal": "wrote == 0 and refused == 0" in _ref92,
        "shortfallRefusal": "wrote + refused < owed" in _ref92,
        "refusedLinePattern": bool(re.search(r"^REFUSED_LINE = re\.compile\(", _ck92, re.M)),
        "refusalKinds": len(re.findall(r"found\.append\(", _ref92)),
        "pairs": len(_pairs92), "positives": len([p for p in _pairs92 if p[1] != "None"]), "negatives": len([p for p in _pairs92 if p[1] == "None"]),
        "pairsUnderOwed": len([p for p in _pairs92 if p[2] != "None"]),
        "sprint723Fixtures": sorted(p[0] for p in _pairs92 if "sprint723" in p[0]),
        "fixturesOnDisk": len([p for p in _pairs92 if os.path.exists(os.path.join(_fx_dir, p[0]))]),
        "claudeCopyIdentical": bool(_ck92) and _ck92 == _ck92_claude,
    },
    "live": {
        "probeExit0": _probe92_rc == 0, "probeLastLine": _probe92_lines[-1] if _probe92_lines else "",
        "pairsHolding": len([l for l in _probe92_lines if l.startswith("probe: known-") and ("-> PASS" in l or "-> REFUSED naming" in l)]),
        "nothingWrittenNoOwed": _ck_run("positive-sprint723-nothing-written.txt"),
        "nothingWrittenOwed7": _ck_run("positive-sprint723-nothing-written.txt", 7),
        "sevenOwed7": _ck_run("negative-sprint723-seven-owed.txt", 7),
        "sevenNoOwed": _ck_run("negative-sprint723-seven-owed.txt"),
        "sixOfSevenOwed7": _ck_run("positive-sprint723-six-of-seven.txt", 7),
        "sixOfSevenNoOwed": _ck_run("positive-sprint723-six-of-seven.txt"),
    },
    "issue568": _i568,
    "resolverPosition": (_eb_actions.index("dcRecorderReportAccountsForEveryOwedRecord") + 1) if "dcRecorderReportAccountsForEveryOwedRecord" in _eb_actions else None,
    "engineBuildItems": len(_eb_actions),
    "sprint724": _s724,
    "surfaces": {
        "briefRuleSix": "(6) every owed record is accounted for" in _skill92,
        "briefPassesOwed": "--owed <COUNT>" in _skill92,
        "briefListsSevenRefusals": bool(re.search(r"^7\. under `--owed N`", _skill92, re.M)),
        "briefClaudeIdentical": bool(_skill92) and _skill92 == _skill92_claude,
        "processDispatchNamesCount": "which the recorder passes to the report check as --owed (D0492)" in _proc92,
        "processReportNamesRefusals": "accounts for fewer than N records" in _proc92,
        "processNamesD0492": _proc92.count("D0492"),
    },
} if _d0492 and _ck92 else None, "the owed-count control: Decision, checker source, checker live, Issue, sprint and surfaces",
     _DEC_HOW + " Names by literal search in the field named (notAFork/measuredToken read the guard's fields and tokens; this Decision "
     "carries `NOT A FORK:` and `MEASURED on this Windows 11 host:` in its rationale, so the two ...InRationale keys are the ones that hold). "
     "singleProbeRow = `def probe(root, only=None):` in the checker. source: the checker's text - `\"--owed\" in argv` in main, the refusals() "
     "signature, the two predicates `wrote == 0 and refused == 0` / `wrote + refused < owed` inside refusals(), the compiled REFUSED_LINE, "
     "`found.append(` calls in refusals(), the three-tuple PAIRS rows (fixture, expectation, owed); claudeCopyIdentical = byte equality with "
     "the .claude copy. live: `python check_report.py --probe --root .` exit and last line, pairsHolding = probe lines reading `-> PASS` or "
     "`-> REFUSED naming`; then the checker run over each sprint 723 fixture with and without --owed 7 - exit code, line count, first line, "
     "and the first indented refusal line. issue568 from .tracking/issues-claudeFable5.sysml with its #Resolves edge and the resolver's DoD "
     "line; resolverPosition = the resolver's 1-based place among `action x;` lines in EngineBuild (declaration order IS priority, D0052). "
     "sprint724 = results / outcomes / shas / charter from its delivery file. surfaces by literal search in the recorder brief (SKILL.md, "
     "and byte equality with its .claude copy) and in .engine/processes/delegated-ceremony.sysml.")

# ================================================================ 32. the verifier's stems row names the embedded tree (D0494 / issue530)
# The Decision's fields as the guard reads them; the corrected row in the test-verify skill and its .claude copy;
# the binary's rule in touched.rs (embedded_stem and its own test) and that test run live; the two receipts on disk
# (the sprint 725 verifier's ladder, the landing run over this commit) with the .engine paths each attributed `init`
# from; the Issue whose procedure half this is; the two red-yield obligations the Decision triages; sprint 725.
_d0494 = _dec_file("0494-")
_f494 = _decision_facts(_d0494, "d0494")
_tv94 = read(os.path.join(REPO, ".engine", "skills", "test-verify", "SKILL.md")) or ""
_tv94_claude = read(os.path.join(REPO, ".claude", "skills", "test-verify", "SKILL.md")) or ""
_stems94 = next((l for l in _tv94.splitlines() if l.startswith("| `stems` |")), "")
_touched94 = read(_module_home("touched") or "") or ""  # issue559/issue574: resolved, never anchored
_emb94_m = re.search(r"^pub fn embedded_stem\(path: &str\) -> Option<String> \{.*?^\}", _touched94, re.S | re.M)
_emb94 = _emb94_m.group(0) if _emb94_m else ""
_emb94_line = (_touched94[: _emb94_m.start()].count("\n") + 1) if _emb94_m else None
_emb94_test = re.search(r"^\s*fn a_change_under_the_embedded_tree_names_init\(\)", _touched94, re.M)
_emb94_test_line = (_touched94[: _emb94_test.start()].count("\n") + 1) if _emb94_test else None


def _receipt94(fname):
    _t = read(os.path.join(REPO, ".keel", "metrics", fname)) or ""
    if not _t:
        return {"exists": False}
    _stems_m = re.search(r'^stems = \[([^\]]*)\]', _t, re.M)
    _stems = re.findall(r'"([^"]+)"', _stems_m.group(1)) if _stems_m else None
    _head = (re.search(r'^head = "([^"]+)"', _t, re.M) or [None, None])[1]
    _out = (re.search(r'^outcome = "([^"]+)"', _t, re.M) or [None, None])[1]
    _r = {"exists": True, "head": _head, "outcome": _out, "stems": _stems, "stemsIncludeInit": bool(_stems and "init" in _stems),
          "passed": (re.search(r"^passed = (\d+)", _t, re.M) or [None, None])[1],
          "failed": (re.search(r"^failed = (\d+)", _t, re.M) or [None, None])[1]}
    # a verify receipt (D0476) is a ladder: its [[rung]] blocks carry the verdicts, the touched receipt carries the run
    _rungs = re.findall(r'^\[\[rung\]\]\nname = "(\w+)"\nverdict = "(\w+)"', _t, re.M)
    if _rungs:
        _r["rungs"] = {n: vd for n, vd in _rungs}
        _r["rungsGreen"] = len([1 for _n, vd in _rungs if vd == "pass"])
        _r["stoppedAt"] = (re.search(r'^stopped_at = "([^"]+)"', _t, re.M) or [None, None])[1]
    return _r


_vr94 = _receipt94("verify-receipt.toml")
_tr94 = _receipt94("touched-receipt.toml")
# the .engine paths in the commit each receipt is over (a landed run's change set is that commit against its parent);
# `running` names a run in flight, so its attribution is not yet a verdict and is reported as such
_eng94 = {}
if _tr94.get("head"):
    _ok94, _o94 = run(["git", "show", "--name-only", "--format=", _tr94["head"], "--", ".engine"])
    _eng94["landing"] = sorted(l for l in _o94.splitlines() if l.strip()) if _ok94 else None
else:
    _eng94["landing"] = None
_ok94s, _o94s = run(["git", "status", "--short", "--", ".engine"])
_eng94["workingTree"] = sorted(l for l in _o94s.splitlines() if l.strip()) if _ok94s else None
# the binary's own test for the rule, run live (the lib test binary is the one keel land just built; ~a minute cold)
_t94_rc, _t94_out = run_rc(["cargo", "test", "--release", "--manifest-path", "keel-cli/Cargo.toml", "--lib", "--",
                            "touched::tests::a_change_under_the_embedded_tree_names_init"], timeout=600)
_t94_res = re.search(r"test result: (\w+)\. (\d+) passed; (\d+) failed", _t94_out or "")
_i530 = _issue_facts("530", "dcTouchedReceiptStatesItsOwnAttribution")
_i530_body = (re.search(r"part issue530 : Issue\s*\{(.*?)\n\s*\}", _iss, re.S) or [None, ""])[1]


def _obligation94(hex8):
    _p = os.path.join(REPO, ".tracking", "obligations", "red-yield-" + hex8 + ".sysml")
    _t = read(_p) or ""
    return {"exists": bool(_t),
            "resolvesFromD0494": ("#Resolves dependency from d0494 to obligation" + hex8 + ";") in _t,
            "namesLockedFile": ".engine/skills/test-verify/SKILL.md" in _t,
            "namesProcessChangeGuard": "[process-change] locked file(s) changed" in _t,
            "namedInConsequences": ("obligation" + hex8) in (_f494["consequences"] or "")}


_nw94 = re.search(r"action def NextWork \{(.*?)^    \}", _bl, re.S | re.M)
_nw94_actions = re.findall(r"^\s{8}action (\w+);", _nw94.group(1), re.M) if _nw94 else []
_s725 = _sprint_facts("sprint725_touchedRunIsExclusive.sysml", "d0493")
fact("verifierStemsRow", {
    **_f494,
    "namesIssue530": "issue530" in (_f494["context"] or ""),
    "namesEmbeddedStem": "embedded_stem" in (_f494["context"] or ""),
    "namesSprint725Mismatch": "MISMATCH" in (_f494["context"] or "") and "865 passed" in (_f494["context"] or ""),
    "namesReceiptHalfRemains": "dcTouchedReceiptStatesItsOwnAttribution" in (_f494["rationale"] or ""),
    "saysProcessChange": "This is a process-change" in (_f494["rationale"] or ""),
    "decisionNamesGitStatus": "git status --short -- .engine" in (_f494["decision"] or ""),
    "skill": {
        "rowFound": bool(_stems94),
        "rowNamesInit": "PLUS `init`" in _stems94,
        "rowNamesEngineTree": "`.engine/`" in _stems94,
        "rowNamesEmbeddedStem": "embedded_stem" in _stems94,
        "rowNamesIssue530": "issue530" in _stems94,
        "rowNamesGitStatus": "git status --short -- .engine" in _stems94,
        "rowLength": len(_stems94),
        "claudeCopyIdentical": bool(_tv94) and _tv94 == _tv94_claude,
        "stemsRowsInSkill": len([l for l in _tv94.splitlines() if l.startswith("| `stems` |")]),
    },
    "binary": {
        "embeddedStemFound": bool(_emb94), "embeddedStemLine": _emb94_line,
        "stripsEnginePrefix": 'strip_prefix(".engine/")' in _emb94,
        "returnsInit": 'Some("init".to_string())' in _emb94,
        "bareTreeIsNone": "if rel.is_empty()" in _emb94 and "return None" in _emb94,
        "ownTestFound": bool(_emb94_test), "ownTestLine": _emb94_test_line,
    },
    "live": {
        "ownTest": {"exit": _t94_rc, "result": _t94_res.group(1) if _t94_res else None,
                    "passed": int(_t94_res.group(2)) if _t94_res else None, "failed": int(_t94_res.group(3)) if _t94_res else None},
        "verifierReceipt": _vr94,
        "landingReceipt": _tr94, "landingCommitEnginePaths": _eng94["landing"],
        "workingTreeEngineChanges": _eng94["workingTree"],
    },
    "issue530": {**_i530,
                 # the resolver's DoD opens `issue530 is resolved - ...`, a phrasing the shared helper's `Resolves issueNNN` search misses
                 "resolverDodNamesIssue": bool(re.search(r"dcTouchedReceiptStatesItsOwnAttributionDoD[^\n]*procedureText = \"issue530 is resolved", _bl)),
                 "namesSkillTable": "test-verify skill's receipt-honesty table" in _i530_body,
                 "namesMismatch": "MISMATCH" in _i530_body},
    "resolverPosition": (_nw94_actions.index("dcTouchedReceiptStatesItsOwnAttribution") + 1) if "dcTouchedReceiptStatesItsOwnAttribution" in _nw94_actions else None,
    "nextWorkItems": len(_nw94_actions),
    "obligations": {"obligation1ab52489": _obligation94("1ab52489"), "obligation87f79374": _obligation94("87f79374")},
    "sprint725": {k: v for k, v in _s725.items() if k != "text"},
} if _d0494 and _tv94 else None, "the stems-row correction: Decision, skill row, binary rule, receipts live, Issue, obligations, sprint",
     _DEC_HOW + " Names by literal search in the field named. skill: the one line of .engine/skills/test-verify/SKILL.md beginning "
     "`| `stems` |`, searched for `PLUS `init``, `.engine/`, `embedded_stem`, `issue530`, `git status --short -- .engine`; "
     "claudeCopyIdentical = byte equality with .claude/skills/test-verify/SKILL.md. binary: the `pub fn embedded_stem` body in "
     "keel-cli/src/touched.rs (its 1-based line), searched for the strip_prefix, the `Some(\"init\")` return and the empty-rel None; "
     "ownTestLine = the line of `fn a_change_under_the_embedded_tree_names_init`. live.ownTest = `cargo test --release --lib -- "
     "touched::tests::a_change_under_the_embedded_tree_names_init` exit and its `test result:` line. verifierReceipt / landingReceipt "
     "= head, outcome, stems, passed, failed read from .keel/metrics/verify-receipt.toml and touched-receipt.toml (a verify receipt "
     "carries no run of its own: its [[rung]] name/verdict pairs, rungsGreen and stopped_at are read instead); landingCommitEnginePaths = `git show --name-only --format= <head> -- .engine`, "
     "the .engine paths the commit that receipt names changed; workingTreeEngineChanges = `git status --short -- .engine` now. "
     "issue530 from .tracking/issues-claudeFable5.sysml with its #Resolves edge, resolverDodNamesIssue = the resolver's DoD "
     "procedureText opening `issue530 is resolved`, and two literal searches of its description. resolverPosition = the resolver's 1-based place among `action x;` lines in NextWork (declaration order IS "
     "priority, D0052). obligations: each red-yield file's existence, its `#Resolves dependency from d0494 to obligationX;` edge, "
     "the locked file and guard it names, and whether D0494's consequences name it. sprint725 = results / outcomes / shas / charter "
     "from its delivery file.")

# ================================================================ 33. clippy lints the triple CI lints (D0495 / issue572)
# The Decision's fields as the guard reads them; the pure control in verify.rs and its two tests; the hook's second
# clippy and its GATE CANNOT RUN branch; the skill sentence and its .claude copy; this host's triple and installed
# targets; the ladder receipt's clippy rung (both command lines, its seconds) and the probe/touched rungs; the four
# Issues of the local-gate-differs-from-CI class and their resolvers; sprint 726.
_d0495 = _dec_file("0495-")
_f495 = _decision_facts(_d0495, "d0495")
_verify95 = read(_module_home("verify") or "") or ""
_ohl95_m = re.search(r"^pub fn other_host_lint\(host_is_ci: bool, installed: &\[&str\]\) -> OtherHostLint \{.*?^\}", _verify95, re.S | re.M)
_ohl95 = _ohl95_m.group(0) if _ohl95_m else ""
_ohl95_line = (_verify95[: _ohl95_m.start()].count("\n") + 1) if _ohl95_m else None
_hook95 = read(os.path.join(REPO, ".githooks", "pre-commit")) or ""
_tv95 = read(os.path.join(REPO, ".engine", "skills", "test-verify", "SKILL.md")) or ""
_tv95_claude = read(os.path.join(REPO, ".claude", "skills", "test-verify", "SKILL.md")) or ""
_ok95h, _rustc95 = run(["rustc", "-vV"])
_host95 = (re.search(r"^host: (\S+)", _rustc95 or "", re.M) or [None, None])[1] if _ok95h else None
_ok95t, _targets95 = run(["rustup", "target", "list", "--installed"])
_installed95 = sorted(l.strip() for l in (_targets95 or "").splitlines() if l.strip()) if _ok95t else None
_vr95 = _receipt94("verify-receipt.toml")
_vr95_text = read(os.path.join(REPO, ".keel", "metrics", "verify-receipt.toml")) or ""
_clippy95 = re.search(r'^\[\[rung\]\]\nname = "clippy"\nverdict = "(\w+)"\nexit = (\d+)\nseconds = (\d+)\ncommand = "([^"]*)"', _vr95_text, re.M)
_rung95_secs = {n: int(s) for n, s in re.findall(r'^\[\[rung\]\]\nname = "(\w+)"\nverdict = "\w+"\nexit = \d+\nseconds = (\d+)', _vr95_text, re.M)}
_tr95 = _receipt94("touched-receipt.toml")
_i572, _i573 = _issue_facts("572", "dcClippyLintsTheOtherHostsCfg"), _issue_facts("573", "dcSprintPurposeIsNotPrefixed")
_i574, _i575 = _issue_facts("574", "dcScriptProbesRunBeforeCommit"), _issue_facts("575", "dcVerifyWaitsForItsPid")
_s726 = _sprint_facts("sprint726_clippyLintsTheTripleCiLints.sysml", "d0495")
_bl95_actions = re.findall(r"^\s{8}action (\w+);", _bl, re.M)          # every def, in declaration order (D0052)
_bl95_defs = {}
for _m95 in re.finditer(r"^    action def (\w+) \{(.*?)^    \}", _bl, re.S | re.M):
    for _a95 in re.findall(r"^\s{8}action (\w+);", _m95.group(2), re.M):
        _bl95_defs.setdefault(_a95, _m95.group(1))
_ci95_ok, _ci95_out = run(["gh", "run", "list", "--limit", "6", "--json", "conclusion,status,headSha,event,databaseId,createdAt"], timeout=60)
try:
    _ci95 = json.loads(_ci95_out) if _ci95_ok else None
except ValueError:
    _ci95 = None
fact("clippyLintsCiTriple", {
    **_f495,
    "namesIssue572": "issue572" in (_f495["context"] or ""),
    "namesTouchedLine": "touched.rs:662" in (_f495["context"] or ""),
    "namesSecondInstance": "this is the second" in (_f495["context"] or ""),
    "namesD0098": "D0098" in (_f495["decision"] or ""),
    "namesRemedy": "rustup target add x86_64-unknown-linux-gnu" in (_f495["decision"] or ""),
    "saysProcessChange": "This is a process-change" in (_f495["rationale"] or ""),
    "binary": {
        "constFound": 'pub const CI_TRIPLE: &str = "x86_64-unknown-linux-gnu";' in _verify95,
        "controlFound": bool(_ohl95), "controlLine": _ohl95_line,
        "controlOnceForCi": "if host_is_ci" in _ohl95 and "OtherHostLint::Once" in _ohl95,
        "controlAgainWhenInstalled": "OtherHostLint::Again" in _ohl95,
        "controlRefusesWithRemedy": "OtherHostLint::Refused" in _ohl95 and "rustup target add" in _ohl95 and "issue572" in _ohl95,
        "positiveTestFound": "fn a_host_without_the_ci_triples_std_is_refused_with_the_remedy()" in _verify95,
        "negativeTestFound": "fn a_host_with_the_ci_triples_std_lints_again_and_the_ci_triple_once()" in _verify95,
        "rungRunsWorkspace": "--workspace" in _verify95 and "--all-targets" in _verify95,
    },
    "hook": {
        "secondClippyFound": 'cargo clippy --workspace --all-targets --target "$ci_triple" -- -D warnings' in _hook95,
        "hostFromRustc": "rustc -vV" in _hook95,
        "installedTest": 'rustup target list --installed' in _hook95,
        "cannotRunBranch": "GATE CANNOT RUN" in _hook95 and "rustup target add $ci_triple" in _hook95,
        "namesD0495": "D0495" in _hook95,
    },
    "skill": {
        "namesSecondCommand": "--target x86_64-unknown-linux-gnu" in _tv95,
        "namesRedRung": "RED rung naming `rustup target add`" in _tv95,
        "claudeCopyIdentical": bool(_tv95) and _tv95 == _tv95_claude,
    },
    "host": {"triple": _host95, "isCiTriple": _host95 == "x86_64-unknown-linux-gnu",
             "installedTargets": _installed95, "ciStdInstalled": bool(_installed95 and "x86_64-unknown-linux-gnu" in _installed95)},
    "live": {
        "verifierReceipt": _vr95,
        "clippyRung": {"verdict": _clippy95.group(1), "exit": int(_clippy95.group(2)), "seconds": int(_clippy95.group(3)),
                       "command": _clippy95.group(4),
                       "namesBothTriples": _clippy95.group(4).count("cargo clippy") == 2 and "--target x86_64-unknown-linux-gnu" in _clippy95.group(4)} if _clippy95 else None,
        "rungSeconds": _rung95_secs,
        "touchedReceipt": _tr95,
    },
    "issues": {"issue572": _i572, "issue573": _i573, "issue574": _i574, "issue575": _i575},
    "classMembers": [n for n, i in (("issue453", None), ("issue572", _i572), ("issue574", _i574)) if i is None or i["exists"]],
    "resolverPositions": {a: ({"place": _bl95_actions.index(a) + 1, "def": _bl95_defs.get(a)} if a in _bl95_actions else None)
                          for a in ("dcClippyLintsTheOtherHostsCfg", "dcSprintPurposeIsNotPrefixed", "dcScriptProbesRunBeforeCommit", "dcVerifyWaitsForItsPid")},
    "backlogItems": len(_bl95_actions),
    "sprint726": {k: v for k, v in _s726.items() if k != "text"},
    "ci": _ci95,
} if _d0495 and _verify95 else None, "the other-host clippy: Decision, pure control and tests, hook branch, skill sentence, host, receipts, the class's Issues, sprint",
     _DEC_HOW + " Names by literal search in the field named. binary: keel-cli/src/verify.rs (resolved by scripts/module_home.py) searched for the "
     "CI_TRIPLE const, the `pub fn other_host_lint` body (its 1-based line) and its three arms, the two test fns and `--workspace`. hook: "
     ".githooks/pre-commit searched for the `--target \"$ci_triple\"` clippy, `rustc -vV`, `rustup target list --installed`, the GATE "
     "CANNOT RUN branch naming `rustup target add`, and `D0495`. skill: .engine/skills/test-verify/SKILL.md searched for the `--target` "
     "command and the RED-rung sentence; claudeCopyIdentical = byte equality with the .claude copy. host: `rustc -vV` host line; "
     "`rustup target list --installed` sorted. live: .keel/metrics/verify-receipt.toml as the D0494 reader parses it, plus the clippy "
     "[[rung]] row's verdict/exit/seconds/command (namesBothTriples = two `cargo clippy` and one `--target x86_64-unknown-linux-gnu`) and "
     "every rung's seconds; touched-receipt.toml likewise. issues: each from .tracking/issues-claudeFable5.sysml with its #Resolves edge "
     "and whether the resolver's DoD names it. classMembers = the local-gate-differs-from-CI Issues (issue453 by citation in D0495's "
     "context, the others by existence). resolverPositions = 1-based place among every `action x;` in .tracking/backlog.sysml (declaration order IS "
     "priority, D0052) and the `action def` that declares it; backlogItems = that count. sprint726 from its delivery file. ci = `gh run list --limit 6 --json conclusion,status,headSha,event,databaseId,createdAt` "
     "verbatim (D0420: the conclusion field, never a wrapper's exit); null when gh is unavailable.")

# ================================================================ 34. one script-probe runner for both surfaces (D0496 / issue574)
# The Decision's fields as the guard reads them; the runner in source (its marker, functions and the pair's two
# temp-tree cases); ci.yml's step as one line with no heredoc; the hook block with its staged-path test and its
# GATE CANNOT RUN branch; the runner run live twice (--probe, then the real tree: every probe it discovered and the
# wall-clock); the two CI runs that failed on the heredoc step; the class Issues and their resolvers; sprint 727.
_d0496 = _dec_file("0496-")
_f496 = _decision_facts(_d0496, "d0496")
_runner96_path = os.path.join(REPO, "scripts", "script_probes.py")
_runner96 = read(_runner96_path) or ""
_ci96 = read(os.path.join(REPO, ".github", "workflows", "ci.yml")) or ""
_ci96_step = re.search(r"^      - name: script probes.*?^      - name: |^      # Layer 3", _ci96, re.S | re.M)
_ci96_step_text = _ci96_step.group(0) if _ci96_step else ""
_hook96 = read(os.path.join(REPO, ".githooks", "pre-commit")) or ""
_hook96_block = re.search(r"^# SCRIPT PROBES \(D0496/issue574\).*?^fi\n", _hook96, re.S | re.M)
_hook96_text = _hook96_block.group(0) if _hook96_block else ""
_t96 = time.time()
_ok96p, _out96p = run([sys.executable, "scripts/script_probes.py", "--probe"], timeout=120)
_probe96_secs = round(time.time() - _t96, 1)
_t96 = time.time()
_ok96r, _out96r = run([sys.executable, "scripts/script_probes.py", "."], timeout=300)
_run96_secs = round(time.time() - _t96, 1)
_run96_probes = re.findall(r"^== (\S+) (\S+)$", _out96r or "", re.M)
_run96_verdict = (re.search(r"^(\d+ script probe\(s\) passed|SCRIPT PROBES FAILED:.*)$", _out96r or "", re.M) or [None, None])[1]
_i574b = _issue_facts("574", "dcScriptProbesRunBeforeCommit")
_s727 = _sprint_facts("sprint727_scriptProbesRunBeforeCommit.sysml", "d0496")
_ci96_ok, _ci96_out = run(["gh", "run", "list", "--limit", "12", "--json", "conclusion,status,headSha,event,databaseId,createdAt"], timeout=60)
try:
    _ci96_runs = json.loads(_ci96_out) if _ci96_ok else None
except ValueError:
    _ci96_runs = None
_ci96_failed_on_heredoc = [r for r in (_ci96_runs or []) if r.get("headSha", "")[:7] in ("eb82b52", "6a493f4")]
fact("scriptProbesOneRunner", {
    **_f496,
    "namesIssue574": "issue574" in (_f496["context"] or ""),
    "namesBothRedCommits": "eb82b52" in (_f496["context"] or "") and "6a493f4" in (_f496["context"] or ""),
    "namesThirdMember": "third member" in (_f496["context"] or ""),
    "namesD0098": "D0098" in (_f496["decision"] or ""),
    "namesNoHeredoc": "no heredoc" in (_f496["decision"] or ""),
    "saysProcessChange": "This is a process-change" in (_f496["rationale"] or ""),
    "runner": {
        "exists": bool(_runner96), "lines": _runner96.count("\n"),
        "carriesOwnMarker": "\n# ci-probe: --probe\n" in _runner96,
        "discoverFound": "def discover(root):" in _runner96 and 'glob.glob(os.path.join(root, "scripts", "**", "*.py"), recursive=True)' in _runner96,
        "runFound": "def run_probes(root, out=print):" in _runner96 and '"target", "release"' in _runner96,
        "reportFound": "def report(ran, failed, out=print):" in _runner96 and "SCRIPT PROBES FAILED:" in _runner96,
        "pairFound": "def probe():" in _runner96 and "scripts/failing.py" in _runner96 and "scripts/unmarked.py" in _runner96,
        "noScriptsDirExits2": "the runner cannot run" in _runner96,
    },
    "ci": {
        "stepFound": bool(_ci96_step_text),
        "stepCallsRunner": "run: python3 scripts/script_probes.py" in _ci96_step_text,
        "stepHasHeredoc": "run: |" in _ci96_step_text,
        "stepLines": _ci96_step_text.count("\n"),
        "stepNamesD0496": "D0496" in _ci96_step_text,
        "fileHasScriptProbeDiscovery": "ci-probe" in _ci96.replace(_ci96_step_text, ""),
    },
    "hook": {
        "blockFound": bool(_hook96_text),
        "stagedPattern": (re.search(r"grep -E '([^']+)'", _hook96_text) or [None, None])[1],
        "callsRunner": "python scripts/script_probes.py" in _hook96_text,
        "abortsNamingIssue": "SCRIPT PROBES FAILED" in _hook96_text and "issue574" in _hook96_text,
        "cannotRunBranch": "GATE CANNOT RUN" in _hook96_text and "python not found" in _hook96_text,
        "pythonGatedBlocks": _hook96.count("command -v python"),
    },
    "live": {
        "pair": {"exit0": _ok96p, "seconds": _probe96_secs,
                 "line": (re.search(r"^PROBE (PASS|FAILED).*$", _out96p or "", re.M) or [None, None])[0]},
        "realTree": {"exit0": _ok96r, "seconds": _run96_secs, "verdict": _run96_verdict,
                     "probes": [{"path": p, "flag": f} for p, f in _run96_probes], "count": len(_run96_probes)},
    },
    "issue574": _i574b,
    "classMembers": [n for n, i in (("issue453", None), ("issue572", _i572), ("issue574", _i574b)) if i is None or i["exists"]],
    "classResolverPositions": {a: ({"place": _bl95_actions.index(a) + 1, "def": _bl95_defs.get(a)} if a in _bl95_actions else None)
                               for a in ("dcScriptProbesRunBeforeCommit", "dcVerifyWaitsForItsPid")},
    "ciRunsThatFailedOnTheHeredocStep": _ci96_failed_on_heredoc,
    "sprint727": {k: v for k, v in _s727.items() if k != "text"},
    "ciRuns": _ci96_runs,
} if _d0496 and _runner96 else None, "the one script-probe runner: Decision, runner source and pair, ci.yml step, hook block, both surfaces run live, the class, sprint",
     _DEC_HOW + " Names by literal search in the field named. runner: scripts/script_probes.py searched for its own `# ci-probe: --probe` "
     "line, the four def lines and their distinguishing literals (the recursive glob, the target/release PATH entry, the FAILED banner, the "
     "two temp-tree fixture names) and the exit-2 sentence. ci: .github/workflows/ci.yml's `script probes` step cut from its `- name:` to "
     "the next step or the Layer-3 comment; stepCallsRunner / stepHasHeredoc / stepNamesD0496 are literal searches of that cut; "
     "fileHasScriptProbeDiscovery = `ci-probe` anywhere else in the file (a second copy of the discovery). hook: the block between the "
     "`# SCRIPT PROBES (D0496/issue574)` comment and its closing `fi`; stagedPattern = the grep -E argument; pythonGatedBlocks = count of "
     "`command -v python` in the whole hook. live.pair = `python scripts/script_probes.py --probe` exit, seconds and its PROBE line; "
     "live.realTree = `python scripts/script_probes.py .` exit, seconds, verdict line and every `== path flag` line it printed. issue574 "
     "from .tracking/issues-claudeFable5.sysml with its #Resolves edge and whether the resolver's DoD names it. classMembers as section 33. "
     "classResolverPositions = 1-based place among every `action x;` in .tracking/backlog.sysml and the declaring def (D0052). "
     "ciRunsThatFailedOnTheHeredocStep = the rows of `gh run list --limit 12 --json ...` whose headSha begins eb82b52 or 6a493f4 "
     "(D0420: the conclusion field); ciRuns = that list verbatim; null when gh is unavailable. sprint727 from its delivery file.")

# ================================================================ 35. the wait for a launched ladder is a command (D0497 / issue575)
# The Decision's fields as the guard reads them; the control in source (the three-way WaitState, the pure wait_state,
# the refusal of launch flags beside --wait, the two D0388 tests); the CLI fact and its mirror naming the flag; the
# skill's steps 4 and 5 opening with the command and the .claude copy agreeing; CLAUDE.md's line; the command run
# live twice (the refusal, which launches nothing; --help); issue575 and its resolver; the retro's new item; sprint 728.
_d0497 = _dec_file("0497-")
_f497 = _decision_facts(_d0497, "d0497")
_vr97_path = _module_home("verify")                  # wherever the workspace holds it (issue559/560), never a path typed here
_vr97 = read(_vr97_path or "") or ""
_facts97 = read(os.path.join(REPO, "members", "keel-schema", "src", "cli_facts.rs")) or ""
_cmds97 = read(os.path.join(REPO, ".engine", "cli", "commands.sysml")) or ""
_skill97 = read(os.path.join(REPO, ".engine", "skills", "test-verify", "SKILL.md")) or ""
_skill97c = read(os.path.join(REPO, ".claude", "skills", "test-verify", "SKILL.md")) or ""
_claude97 = read(os.path.join(REPO, "CLAUDE.md")) or ""
_verify97 = re.search(r"^\s*part cliVerify : CliCommand \{.*$", _cmds97, re.M)
_verify97_text = _verify97.group(0) if _verify97 else ""
_mirror97 = re.search(r'^\s*CliFact \{ name: "verify",.*$', _facts97, re.M)
_mirror97_text = _mirror97.group(0) if _mirror97 else ""
def _step_opens_with(md, heading, cmd):
    """True when the first fenced block after `heading` holds exactly `cmd`."""
    m = re.search(re.escape(heading) + r".*?```\n(.*?)\n```", md, re.S)
    return bool(m) and m.group(1).strip() == cmd
_step4_97 = _step_opens_with(_skill97, "### 4. Read the ladder receipt", "KEEL verify --wait .")
_step5_97 = _step_opens_with(_skill97, "### 5. Read the touched receipt", "KEEL verify --wait .")
_ok97r, _out97r = run([KEEL, "verify", "--wait", "--probe", "a,b", "."], timeout=30)
_ok97h, _out97h = run([KEEL, "verify", "--help"], timeout=30)
_i575b = _issue_facts("575", "dcVerifyWaitsForItsPid")
_s728 = _sprint_facts("sprint728_verifyWaitsForItsPid.sysml", "d0497")
fact("verifyWaitsForItsPid", {
    **_f497,
    "namesIssue575": "issue575" in (_f497["context"] or ""),
    "namesSeventeenSeconds": "17 seconds" in (_f497["context"] or ""),
    "namesD0047": "D0047" in (_f497["context"] or ""),
    "namesLaunchesNothing": "launches nothing" in (_f497["decision"] or ""),
    "namesKilledNotAVerdict": "KILLED during <rung>, exit 2" in (_f497["decision"] or ""),
    "saysProcessChange": "This is a process-change" in (_f497["rationale"] or ""),
    "source": {
        "waitStateFound": "pub enum WaitState {" in _vr97,
        "threeStatesPlusFinished": all(s in _vr97 for s in ("NoReceipt,", "InFlight {", "Killed {", "Finished(Ladder)")),
        "pureControlFound": "pub fn wait_state(text: &str, alive: impl Fn(u32) -> bool) -> WaitState {" in _vr97,
        "livenessDecides": "Some(rung) if l.pid != 0 && alive(l.pid) => WaitState::InFlight" in _vr97,
        "deadWriterIsKilled": "Some(rung) => WaitState::Killed" in _vr97,
        "oneReportForBothSurfaces": _vr97.count("report(&ladder)") >= 1 and "fn report(ladder: &Ladder) -> i32" in _vr97,
        "launchFlagsRefused": "belongs to the launch, not the wait" in _vr97,
        "killedLine": "KILLED during {} - {RECEIPT} says running and its writer (pid {pid}) is gone. Not a verdict on either side" in _vr97,
        "probeTests": [t for t in ("a_running_stub_whose_writer_is_gone_is_killed_not_a_verdict",
                                    "a_finished_receipt_is_the_verdict_and_a_live_writer_is_in_flight") if f"fn {t}()" in _vr97],
    },
    "cliFact": {
        "found": bool(_verify97_text),
        "invocationNamesWait": "| --wait [ROOT]" in _verify97_text,
        "synopsisNamesIssue575": "issue575" in _verify97_text,
        "mirrorFound": bool(_mirror97_text),
        "mirrorNamesWait": "| --wait [ROOT]" in _mirror97_text,
        "mirrorNamesIssue575": "issue575" in _mirror97_text,
    },
    "skill": {
        "step4OpensWithWait": _step4_97, "step5OpensWithWait": _step5_97,
        "step5ProseWaitGone": "Wait until the process is gone" not in _skill97,
        "claudeCopyAgrees": _skill97 == _skill97c,
        "waitMentions": _skill97.count("verify --wait"),
    },
    "claudeMdLine": (re.search(r"^keel verify --wait \[ROOT\].*$", _claude97, re.M) or [None])[0],
    "live": {
        "refusal": {"exit0": _ok97r, "line": (_out97r or "").strip().splitlines()[-1] if (_out97r or "").strip() else None,
                    "namesTheLaunch": "belongs to the launch, not the wait" in (_out97r or "")},
        "help": {"exit0": _ok97h, "namesWait": "--wait launches nothing" in (_out97h or ""), "namesIssue575": "issue575" in (_out97h or "")},
    },
    "issue575": _i575b,
    "resolverPositions": {a: ({"place": _bl95_actions.index(a) + 1, "def": _bl95_defs.get(a)} if a in _bl95_actions else None)
                          for a in ("dcVerifyWaitsForItsPid", "dcWaitChecksTheLaunchEpoch", "dcGuardReadFailureIsNotAnEmptyDiff")},
    "sprint728": {k: v for k, v in _s728.items() if k != "text"},
} if _d0497 and _vr97 else None, "the wait as a command: Decision, control in source, CLI fact and mirror, skill steps, CLAUDE.md, live refusal and help, issue575, sprint",
     _DEC_HOW + " Names by literal search in the field named. source: the verify module wherever module_home resolves it, searched for the enum, its four variants, the "
     "wait_state signature, the two match arms that separate a live writer from a dead one, the shared report fn and its call, the "
     "refusal text, the KILLED line and the two test fn names. cliFact: the one-line `part cliVerify : CliCommand {...}` of "
     ".engine/cli/commands.sysml and the one-line `CliFact { name: \"verify\", ...}` of members/keel-schema/src/cli_facts.rs, each searched for the "
     "invocation's `| --wait [ROOT]` and for `issue575`. skill: the first fenced block after each step heading in "
     ".engine/skills/test-verify/SKILL.md must be exactly `KEEL verify --wait .`; the old prose opener searched for; claudeCopyAgrees = "
     "byte equality with the .claude copy. claudeMdLine = the CLAUDE.md line beginning `keel verify --wait [ROOT]`. live.refusal = "
     "`keel verify --wait --probe a,b .` (launches nothing, so it is safe at any time) exit and last line; live.help = `keel verify --help` "
     "searched for the --wait sentence. issue575 from .tracking/issues-claudeFable5.sysml with its #Resolves edge and whether the resolver's "
     "DoD names it. resolverPositions as section 34. sprint728 from its delivery file.")


# ================================================================ 36. D0499 - the guard source's own tests follow D0498 (sprint 729, brief 36)
_d0499 = _dec_file("0499-")
_f499 = _decision_facts(_d0499, "d0499")
_d0498 = _dec_file("0498-")
# the Decision describes commit 50e171a against 1ecbef0; guards.rs is read AT 50e171a under the path it had there
# (keel-cli/src/guards.rs became member keel-guards in sprint 733, so the working tree has no file to read for this diff)
_gd99_rel = "keel-cli/src/guards.rs"
_ok99s, _gd99 = run(["git", "show", f"50e171a:{_gd99_rel}"])
_gd99 = _gd99 if _ok99s else ""
_td99_path = _module_home("touched")
_td99 = read(_td99_path or "") or ""
_fs99 = read(os.path.join(REPO, "members", "keel-fs", "src", "fsx.rs")) or ""
_ob99 = read(os.path.join(REPO, ".tracking", "obligations", "red-yield-637916a4.sysml")) or ""
_first_cfg99 = next((i + 1 for i, l in enumerate(_gd99.splitlines()) if l.strip() == "#[cfg(test)]"), None)
_test99 = "\n".join(_gd99.splitlines()[_first_cfg99 - 1:]) if _first_cfg99 else ""
_prod99 = "\n".join(_gd99.splitlines()[:_first_cfg99 - 1]) if _first_cfg99 else _gd99
# the diff the Decision describes: sprint 729's landed commit against the commit before it, this file only
_ok99d, _out99d = run(["git", "diff", "--numstat", "1ecbef0", "50e171a", "--", _gd99_rel or "."])
_ok99h, _out99h = run(["git", "diff", "-U0", "1ecbef0", "50e171a", "--", _gd99_rel or "."])
_hunks99 = [int(m) for m in re.findall(r"^@@ -(\d+)", _out99h or "", re.M)]
_numstat99 = (_out99d or "").split()
_tag_re99 = r"temp_dir\(\)\.join\("
_i577b = _issue_facts("577", "dcWorktreeScratchIsPerProcess")
_s729 = _sprint_facts("sprint729_unitTestScratchIsPerProcess.sysml", "d0498")
_sites99 = 0
for _sub in (("keel-cli", "src"), ("keel-cli", "tests")) + tuple(("members", m, "src") for m in sorted(os.listdir(os.path.join(REPO, "members"))) if os.path.isdir(os.path.join(REPO, "members", m, "src"))):
    for _dp, _, _fns in os.walk(os.path.join(REPO, *_sub)):
        for _fn in _fns:
            if _fn.endswith(".rs"):
                _sites99 += len(re.findall(r"(?<![\w:])(?:keel_fs::|fsx::)?scratch\(", read(os.path.join(_dp, _fn)) or ""))
fact("guardTestsNameScratchPerProcess", {
    **_f499,
    "dependsOnD0498": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0499\s+to\s+d0498\s*;", _d0499)),
    "d0498Status": (re.search(r"status\s*=\s*DecisionStatus::(\w+)", _d0498) or [None, None])[1],
    "namesSevenSites": "Seven of the thirty-six sites" in (_f499["context"] or ""),
    "namesTheLockReadsThePath": "The lock reads the path, not the region" in (_f499["context"] or ""),
    "namesNoPredicateChange": "no guard predicate" in (_f499["decision"] or ""),
    "namesCarveOutRejected": "not a carve-out for test regions" in (_f499["rationale"] or ""),
    "namesLaterDecisionMayNarrow": "a later Decision may narrow the lock" in (_f499["consequences"] or ""),
    "namesObligation": "obligation637916a4" in (_f499["consequences"] or ""),
    "guardsRs": {
        "firstCfgTestLine": _first_cfg99,
        "scratchCallsInTestRegion": len(re.findall(r"keel_fs::scratch\(", _test99)),
        "scratchCallsAboveIt": len(re.findall(r"keel_fs::scratch\(", _prod99)),
        "fixedJoinsInTestRegion": len([m for m in re.finditer(_tag_re99 + r"([^\n]*)", _test99)
                                       if "process::id()" not in m.group(1) and "gen_uuid()" not in m.group(1)]),
        "lockedByName": _gd99_rel is not None and f'"{_gd99_rel}"' in (re.search(r"const GUARD_SOURCE_FILES: &\[&str\] = &\[(.*?)\];", _gd99) or [None, ""])[1],
        "lockMessageFound": "HARD LOCK: process definitions (D0070) AND the enforcement surface" in _gd99,
        "diff": {"insertions": int(_numstat99[0]) if _ok99d and len(_numstat99) >= 2 and _numstat99[0].isdigit() else None,
                 "deletions": int(_numstat99[1]) if _ok99d and len(_numstat99) >= 2 and _numstat99[1].isdigit() else None,
                 "hunkStartLines": _hunks99,
                 "allHunksBelowFirstCfgTest": bool(_hunks99) and _first_cfg99 is not None and min(_hunks99) > _first_cfg99},
    },
    "control": {
        "helperFound": "pub fn scratch(tag: &str) -> std::path::PathBuf {" in _fs99,
        "helperCarriesPid": bool(re.search(r"pub fn scratch\(tag: &str\) -> std::path::PathBuf \{\n[^\n]*std::process::id\(\)[^\n]*\n\}", _fs99)),
        "censusFound": "fn no_test_names_a_scratch_directory_two_processes_could_share()" in _td99,
        "probePair": [t for t in ("the_scratch_census_names_a_fixed_join_in_test_code",
                                   "the_scratch_census_passes_per_process_joins_and_production_sites") if f"fn {t}()" in _td99],
        "scratchSitesInTree": _sites99,
    },
    "obligation": {"exists": bool(_ob99), "resolvedByD0499": bool(re.search(r"#Resolves\s+dependency\s+from\s+d0499\s+to\s+obligation637916a4\s*;", _ob99)),
                   "firstRedWasTheLock": "First problem at yield: keel gate guard: [process-change] locked file(s) changed (keel-cli/src/guards.rs)" in _ob99},
    "issue577": _i577b,
    "resolverPositions": {a: ({"place": _bl95_actions.index(a) + 1, "def": _bl95_defs.get(a)} if a in _bl95_actions else None)
                          for a in ("dcWorktreeScratchIsPerProcess",)},
    "sprint729": {k: v for k, v in _s729.items() if k != "text"},
} if _d0499 and _gd99 else None, "the held record for the guards.rs test-site rewrite: Decision, the file's regions and diff, the D0498 control, the obligation it discharges, issue577, sprint 729",
     _DEC_HOW + " Names by literal search in the field named; dependsOnD0498 = the `#DependsOn dependency from d0499 to d0498;` line. guardsRs: firstCfgTestLine = the "
     "first line that is exactly `#[cfg(test)]`; the test region is that line to EOF and the production region is above it; scratch calls = "
     "`keel_fs::scratch(` in each region; fixedJoinsInTestRegion = `temp_dir().join(` lines in the test region whose rest of line has neither "
     "`process::id()` nor `gen_uuid()` (the census's own rule, touched.rs fixed_scratch_joins); lockedByName = the resolved path, repo-relative, inside GUARD_SOURCE_FILES; "
     "diff = `git diff --numstat 1ecbef0 50e171a -- keel-cli/src/guards.rs` (guardsRs = `git show 50e171a:keel-cli/src/guards.rs`, the file at the commit the Decision describes; it is member keel-guards since sprint 733) and the `@@ -N` starts of `git diff -U0` over the same range, every one "
     "compared against firstCfgTestLine. control: fsx.rs searched for the scratch signature and the pid inside its body; touched.rs for the census and "
     "the two probe fn names (helperCarriesPid = the one-line body between the signature and its closing brace holds `std::process::id()`); scratchSitesInTree = `scratch(` calls (bare, keel_fs:: or fsx::) over every .rs file under keel-cli/src, keel-cli/tests "
     "and members/*/src. obligation from .tracking/obligations/red-yield-637916a4.sysml: its #Resolves edge and whether its description opens on the "
     "process-change lock. issue577 and resolverPositions as section 35. sprint729 from its delivery file, charter d0498.")


# ================================================================ 37. D0500 - the probe pair travels in a file (sprint 730, brief 37)
_d0500 = _dec_file("0500-")
_f500 = _decision_facts(_d0500, "d0500")
_vr00_path = _module_home("verify")
_vr00 = read(_vr00_path or "") or ""
_cmds00 = read(os.path.join(REPO, ".engine", "cli", "commands.sysml")) or ""
_facts00 = read(os.path.join(REPO, "members", "keel-schema", "src", "cli_facts.rs")) or ""
_tv00 = read(os.path.join(REPO, ".engine", "skills", "test-verify", "SKILL.md")) or ""
_tv00c = read(os.path.join(REPO, ".claude", "skills", "test-verify", "SKILL.md")) or ""
_dc00 = read(os.path.join(REPO, ".engine", "skills", "delegated-ceremony", "SKILL.md")) or ""
_dc00c = read(os.path.join(REPO, ".claude", "skills", "delegated-ceremony", "SKILL.md")) or ""
_pr00 = read(os.path.join(REPO, ".engine", "processes", "delegated-ceremony.sysml")) or ""
_claude00 = read(os.path.join(REPO, "CLAUDE.md")) or ""
_verify00 = (re.search(r"^\s*part cliVerify : CliCommand \{.*$", _cmds00, re.M) or [""])[0]
_mirror00 = (re.search(r'^\s*CliFact \{ name: "verify",.*$', _facts00, re.M) or [""])[0]
_vr00_receipt = read(os.path.join(REPO, ".keel", "metrics", "verify-receipt.toml")) or ""
_probe00 = re.search(r'^\[\[rung\]\]\nname = "probe"\nverdict = "(\w+)"\nexit = (\d+)\nseconds = (\d+)\ncommand = "([^"]*)"', _vr00_receipt, re.M)
_probe00_cmd = _probe00.group(4) if _probe00 else ""
_rung00_secs = {n: int(s) for n, s in re.findall(r'^\[\[rung\]\]\nname = "(\w+)"\nverdict = "\w+"\nexit = \d+\nseconds = (\d+)', _vr00_receipt, re.M)}
_vr00_fact = _receipt94("verify-receipt.toml")
# live: the refusals are decided at parse, before any rung and before the receipt is touched (verify.rs `let probe = match parse_probe`
# returns 2), so they are safe to run at any time, land included; the file is written under the scratch rule and removed after
import shutil     # noqa: E402
import tempfile   # noqa: E402
_scratch00 = os.path.join(tempfile.gettempdir(), f"keel-facts-pair-{os.getpid()}")   # per-process, the D0498 rule
os.makedirs(_scratch00, exist_ok=True)
_one00 = os.path.join(_scratch00, "one-line.txt")
_blank00 = os.path.join(_scratch00, "blank-second.txt")
_three00 = os.path.join(_scratch00, "three-lines.txt")
with open(_one00, "w", encoding="utf-8") as _fh:
    _fh.write("cargo test a\n")
with open(_blank00, "w", encoding="utf-8") as _fh:
    _fh.write("cargo test a\n\n")
with open(_three00, "w", encoding="utf-8") as _fh:
    _fh.write("cargo test a\ncargo test b\ncargo test c\n")
_rc00_one, _out00_one = run_rc([KEEL, "verify", ".", "--probe-from", _one00], timeout=30)
_rc00_blank, _out00_blank = run_rc([KEEL, "verify", ".", "--probe-from", _blank00], timeout=30)
_rc00_three, _out00_three = run_rc([KEEL, "verify", ".", "--probe-from", _three00], timeout=30)
_rc00_both, _out00_both = run_rc([KEEL, "verify", ".", "--probe", "a,b", "--probe-from", _one00], timeout=30)
_rc00_wait, _out00_wait = run_rc([KEEL, "verify", "--wait", "--probe-from", _one00, "."], timeout=30)
_rc00_help, _out00_help = run_rc([KEEL, "verify", "--help"], timeout=30)
_receipt00_after = read(os.path.join(REPO, ".keel", "metrics", "verify-receipt.toml")) or ""
shutil.rmtree(_scratch00, ignore_errors=True)


def _last00(out):
    return (out or "").strip().splitlines()[-1] if (out or "").strip() else None


_i571 = _issue_facts("571", "dcProbePairTravelsInAFile")
_i578 = _issue_facts("578", "dcVerifierStopIsNeverABlock")
_s730 = _sprint_facts("sprint730_probePairTravelsInAFile.sysml", "d0500")
_bl00_actions = re.findall(r"^\s{8}action (\w+);", _bl, re.M)
_bl00_defs = {}
for _m00 in re.finditer(r"^    action def (\w+) \{(.*?)^    \}", _bl, re.S | re.M):
    for _a00 in re.findall(r"^\s{8}action (\w+);", _m00.group(2), re.M):
        _bl00_defs.setdefault(_a00, _m00.group(1))
fact("probePairTravelsInAFile", {
    **_f500,
    "namesIssue571": "issue571" in (_f500["context"] or ""),
    "namesThreeDispatches": "a third time" in (_f500["context"] or ""),
    "namesEightySeconds": "80-second climb" in (_f500["context"] or ""),
    "namesD0224": "D0224" in (_f500["context"] or ""),
    "namesExactlyTwoLines": "exactly two non-empty lines" in (_f500["decision"] or ""),
    "namesVerifierNeverTranscribes": "the verifier never transcribes the pair" in (_f500["decision"] or ""),
    "namesBothFlagsRefused": "--probe given beside --probe-from is refused at parse" in (_f500["decision"] or ""),
    "namesTypedPairStays": "stays for a human at a terminal" in (_f500["decision"] or ""),
    "namesD0047": "D0047" in (_f500["rationale"] or ""),
    "namesTwoSecondRefusal": "two-second refusal" in (_f500["rationale"] or ""),
    "saysProcessChange": "This is a process change" in (_f500["rationale"] or ""),
    "namesResolvesIssue571": "Resolves issue571" in (_f500["consequences"] or ""),
    "source": {
        "probeCarriesSource": "pub from: Option<PathBuf>," in _vr00,
        "textNamesTheFile": '|f| format!("--probe-from {}: {} ; {}", f.display(), self.pos, self.neg),' in _vr00,
        "bothFlagsRefused": '(Some(_), Some(_)) => Err("--probe and --probe-from name the same pair twice; give one (D0500)".to_owned()),' in _vr00,
        "exactlyTwoLines": "let [pos, neg] = lines[..] else {" in _vr00,
        "blankLineRefused": "is blank; both lines of a D0388 pair file are command lines" in _vr00,
        "ownArgsSkipsThePath": 'if a == "--probe" || a == "--probe-from" {' in _vr00,
        "waitRefusesIt": '*a == "--probe" || *a == "--probe-from" || *a == "--no-receipt"' in _vr00,
        "parseBeforeAnyRung": ("let probe = match parse_probe(args) {" in _vr00 and "let (steps, stopped_at) = climb(" in _vr00
                               and _vr00.index("let probe = match parse_probe(args) {") < _vr00.index("let (steps, stopped_at) = climb(")),
        "notRunRowNamesThePair": "(Rung::Probe, Some(pair)) => pair.to_owned()," in _vr00,
        "probeTests": [t for t in ("a_probe_file_names_the_pair", "a_probe_file_that_is_not_a_pair_is_refused") if f"fn {t}()" in _vr00],
        "notRunRowTest": 'assert_eq!(steps[3].command, "--probe-from pair.txt: cargo test a ; cargo test b"' in _vr00,
    },
    "cliFact": {
        "found": bool(_verify00), "invocationNamesFlag": "| --probe-from FILE]" in _verify00, "synopsisNamesD0500": "D0500" in _verify00,
        "mirrorFound": bool(_mirror00), "mirrorNamesFlag": "| --probe-from FILE]" in _mirror00, "mirrorNamesD0500": "D0500" in _mirror00,
    },
    "skill": {
        "step1LaunchesFromFile": 'KEEL verify . --probe-from "<ABS PAIR FILE>"' in _tv00,
        "transcribesNothing": "transcribe NOTHING" in _tv00,
        "namesIssue571": "issue571" in _tv00,
        "receiptShapeNamesFlag": "LADDER: KEEL verify . --probe-from <ABS PAIR FILE>" in _tv00,
        "notRunRowForm": "PROBE PAIR: <the row's command> -> not" in _tv00,
        "typedPairMentions": _tv00.count("--probe POS,NEG") + _tv00.count("--probe POSITIVE,NEGATIVE"),
        "claudeCopyAgrees": _tv00 == _tv00c,
        "briefSlotIsTheFile": "the pair file at <ABS PAIR FILE>" in _dc00 and "`--probe-from <ABS PAIR FILE>` and transcribe nothing" in _dc00,
        "briefClaudeCopyAgrees": _dc00 == _dc00c,
        "processStepWritesTheFile": "Write the sprint's D0388 pair as a file of two lines" in _pr00 and "transcribes nothing, D0500" in _pr00,
    },
    "claudeMdLines": len(re.findall(r"--probe-from", _claude00)),
    "live": {
        "oneLine": {"exit": _rc00_one, "line": _last00(_out00_one), "namesTheShape": "1 line(s); a D0388 pair file is exactly two" in (_out00_one or "")},
        "blankSecond": {"exit": _rc00_blank, "line": _last00(_out00_blank), "namesTheLine": "line 2 is blank" in (_out00_blank or "")},
        "threeLines": {"exit": _rc00_three, "line": _last00(_out00_three), "namesTheShape": "3 line(s); a D0388 pair file is exactly two" in (_out00_three or "")},
        "bothFlags": {"exit": _rc00_both, "line": _last00(_out00_both), "namesTwice": "name the same pair twice" in (_out00_both or "")},
        "waitBeside": {"exit": _rc00_wait, "line": _last00(_out00_wait), "namesTheLaunch": "belongs to the launch, not the wait" in (_out00_wait or "")},
        "help": {"exit": _rc00_help, "namesFlag": "--probe-from FILE" in (_out00_help or "")},
        "receiptUntouchedByRefusals": _receipt00_after == _vr00_receipt,
        "verifierReceipt": {**{k: v for k, v in _vr00_fact.items() if k in ("exists", "outcome", "rungs")},
                            "stoppedAt": (re.search(r'^stopped_at = "([^"]+)"', _vr00_receipt, re.M) or [None, None])[1],
                            "rungsGreen": len([v for v in (_vr00_fact.get("rungs") or {}).values() if v == "pass"])},
        "probeRung": {"verdict": _probe00.group(1), "exit": int(_probe00.group(2)), "seconds": int(_probe00.group(3)),
                      "namesAFile": _probe00_cmd.startswith("--probe-from "),
                      "fileStem": _pair_file_stem(_probe00_cmd), "fileStemIsAStem": _pair_file_stem(_probe00_cmd) is not None,
                      "sides": [s.strip() for s in _probe00_cmd.split(": ", 1)[1].split(" ; ")] if ": " in _probe00_cmd else [],
                      "sidesAreTheTests": all(t in _probe00_cmd for t in ("a_probe_file_names_the_pair", "a_probe_file_that_is_not_a_pair_is_refused"))} if _probe00 else None,
        "rungSeconds": _rung00_secs,
    },
    "issue571": _i571,
    "issue578": _i578,
    "resolverPositions": {a: ({"place": _bl00_actions.index(a) + 1, "def": _bl00_defs.get(a)} if a in _bl00_actions else None)
                          for a in ("dcProbePairTravelsInAFile", "dcVerifierStopIsNeverABlock", "dcViewIsAMember")},
    "backlogItems": len(_bl00_actions),
    "sprint730": {k: v for k, v in _s730.items() if k != "text"} | ({"retroNamesIssue578": "issue578" in _s730["text"], "retroNamesNoNewItem": "no new item" in _s730["text"]} if _s730.get("exists") else {}),
} if _d0500 and _vr00 else None, "the held record for the pair file: Decision, the control in source, CLI fact and mirror, the two skills and the process step, CLAUDE.md, live refusals and help, the sprint's own ladder receipt, issue571, issue578, sprint 730",
     _DEC_HOW + " Names by literal search in the field named. source: the verify module wherever module_home resolves it, searched for the Probe field, "
     "the text() arm that prefixes the file, the both-flags Err arm, the two-line slice pattern, the blank-line message, own_args' skip, --wait's "
     "refusal list, the not-run arm of climb, the two probe test fn names and the not-run assertion; parseBeforeAnyRung = parse_probe's call site "
     "sits in the file before the `let (steps, stopped_at) = climb(` call (the Err arm returns 2 before the ladder is entered). cliFact: the one-line `part cliVerify` of "
     ".engine/cli/commands.sysml and the one-line `CliFact { name: \"verify\"` of members/keel-schema/src/cli_facts.rs, each searched for the invocation's "
     "`| --probe-from FILE]` and for `D0500`. skill: literal spans of .engine/skills/test-verify/SKILL.md (the step-1 launch line, `transcribe NOTHING`, "
     "issue571, the LADDER receipt line, the not-run form; typedPairMentions counts the typed spelling), byte equality with the .claude copy; the "
     "delegated-ceremony brief's pair slot and its copy; the process file's dispatch step. claudeMdLines = `--probe-from` occurrences in CLAUDE.md. live: "
     "each refusal is `keel verify . --probe-from <file>` over a file this script writes under the per-process temp dir (one line / a blank second line / "
     "three lines), `--probe a,b --probe-from <file>`, and `--wait --probe-from <file>`; exit from run_rc and the last output line; every one is refused "
     "at parse before a rung runs, so receiptUntouchedByRefusals = .keel/metrics/verify-receipt.toml byte-equal before and after. help = `keel verify --help` "
     "searched for the flag. verifierReceipt/probeRung/rungSeconds read .keel/metrics/verify-receipt.toml: outcome, stopped_at, each [[rung]] verdict, and "
     "the probe row's command split at `: ` then ` ; ` into the file and the two sides. issue571/issue578 from .tracking/issues-claudeFable5.sysml with "
     "their #Resolves edges and whether each resolver's DoD names them. resolverPositions as section 34; backlogItems = every `action x;` in backlog.sysml. "
     "sprint730 from its delivery file, charter d0500, plus literal `issue578` and `no new item` in its text.")
# ================================================================ 38. D0501 + D0502 - a subagent is measured from its own start; a verifier's stop is never a block (sprint 731, brief 38)
_d0501 = _dec_file("0501-")
_d0502 = _dec_file("0502-")
_f501 = _decision_facts(_d0501, "d0501")
_f502 = _decision_facts(_d0502, "d0502")
_d0502_text = _d0502 or ""            # _dec_file returns the TEXT of the record
_main01 = _main_rs
_cs01 = read(_mh("claude_surface") or "") or ""
_census01 = read(_module_home("view/census") or "") or ""
_events01 = read(os.path.join(REPO, ".engine", "contracts", "control-events.toml")) or ""
_cmap01 = read(os.path.join(REPO, ".tracking", "architecture", "control-map.sysml")) or ""
_hook01_fact = (re.search(r"^\s*part cliHook2 : CliCommand \{.*$", _cmds00, re.M) or [""])[0]
_hook01_mirror = (re.search(r'^\s*CliFact \{ name: "hook",.*$', _facts00, re.M) or [""])[0]
_events01_block = (re.search(r"^\[subagent-start\]\n(?:[^\[\n][^\n]*\n)*", _events01, re.M) or [""])[0]
_ledger01_path = os.path.join(REPO, ".keel", "metrics", "hooks.jsonl")
_ledger01 = (read(_ledger01_path) or "").splitlines()


def _ledger01_real(pred):
    """Ledger lines matching `pred`, minus this script's own probe sessions and the hand probes of the sprint."""
    n = 0
    for _ln in _ledger01:
        if not pred(_ln):
            continue
        _sess = (re.search(r'"session":"([^"]*)"', _ln) or [None, ""])[1]
        if _sess.startswith("probe") or _sess.startswith("keel-facts-"):
            continue
        n += 1
    return n


# live: `keel hook <event>` reads its payload from stdin; the session and agent ids are this process's, so the only files
# the probes touch are their own fp files under .keel/metrics (removed after) and their own ledger lines (excluded above)
def _hook01(event, payload):
    try:
        p = subprocess.run([KEEL, "hook", event], cwd=REPO, capture_output=True, text=True, timeout=120,
                           input=json.dumps(payload), encoding="utf-8", errors="replace")
        return p.returncode, (p.stdout or "").strip()
    except Exception as exc:                                        # pragma: no cover
        return 1, "%s: %s" % (type(exc).__name__, exc)


_sess01 = f"keel-facts-{os.getpid()}"
_aid01 = f"facts/{os.getpid()}"                      # the `/` is folded to `_` by agent_baseline_path; the file name proves it
_fp01 = os.path.join(REPO, ".keel", "metrics", f"agent-facts_{os.getpid()}.fp")
_sessfp01 = os.path.join(REPO, ".keel", "metrics", f"baseline-{_sess01}.fp")
_ver01 = {"session_id": _sess01, "agent_id": _aid01, "agent_type": "verifier", "stop_hook_active": False}
_rc01_start, _out01_start = _hook01("subagent-start", _ver01)
_fp01_written = os.path.isfile(_fp01)
_fp01_first = read(_fp01) or ""
_rc01_start2, _ = _hook01("post-edit", {**_ver01, "tool_name": "Edit", "tool_input": {"file_path": "nothing.txt"}})
_fp01_second = read(_fp01) or ""
_rc01_still, _out01_still = _hook01("subagent-stop", _ver01)
try:
    with open(_fp01, "w", encoding="utf-8") as _fh:
        _fh.write("moved-by-facts-py")
except OSError:
    pass
_rc01_moved, _out01_moved = _hook01("subagent-stop", _ver01)
_rc01_none, _out01_none = _hook01("subagent-stop", {"session_id": f"{_sess01}-nobaseline", "agent_id": f"{_aid01}-none", "agent_type": "verifier", "stop_hook_active": False})
_ledger01_after = (read(_ledger01_path) or "").splitlines()
_probe01_lines = [l for l in _ledger01_after if f'"session":"{_sess01}"' in l]
for _p01 in (_fp01, _sessfp01):
    try:
        os.remove(_p01)
    except OSError:
        pass
_i578b = _issue_facts("578", "dcVerifierStopIsNeverABlock")
_s731 = _sprint_facts("sprint731_verifierStopIsNeverABlock.sysml", "d0502")
_blocks01 = [l for l in _ledger01 if '"event":"subagent-stop"' in l and '"decision":"block"' in l]
_blocks01_day = [l for l in _blocks01 if 1789574400 <= int((re.search(r'"ts":(\d+)', l) or [None, "0"])[1]) < 1789660800]
fact("subagentOwnStartAndVerifierStop", {
    "d0501": {
        **_f501,
        "namesIssue578": "issue578" in (_f501["context"] or ""),
        "namesSixBlocks": "six blocks" in (_f501["context"] or ""),
        "namesSessionBaseline": "baseline-{session}.fp" in (_f501["context"] or ""),
        "namesAgentFile": "agent-{agent_id}.fp" in (_f501["decision"] or ""),
        "namesNeverOverwrites": "never overwrites it" in (_f501["decision"] or ""),
        "namesFallback": "the session baseline is read only for a stop payload that names no agent_id" in (_f501["decision"] or ""),
        "namesSubagentStartHome": "claude_surface.rs keel_hooks" in (_f501["decision"] or ""),
        "saysSafetyChange": "This is a safety change" in (_f501["rationale"] or ""),
        "saysNotAFork": "NOT A FORK" in (_f501["rationale"] or ""),
        "namesWrongIf": "Wrong if a subagent's hook fires arrive without agent_id" in (_f501["consequences"] or ""),
    },
    "d0502": {
        **_f502,
        "namesIssue578": "issue578" in (_f502["context"] or ""),
        "namesSixBlocks": "blocked six times" in (_f502["context"] or ""),
        "namesTheInventedFinding": "a retro finding it invented" in (_f502["context"] or ""),
        "namesNeverABlock": "the hook never emits a block" in (_f502["decision"] or ""),
        "namesTheControl": "verifier:tree-written" in (_f502["decision"] or ""),
        "namesRecorderKeepsBlock": "The recorder keeps its block under recorder:tree-red" in (_f502["decision"] or ""),
        "namesD0424": "D0424" in (_f502["rationale"] or ""),
        "saysSafetyChange": "This is a safety change" in (_f502["rationale"] or ""),
        "saysNotAFork": "NOT A FORK" in (_f502["rationale"] or ""),
        "namesWrongIf": "Wrong if a verifier writes the tree and the primary's next orient does not surface the census row" in (_f502["consequences"] or ""),
        "dependsOnD0501": "#DependsOn dependency from d0502 to d0501;" in _d0502_text,
    },
    "source": {
        "baselinePathFolds": "fn agent_baseline_path(root: &Path, agent_id: &str) -> PathBuf {" in _main01 and 'format!("agent-{safe}.fp")' in _main01,
        "dispatcherWritesOnFirstFire": 'if event != "subagent-stop" {' in _main01 and "let bl = agent_baseline_path(&root, agent_id);" in _main01 and "if !bl.exists() {" in _main01,
        "baselineOwnThenSession": "fn subagent_baseline(root: &Path, payload: &serde_json::Value, session: &str) -> Option<String> {" in _main01
                                  and 'own.or_else(|| std::fs::read_to_string(root.join(".keel").join("metrics").join(format!("baseline-{session}.fp"))).ok())' in _main01,
        "routeEnumArms": [a for a in ("NotGated", "Silent", "VerifierTreeWritten", "Gate") if re.search(r"^\s{4}" + a + r",$", _main01, re.M)],
        "routeIsPure": "fn subagent_stop_route(agent_type: Option<&str>, baseline: Option<&str>, now: &str) -> SubagentStopRoute {" in _main01,
        "verifierArmIsTheType": 'if agent_type == Some("verifier") {' in _main01,
        "verifierArmLedgersRefused": 'note_verdict("refused", "verifier:tree-written");' in _main01,
        "verifierArmExitsZero": ("SubagentStopRoute::VerifierTreeWritten => {" in _main01
                                 and re.search(r"SubagentStopRoute::VerifierTreeWritten => \{.*?\n\s{12}0\n\s{8}\}", _main01, re.S) is not None),
        "routeBeforeGate": ("match subagent_stop_route(agent_type, baseline.as_deref(), &now) {" in _main01
                            and "let code = hook_stop(payload, root);" in _main01
                            and _main01.index("match subagent_stop_route(agent_type, baseline.as_deref(), &now) {") < _main01.index("let code = hook_stop(payload, root);")),
        "recorderRelabelKept": "let relabelled = subagent_block_control(agent_type, control);" in _main01,
        "startEventIsCounted": '"subagent-start" => 0,' in _main01,
        "tests": [t for t in ("a_verifier_is_never_gated_and_its_own_write_is_the_ledgered_fact",
                              "a_recorder_and_an_untyped_agent_keep_the_gate_over_a_moved_tree",
                              "the_agents_own_start_is_read_before_the_sessions") if f"fn {t}()" in _main01],
        "claudeSurface": {"registersSubagentStart": '"SubagentStart": [{ "hooks": [hook_entry(' in _cs01 and 'hook subagent-start"' in _cs01,
                          "recognisesTheCommand": 'c.contains("hook subagent-start")' in _cs01,
                          "sevenEventsTest": "fn generated_hooks_have_seven_events_and_fail_loud_resolution()" in _cs01},
        "guardsLedger": "const EMITTED_LEDGER: [&str; 15] = [" in _guards_rs and '"subagent-start"' in _guards_rs,
        "censusHookEvents": '"subagent-start"' in ((re.search(r"^const HOOK_EVENTS: &\[&str\] = &\[.*$", _census01, re.M) or [""])[0]),
    },
    "declared": {
        "controlEvent": {"found": bool(_events01_block), "control": (re.search(r'^control = "([^"]+)"', _events01_block, re.M) or [None, None])[1],
                         "record": (re.search(r'^record = "([^"]+)"', _events01_block, re.M) or [None, None])[1], "namesD0501": "D0501" in _events01_block},
        "cliFact": {"found": bool(_hook01_fact), "invocationNamesStart": "|subagent-start|" in _hook01_fact,
                    "supersedesCliHook": "#Supersede dependency from cliHook2 to cliHook;" in _cmds00,
                    "mirrorFound": bool(_hook01_mirror), "mirrorNamesStart": "|subagent-start|" in _hook01_mirror},
        "controlMap": {"found": "part ctlVerifierTreeWritten : SystemSafetyConstraint {" in _cmap01,
                       "title": (re.search(r'part ctlVerifierTreeWritten : SystemSafetyConstraint \{[^\n]*?title = "([^"]+)"', _cmap01) or [None, None])[1],
                       "dischargesEhz2": "dependency from ctlVerifierTreeWritten to ehz2;" in _cmap01,
                       "hookRuleTitles": len(re.findall(r'title = "hook-rule: ', _cmap01))},
        "process": {"stepNamesBoth": "recorder:tree-red" in _pr00 and "verifier:tree-written" in _pr00,
                    "stepNamesOwnStart": "measured from the fingerprint" in _pr00 and "D0501" in _pr00},
        "skill": {"namesBoth": "`recorder:tree-red`" in _dc00 and "`verifier:tree-written`" in _dc00,
                  "namesNeverBlocked": "a verifier is NEVER blocked (D0502)" in _dc00,
                  "namesOwnStart": "the tree at its OWN start (D0501)" in _dc00,
                  "claudeCopyAgrees": _dc00 == _dc00c},
        "claudeMd": {"namesControl": "`verifier:tree-written`" in _claude00, "namesNeverBlocked": "never blocked (D0501/D0502" in _claude00},
    },
    "live": {
        "start": {"exit": _rc01_start, "silent": _out01_start == "", "wroteOwnFile": _fp01_written, "fileNameFoldsTheSlash": os.path.basename(_fp01),
                  "fingerprintLength": len(_fp01_first.strip())},
        "secondFireLeavesIt": {"exit": _rc01_start2, "unchanged": _fp01_second == _fp01_first and _fp01_first != ""},
        "stopUnmoved": {"exit": _rc01_still, "silent": _out01_still == "", "namesNoControl": "verifier:tree-written" not in _out01_still},
        "stopMoved": {"exit": _rc01_moved, "namesControl": "verifier:tree-written" in _out01_moved, "isSystemMessage": _out01_moved.startswith('{"systemMessage":'),
                      "saysNotABlock": "Not a block" in _out01_moved, "namesD0502": "D0502" in _out01_moved,
                      "line": _out01_moved[:240] or None},
        "stopNoBaseline": {"exit": _rc01_none, "namesNotGated": "subagent tree not gated" in _out01_none},
        "ledger": {"probeLines": len(_probe01_lines),
                   "startLines": len([l for l in _probe01_lines if '"event":"subagent-start"' in l]),
                   "refusedLines": len([l for l in _probe01_lines if '"control":"verifier:tree-written"' in l and '"decision":"refused"' in l]),
                   "blockLines": len([l for l in _probe01_lines if '"decision":"block"' in l])},
        "cleanedUp": not os.path.exists(_fp01) and not os.path.exists(_sessfp01),
    },
    "ledger": {
        "subagentStartFires": _ledger01_real(lambda l: '"event":"subagent-start"' in l),
        "verifierTreeWritten": _ledger01_real(lambda l: '"control":"verifier:tree-written"' in l),
        "recorderTreeRed": _ledger01_real(lambda l: '"control":"recorder:tree-red"' in l),
        "subagentStopBlocksEver": len(_blocks01),
        "subagentStopBlocksOn0916": len(_blocks01_day),
        "agentFpFiles": len([f for f in os.listdir(os.path.join(REPO, ".keel", "metrics")) if f.startswith("agent-") and f.endswith(".fp")]),
    },
    "issue578": _i578b,
    "resolverPositions": {a: ({"place": _bl00_actions.index(a) + 1, "def": _bl00_defs.get(a)} if a in _bl00_actions else None)
                          for a in ("dcVerifierStopIsNeverABlock", "dcViewIsAMember", "dcSuiteIsAMember")},
    "backlogItems": len(_bl00_actions),
    "sprint731": {k: v for k, v in _s731.items() if k != "text"} | ({"retroNamesIssue578": "issue578" in _s731["text"], "retroNamesDispatchOrder": "dispatch order" in _s731["text"],
                                                                     "standupNamesOrder": "the retro is filled BEFORE the verifier is dispatched" in _s731["text"]} if _s731.get("exists") else {}),
} if _d0501 and _d0502 and _main01 else None,
     "the two held records: Decisions, the route and the baseline in source, the seven declaration homes, live hook fires, the ledger, issue578, sprint 731",
     _DEC_HOW + " Names by literal search in the field named; dependsOnD0501 = the `#DependsOn dependency from d0502 to d0501;` edge in D0502's file. source: "
     "main.rs wherever module_home resolves it, searched for agent_baseline_path and its `agent-{safe}.fp`, the dispatcher's first-fire write guarded by "
     "`event != subagent-stop` and `!bl.exists()`, subagent_baseline's own-then-session or_else, the four arms of SubagentStopRoute, the pure route fn, the "
     "verifier arm's type test, its note_verdict(refused, verifier:tree-written) and its literal `0` return, the route match sitting before `hook_stop` in the "
     "file, the recorder relabel, the `subagent-start => 0` dispatch arm, and the three test fn names; claude_surface.rs for the SubagentStart hook_entry, "
     "is_keel_hook's `hook subagent-start`, the seven-events test; guards.rs for `EMITTED_LEDGER: [&str; 15]` naming subagent-start; view/census.rs's one-line "
     "HOOK_EVENTS. declared: the [subagent-start] table of control-events.toml (control, record, D0501); the one-line `part cliHook2` of commands.sysml, the "
     "`#Supersede dependency from cliHook2 to cliHook;` edge, and the one-line `CliFact { name: \"hook\"` mirror; control-map.sysml's ctlVerifierTreeWritten "
     "part, its title, its discharge edge to ehz2, and the count of `hook-rule:` titles; delegated-ceremony.sysml, the skill (byte-equal .claude copy) and CLAUDE.md "
     "by literal spans. live: five `keel hook <event>` fires with a JSON payload on stdin whose session_id is keel-facts-<pid> and agent_id facts/<pid>: subagent-start "
     "(exit, silence, the agent-facts_<pid>.fp file it wrote - the slash folded - and its length), a post-edit fire (the file is unchanged: never overwritten), "
     "subagent-stop over the unmoved start (exit, silence), subagent-stop after this script overwrote the fp file (exit, the systemMessage naming the control, "
     "`Not a block`, D0502), and subagent-stop under a session and agent no fire named (the not-gated advisory); ledger = the probe session's own lines in "
     ".keel/metrics/hooks.jsonl by event / control / decision; both fp files are removed after and cleanedUp says so. ledger (whole file, sessions beginning "
     "`probe` or `keel-facts-` excluded): subagent-start fires, verifier:tree-written and recorder:tree-red lines, subagent-stop block lines ever and in the "
     "UTC day 2026-09-16 by ts; agentFpFiles = agent-*.fp under .keel/metrics now. issue578 as section 37. resolverPositions as section 34; backlogItems = every "
     "`action x;` in backlog.sysml. sprint731 from its delivery file, charter d0502, plus literal `issue578`, `dispatch order` and the standup's ordering sentence.")
# ================================================================ 39. D0503 - the guard-name list is a fact of the schema member (sprint 732, brief 39)
_d0503 = _dec_file("0503-")
_f503 = _decision_facts(_d0503, "d0503")
_gn03_path = os.path.join(REPO, "members", "keel-schema", "src", "guard_names.rs")
_gn03 = read(_gn03_path) or ""
_gd03_path = _crate_root("keel-guards")  # sprint 733: the module is a crate; its root carries the lock and the two tests
_gd03 = _crate_text("keel-guards")
_gd03_rel = os.path.relpath(_gd03_path, REPO).replace(os.sep, "/") if _gd03_path else None
_cp03 = read(os.path.join(REPO, "members", "keel-view", "src", "control_proof.rs")) or ""
_vc03 = read(os.path.join(REPO, "members", "keel-view", "Cargo.toml")) or ""
_vl03 = read(os.path.join(REPO, "members", "keel-view", "src", "lib.rs")) or ""
_cl03 = read(_mh("lib", crate="keel-cli") or "") or ""
_gdoc03 = read(os.path.join(REPO, ".engine", "docs", "guards.md")) or ""
_sp03 = read(os.path.join(REPO, ".engine", "processes", "stpa-diagram.sysml")) or ""
_sk03 = read(os.path.join(REPO, ".engine", "skills", "stpa-diagram", "SKILL.md")) or ""
_sk03c = read(os.path.join(REPO, ".claude", "skills", "stpa-diagram", "SKILL.md")) or ""
_reg03 = read(os.path.join(REPO, ".tracking", "architecture", "code-registry.sysml")) or ""
_mh03 = read(os.path.join(REPO, "scripts", "module_home.py")) or ""
_gn03_decl = re.search(r"pub const GUARD_NAMES: \[&str; (\d+)\] =\s*\[([^\]]*)\]", _gn03, re.S)
_gn03_names = re.findall(r'"([^"]+)"', _gn03_decl.group(2)) if _gn03_decl else []
_gd03_decl = re.search(r"pub const GUARD_NAMES: \[&str; (\d+)\]", _gd03)
_gsf03 = (re.search(r"const GUARD_SOURCE_FILES: &\[&str\] = &\[(.*?)\];", _gd03) or [None, ""])[1]
_gsf03_paths = re.findall(r'"([^"]+)"', _gsf03)
_vdeps03 = re.findall(r"^([\w-]+)\s*=\s*\{\s*path\s*=", (re.search(r"\[dependencies\](.*?)(?:\n\[|\Z)", _vc03, re.S) or [None, ""])[1], re.M)
_view03_mods = sorted(f[:-3] for f in os.listdir(os.path.join(REPO, "members", "keel-view", "src", "view")) if f.endswith(".rs") and f != "mod.rs") \
    if os.path.isdir(os.path.join(REPO, "members", "keel-view", "src", "view")) else []
_reexp03 = [m for m in ("control_proof", "arch", "govern", "pm", "view") if re.search(r"^pub use keel_view::" + m + r";", _cl03, re.M)]
_members03 = sorted(d for d in os.listdir(os.path.join(REPO, "members")) if os.path.isdir(os.path.join(REPO, "members", d, "src")))
_view03_files = sum(len([f for f in fs if f.endswith(".rs")]) for _, _, fs in os.walk(os.path.join(REPO, "members", "keel-view", "src")))
_view03_lines = sum(len((read(os.path.join(dp, f)) or "").splitlines()) for dp, _, fs in os.walk(os.path.join(REPO, "members", "keel-view", "src")) for f in fs if f.endswith(".rs"))
# the landed commit's shape: the sprint's diff against the commit before it (renames followed, D0479's "moved, not rewritten")
_ok03d, _out03d = run(["git", "diff", "--name-status", "-M", "397e49c", "d2ee86e"])
_codes03 = [l.split("\t")[0][:1] for l in (_out03d or "").splitlines() if l.strip()]
_ok03s, _out03s = run(["git", "diff", "--shortstat", "397e49c", "d2ee86e"])
_short03 = re.search(r"(\d+) files? changed(?:, (\d+) insertions?\(\+\))?(?:, (\d+) deletions?\(-\))?", _out03s or "")
# live: what the binary states about the list, and the lock's own verdict on this tree
_rc03v, _out03v = run_rc([KEEL, "version"], timeout=60)
_guards03_line = next((l for l in (_out03v or "").splitlines() if l.strip().startswith("guards:")), "")
_guards03_n = int(re.search(r"guards:\s*(\d+)", _guards03_line).group(1)) if re.search(r"guards:\s*(\d+)", _guards03_line) else None
_rc03g, _out03g = run_rc([KEEL, "gate", "guard", "process-change", "--no-receipt", "."], timeout=300)
_pc03_last = (_out03g or "").strip().splitlines()[-1] if (_out03g or "").strip() else ""
_pc03 = re.search(r"\[guard:process-change\] (PASS|FAIL)", _pc03_last)
_i579 = _issue_facts("579", "storyViewIsAMember")
_i580 = _issue_facts("580", "dcWorkspaceLayeringIsGuarded")
_i581 = _issue_facts("581", "dcQuotedProbeLineIsRefused")
_s732 = _sprint_facts("sprint732_viewIsAMember.sysml", "d0479")
# the issues guard's rule (guards.rs, issue333/D0304): the resolver names the issue in its title, DoD text or decision, OR the issue names its resolver
_i580["resolverNamedByIssue"] = "dcWorkspaceLayeringIsGuarded" in (re.search(r"part issue580 : Issue\s*\{(.*?)\n\s*\}", _iss, re.S) or [None, ""])[1]
_i581["resolverDodNamesIssue"] = bool(re.search(r"dcQuotedProbeLineIsRefusedDoD[^\n]*resolves issue581", _bl))
_i579["resolverDodNamesIssue"] = "this story resolves issue579" in (_s732.get("text") or "")
_reg03_view = re.search(r"part ceViewCore : CodeElement \{(.*?)\n\s*\}", _reg03, re.S)
_reg03_arch = re.search(r"part ceArchViewsMember : CodeElement \{(.*?)\n\s*\}", _reg03, re.S)
fact("guardNameListIsASchemaFact", {
    **_f503,
    "dependsOnD0479": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0503\s+to\s+d0479\s*;", _d0503)),
    "namesThirdExtraction": "the third D0479 extraction" in (_f503["context"] or ""),
    "namesTheOneReference": "control_proof::census read GUARD_NAMES" in (_f503["context"] or ""),
    "namesLockCannotTell": "the lock cannot tell a declaration move from a disarm" in (_f503["context"] or ""),
    "namesD0486Precedent": "sprint 718 met the same lock and recorded D0486" in (_f503["context"] or ""),
    "namesNewHome": "members/keel-schema/src/guard_names.rs" in (_f503["decision"] or ""),
    "namesReExport": "re-exported by keel-cli/src/guards.rs under its old path" in (_f503["decision"] or ""),
    "namesJoinsLock": "guard_names.rs joins GUARD_SOURCE_FILES" in (_f503["decision"] or ""),
    "namesRendererHome": "members/keel-view/src/view/stpa_diagram.rs" in (_f503["decision"] or ""),
    "namesNoGuardChanges": "No guard's name, dispatch, severity or gate set changes" in (_f503["decision"] or ""),
    "namesOneHome": "One home per fact (D0105)" in (_f503["rationale"] or ""),
    "namesDisarmClass": "issue236's class" in (_f503["rationale"] or ""),
    "namesTwoLockedFiles": "Adding a guard edits two locked files instead of one" in (_f503["consequences"] or ""),
    "namesThirdMeeting": "dcGuardsAreMembersPerFamily meets the lock a third time" in (_f503["consequences"] or ""),
    "namesReversesNothing": "reverses nothing" in (_f503["consequences"] or ""),
    "source": {
        "listDeclaredInSchema": bool(_gn03_decl),
        "declaredCount": int(_gn03_decl.group(1)) if _gn03_decl else None,
        "listedNames": len(_gn03_names),
        "countMatchesList": bool(_gn03_decl) and int(_gn03_decl.group(1)) == len(_gn03_names),
        "listNoLongerDeclaredInGuards": not _gd03_decl,
        "guardsReExports": bool(re.search(r"^pub use keel_schema::guard_names::GUARD_NAMES;", _gd03, re.M)),
        "guardNamesOnTheLock": "members/keel-schema/src/guard_names.rs" in _gsf03_paths,
        "lockPaths": _gsf03_paths,
        "lockCommentNamesSprint": "guard_names.rs holds GUARD_NAMES (sprint 732)" in _gd03,
        "dispatchTest": "fn every_enforced_guard_dispatches()" in _gd03,
        "surfaceTest": "fn enforcement_surface_covers_every_guard_source()" in _gd03,
        "censusReadsSchema": "census_over(root, &keel_schema::guard_names::GUARD_NAMES)" in _cp03,
        "viewDependsOn": _vdeps03,
        "viewReadsNoCrateAbove": bool(_vdeps03) and not any(d in ("keel-cli",) for d in _vdeps03),
        "viewSubmodules": _view03_mods,
        "viewSubmoduleCount": len(_view03_mods),
        "viewTopModules": [m for m in ("arch", "control_proof", "govern", "pm", "view") if re.search(r"^pub mod " + m + r";", _vl03, re.M)],
        "cliReExports": _reexp03,
        "viewRsFiles": _view03_files, "viewRsLines": _view03_lines,
        "members": _members03, "memberCount": len(_members03),
        "moduleHomeKnowsGuardNames": '("guard_names", os.path.join("members", "keel-schema", "src", "guard_names.rs"))' in _mh03,
    },
    "landed": {
        "renames": _codes03.count("R"), "modified": _codes03.count("M"), "added": _codes03.count("A"), "deleted": _codes03.count("D"),
        "filesChanged": int(_short03.group(1)) if _short03 else None,
        "insertions": int(_short03.group(2)) if _short03 and _short03.group(2) else None,
        "deletions": int(_short03.group(3)) if _short03 and _short03.group(3) else None,
    } if _ok03d and _ok03s else None,
    "declared": {
        "guardsDocNamesTheFile": "members/keel-schema/src/guard_names.rs" in _gdoc03 and "since D0503" in _gdoc03,
        "processNamesRenderer": "members/keel-view/src/view/stpa_diagram.rs" in _sp03,
        "processNamesOldPath": "keel-cli/src/view/stpa_diagram.rs" in _sp03,
        "skillNamesRenderer": "members/keel-view/src/view/stpa_diagram.rs" in _sk03,
        "claudeCopyAgrees": _sk03 == _sk03c and bool(_sk03),
        "registryViewCorePath": (re.search(r'filePath\s*=\s*"([^"]+)"', _reg03_view.group(1)) or [None, None])[1] if _reg03_view else None,
        "registryArchMemberPath": (re.search(r'filePath\s*=\s*"([^"]+)"', _reg03_arch.group(1)) or [None, None])[1] if _reg03_arch else None,
        "registryArchSupersedes": "#Supersede dependency from ceArchViewsMember to ceArchViews;" in _reg03,
    },
    "live": {
        "version": {"exit0": _rc03v == 0, "guardsLine": _guards03_line.strip() or None, "guards": _guards03_n,
                    "matchesTheList": _guards03_n is not None and _guards03_n == len(_gn03_names)},
        "processChange": {"exit0": _rc03g == 0, "verdict": _pc03.group(1) if _pc03 else None, "line": _pc03_last[:240] or None},
    },
    "issue579": _i579, "issue580": _i580, "issue581": _i581,
    "resolverPositions": {a: ({"place": _bl00_actions.index(a) + 1, "def": _bl00_defs.get(a)} if a in _bl00_actions else None)
                          for a in ("dcSuiteIsAMember", "dcGuardsAreMembersPerFamily", "dcWorkspaceLayeringIsGuarded", "dcQuotedProbeLineIsRefused", "dcOneRepoRootHelper")},
    "backlogItems": len(_bl00_actions),
    "sprint732": {k: v for k, v in _s732.items() if k != "text"} | ({
        "retroNamesIssue579": "issue579" in _s732["text"] or "an Issue with the widened regex" in _s732["text"],
        "retroNamesIssue581": "issue581" in _s732["text"],
        "retroNamesModgraphVacuous": "modgraph.py --check" in _s732["text"] and "cannot fail on what it names" in _s732["text"],
        "retroNamesAnchorControlFired": "fired for the first time on a real move" in _s732["text"],
        "retroNamesD0486Pattern": "as sprint 718 needed D0486" in _s732["text"],
        "retroFindings": len(re.findall(r"(?:^|[.;] )\((\d)\) ", (re.search(r'viewIsAMemberRetroGate[^\n]*procedureText = "([^"]*)"', _s732["text"]) or [None, ""])[1])),
        "dodSaysHelpByteIdentical": "keel --help is byte-identical to the pre-move render" in _s732["text"],
        "dodSaysGuardsUnchanged": "keel version's guards line is unchanged at 75" in _s732["text"],
    } if _s732.get("exists") else {}),
} if _d0503 and _gn03 and _gd03 else None,
     "the held record for the guard-name list's move: Decision, the list and the lock in source, the member's read set, the landed diff, the declaration homes, live version and lock verdict, issues 579-581, sprint 732",
     _DEC_HOW + " Names by literal search in the field named; dependsOnD0479 = the `#DependsOn dependency from d0503 to d0479;` line. source: "
     "members/keel-schema/src/guard_names.rs searched for `pub const GUARD_NAMES: [&str; N] = [...]` (N and the quoted members counted apart); the guards "
     "module (the keel-guards crate's every file since sprint 733) searched for the same declaration (absent = moved), the `pub use keel_schema::guard_names::GUARD_NAMES;` "
     "re-export, GUARD_SOURCE_FILES' quoted paths, the sprint-732 comment above it and the two test fn names; members/keel-view/src/control_proof.rs for the "
     "census call reading keel_schema; keel-view's Cargo.toml [dependencies] `x = { path = ...}` names; the .rs files under members/keel-view/src/view "
     "other than mod.rs; keel-view's lib.rs `pub mod x;` lines; keel-cli's lib.rs `pub use keel_view::x;` lines; .rs files and their line counts under "
     "members/keel-view/src; members/*/src directories; module_home.py's guard_names entry. landed = `git diff --name-status -M 397e49c d2ee86e` first "
     "letters counted and `git diff --shortstat` over the same range. declared: .engine/docs/guards.md, .engine/processes/stpa-diagram.sysml, the "
     "stpa-diagram skill and its .claude copy (byte equality), code-registry.sysml's ceViewCore and ceArchViewsMember filePath and the #Supersede edge. "
     "live: `" + KEEL + " version`'s `guards:` line against the list's length; `" + KEEL + " gate guard process-change --no-receipt .` last line. "
     "issues 579-581 as section 34, with the issues guard's naming rule (resolver names the issue in its DoD text, or the issue names its resolver) read "
     "from the sprint file (579), backlog.sysml (581) and the issue's own description (580). resolverPositions as section 34; sprint732 from its delivery "
     "file, charter d0479, plus literal spans in the retro and DoD; retroFindings = `(n) ` markers opening a sentence in the retro gate's procedureText (a back-reference like `finding (1)` is not one).")
# ================================================================ 40. D0504 - the guard-source lock is a directory prefix (sprint 733, brief 40)
_d0504 = _dec_file("0504-")
_f504 = _decision_facts(_d0504, "d0504")
_LAND733_FROM, _LAND733_TO = "f8d3d8d", "c3ffd45"   # the sprint's landing range, fixed (issue586: a fact about a range reads the range)
_enf04 = read(os.path.join(REPO, "members", "keel-guards", "src", "enforcement.rs")) or ""
_lib04 = read(os.path.join(REPO, "members", "keel-guards", "src", "lib.rs")) or ""
def _tb04(text, name):
    """A test fn's body inside `mod tests`: from `fn name(` to the next line that is exactly four spaces and `}` (_fn_body stops at column 0, which is the module's end)."""
    s = text.find("fn " + name + "(")
    if s < 0:
        return ""
    e = text.find("\n    }\n", s)
    return text[s:] if e < 0 else text[s:e]
_gsf04 = (re.search(r"const GUARD_SOURCE_FILES: &\[&str\] = &\[(.*?)\];", _enf04) or [None, ""])[1]
_gsf04_paths = re.findall(r'"([^"]+)"', _gsf04)
_gsd04 = (re.search(r"const GUARD_SOURCE_DIRS: &\[&str\] = &\[(.*?)\];", _enf04) or [None, ""])[1]
_gsd04_dirs = re.findall(r'"([^"]+)"', _gsd04)
_ies04 = _fn_body(_enf04, "is_enforcement_surface")
_cov04 = _tb04(_lib04, "enforcement_surface_covers_every_guard_source")
_lock04 = _tb04(_lib04, "enforcement_surface_locks_workflows_hooks_and_guard_source")
_disp04 = _tb04(_lib04, "every_enforced_guard_dispatches")
_fam04 = re.search(r"pub const FAMILIES: &\[&Family\] = &\[(.*?)\];", _lib04)
_fam04_names = re.findall(r"&(\w+)::FAMILY", _fam04.group(1)) if _fam04 else []
_gd04_dir = os.path.join(REPO, "members", "keel-guards", "src")
_gd04_files = sorted(f for f in os.listdir(_gd04_dir) if f.endswith(".rs")) if os.path.isdir(_gd04_dir) else []
_gd04_lines = sum(len((read(os.path.join(_gd04_dir, f)) or "").splitlines()) for f in _gd04_files)
_gd04_defining = [f for f in _gd04_files if "-> GuardReport" in (read(os.path.join(_gd04_dir, f)) or "")]
_ws04 = read(os.path.join(REPO, "Cargo.toml")) or ""
_ws04_members = re.findall(r'^\s*"([^"]+)",', (re.search(r"members\s*=\s*\[(.*?)\]", _ws04, re.S) or [None, ""])[1], re.M)
# every guard-defining .rs in the workspace, and whether the lock (as its two constants read) holds it - the coverage test's own question, asked from outside
_def04 = []
for _mem in _ws04_members:
    for _dp, _, _fs in os.walk(os.path.join(REPO, _mem, "src")):
        for _f in _fs:
            if _f.endswith(".rs") and "-> GuardReport" in (read(os.path.join(_dp, _f)) or ""):
                _def04.append(os.path.relpath(os.path.join(_dp, _f), REPO).replace(os.sep, "/"))
_def04_uncovered = [p for p in _def04 if p not in _gsf04_paths and not any(p.startswith(d) for d in _gsd04_dirs)]
_old04 = _module_home("guards") is not None   # resolved, never anchored (issue559): the module has no single home once it is a member
_ok04o, _out04o = run(["git", "show", _LAND733_FROM + ":keel-cli/src/guards.rs"])
_old04_lines = len(_out04o.splitlines()) if _ok04o else None
_gdoc04 = read(os.path.join(REPO, ".engine", "docs", "guards.md")) or ""
_gdoc04_h2 = re.findall(r"^## (.+)$", _gdoc04, re.M)
_mh04 = read(os.path.join(REPO, "scripts", "module_home.py")) or ""
_split04 = read(os.path.join(REPO, "scripts", "split_guards.py")) or ""
# the landed commit's shape over the fixed range (renames followed)
_ok04d, _out04d = run(["git", "diff", "--name-status", "-M", _LAND733_FROM, _LAND733_TO])
_codes04 = [l.split("\t")[0][:1] for l in (_out04d or "").splitlines() if l.strip()]
_ok04s, _out04s = run(["git", "diff", "--shortstat", _LAND733_FROM, _LAND733_TO])
_short04 = re.search(r"(\d+) files? changed(?:, (\d+) insertions?\(\+\))?(?:, (\d+) deletions?\(-\))?", _out04s or "")
# live: the lock's own verdict on this tree, the binary's guard count, the landing run's receipt
_rc04g, _out04g = run_rc([KEEL, "gate", "guard", "process-change", "--no-receipt", "."], timeout=300)
_pc04_last = (_out04g or "").strip().splitlines()[-1] if (_out04g or "").strip() else ""
_pc04 = re.search(r"\[guard:process-change\] (PASS|FAIL|WARN)[^\d]*(\d+) scanned[^\d]*(\d+) warning\(s\), (\d+) violation\(s\)", _pc04_last)
_rc04v, _out04v = run_rc([KEEL, "version"], timeout=60)
_guards04_line = next((l for l in (_out04v or "").splitlines() if l.strip().startswith("guards:")), "")
_guards04_n = int(re.search(r"guards:\s*(\d+)", _guards04_line).group(1)) if re.search(r"guards:\s*(\d+)", _guards04_line) else None
_tr04 = _receipt94("touched-receipt.toml")
_i583 = _issue_facts("583", "dcOneRepoRootHelper")
_i584 = _issue_facts("584", "dcGuardsCatalogueNamesTheFamily")
_i585 = _issue_facts("585", "dcBuildScriptHasOneHomeBelowItsUsers")
_i586 = _issue_facts("586", "dcFactsAboutARangeReadTheRange")
_i587 = _issue_facts("587", "dcGuardsAreMembersPerFamily")
# the issues guard's rule (issue333/D0304): the resolver names the issue in its DoD, OR the issue names its resolver
for _n04, _i04 in (("583", _i583), ("587", _i587)):
    _i04["resolverNamedByIssue"] = _i04["resolver"] is not None and _i04["resolver"] in (re.search(r"part issue" + _n04 + r" : Issue\s*\{(.*?)\n\s*\}", _iss, re.S) or [None, ""])[1]
_s733 = _sprint_facts("sprint733_guardsAreMembersPerFamily.sysml", "d0479")
_retro04 = (re.search(r'guardsAreMembersPerFamilyRetroGate[^\n]*procedureText = "([^"]*)"', _s733.get("text") or "") or [None, ""])[1]
fact("guardSourceLockIsADirectoryPrefix", {
    **_f504,
    "dependsOnD0479": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0504\s+to\s+d0479\s*;", _d0504)),
    "dependsOnD0503": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0504\s+to\s+d0503\s*;", _d0504)),
    "namesFourthExtraction": "the fourth D0479 extraction" in (_f504["context"] or ""),
    "namesOldSize": "9,256 lines" in (_f504["context"] or ""),
    "namesSilentScan": "a scan aimed at keel-cli/src would pass with nothing to check" in (_f504["context"] or ""),
    "namesD0388Class": "the class D0388 names" in (_f504["context"] or ""),
    "namesThirdMeeting": "the third meeting of the lock" in (_f504["context"] or ""),
    "namesFileListLoses": "GUARD_SOURCE_FILES loses keel-cli/src/guards.rs" in (_f504["decision"] or ""),
    "namesDirsConstant": "GUARD_SOURCE_DIRS, names members/keel-guards/src/" in (_f504["decision"] or ""),
    "namesPrefixRule": "starts with a locked directory" in (_f504["decision"] or ""),
    "namesScansEveryMember": "scans every member's src" in (_f504["decision"] or ""),
    "namesReachedOwnFiles": "asserts the scan reached the guards member's own files" in (_f504["decision"] or ""),
    "namesDispatchReadsTables": "every_enforced_guard_dispatches reads the family tables" in (_f504["decision"] or ""),
    "namesUnionTest": "family_union_tests holds the tables equal to GUARD_NAMES" in (_f504["decision"] or ""),
    "namesWidening": "which is a widening" in (_f504["decision"] or ""),
    "namesLockedByConstruction": "locked by construction" in (_f504["rationale"] or ""),
    "namesCheckFollowsSubject": "the check follows the subject" in (_f504["rationale"] or ""),
    "namesNextExtractionCovered": "the next extraction (dcSuiteIsAMember) is covered without editing this test" in (_f504["rationale"] or ""),
    "namesEveryFileLocked": "Every file under members/keel-guards/src/ is a locked surface" in (_f504["consequences"] or ""),
    "namesCargoNotLocked": "The Cargo.toml of the member is not locked" in (_f504["consequences"] or ""),
    "namesTwelveMembers": "at least twelve members" in (_f504["consequences"] or ""),
    "namesRecordedThreeTimes": "recorded three times (D0486, D0503, this)" in (_f504["consequences"] or ""),
    "source": {
        "lockFiles": _gsf04_paths, "lockFileCount": len(_gsf04_paths),
        "oldPathOffTheList": "keel-cli/src/guards.rs" not in _gsf04_paths,
        "adherenceOnTheList": "keel-cli/src/adherence.rs" in _gsf04_paths,
        "guardNamesOnTheList": "members/keel-schema/src/guard_names.rs" in _gsf04_paths,
        "lockDirs": _gsd04_dirs, "guardsDirIsTheLock": _gsd04_dirs == ["members/keel-guards/src/"],
        "prefixRuleInSource": "GUARD_SOURCE_DIRS.iter().any(|d| p.starts_with(d))" in _ies04,
        "coverageTestFound": bool(_cov04),
        "coverageReadsTheManifest": "workspace_members(&manifest)" in _cov04,
        "coverageScansEveryMember": "for member in &members" in _cov04 and 'join(member).join("src")' in _cov04,
        "coverageAssertsTwelve": "members.len() >= 12" in _cov04,
        "coverageAssertsReachedOwn": 'ends_with("identity.rs")' in _cov04,
        "coverageNamesBothConstants": "GUARD_SOURCE_FILES / GUARD_SOURCE_DIRS" in _cov04,
        "lockTestFound": bool(_lock04),
        "lockTestLocks": [p for p in ("members/keel-guards/src/lib.rs", "members/keel-guards/src/identity.rs", "members/keel-guards/src/receipt.rs",
                                     "keel-cli/src/adherence.rs", "members/keel-schema/src/guard_names.rs") if f'assert!(is_enforcement_surface("{p}"))' in _lock04],
        "lockTestFrees": [p for p in ("keel-cli/src/main.rs", "keel-cli/src/guards.rs", "members/keel-guards/Cargo.toml", ".engine/docs/guards.md")
                          if f'assert!(!is_enforcement_surface("{p}"))' in _lock04],
        "dispatchReadsTables": "super::FAMILIES.iter().flat_map(|f| f.arms.iter())" in _disp04,
        "dispatchScansNoText": "read_to_string" not in _disp04,
        "unionTestModule": bool(re.search(r"^mod family_union_tests \{", _lib04, re.M)),
        "unionTestUsesRunnableOnly": "use super::{FAMILIES, GUARD_NAMES, RUNNABLE_ONLY};" in _lib04,
        "families": _fam04_names, "familyCount": len(_fam04_names),
        "memberFiles": _gd04_files, "memberFileCount": len(_gd04_files), "memberLines": _gd04_lines,
        "guardDefiningFilesInMember": _gd04_defining, "guardDefiningFilesInWorkspace": _def04,
        "uncoveredGuardDefiningFiles": _def04_uncovered,
        "workspaceMembers": _ws04_members, "workspaceMemberCount": len(_ws04_members),
        "oldFileGone": not _old04, "oldFileLinesAtBase": _old04_lines,
        "moduleHomeHasCrateHelpers": "def crate_root(" in _mh04 and "def crate_text(" in _mh04 and "def crate_sources(" in _mh04,
        "splitScriptDeclaresItself": "not-an-instrument:" in _split04 and "one-shot codemod" in _split04,
        "splitScriptRecordsWrongFix": "WRONG FIX, kept as the record of what ran" in _split04,
    },
    "landed": {
        "range": [_LAND733_FROM, _LAND733_TO],
        "renames": _codes04.count("R"), "modified": _codes04.count("M"), "added": _codes04.count("A"), "deleted": _codes04.count("D"),
        "filesChanged": int(_short04.group(1)) if _short04 else None,
        "insertions": int(_short04.group(2)) if _short04 and _short04.group(2) else None,
        "deletions": int(_short04.group(3)) if _short04 and _short04.group(3) else None,
    } if _ok04d and _ok04s else None,
    "declared": {
        "guardsDocHeadings": _gdoc04_h2, "guardsDocGroupsByTier": _gdoc04_h2[:2] == ["Hard-blocking", "Warning-only"],
        "guardsDocNamesTheMember": "members/keel-guards/src/" in _gdoc04,
        "guardsDocNamesOldPath": "keel-cli/src/guards.rs" in _gdoc04,
    },
    "live": {
        "processChange": {"exit0": _rc04g == 0, "verdict": _pc04.group(1) if _pc04 else None, "scanned": int(_pc04.group(2)) if _pc04 else None,
                          "violations": int(_pc04.group(4)) if _pc04 else None, "line": _pc04_last[:240] or None},
        "version": {"exit0": _rc04v == 0, "guardsLine": _guards04_line.strip() or None, "guards": _guards04_n},
        "landingReceipt": _tr04,
    },
    "issue583": _i583, "issue584": _i584, "issue585": _i585, "issue586": _i586, "issue587": _i587,
    "resolverPositions": {a: ({"place": _bl00_actions.index(a) + 1, "def": _bl00_defs.get(a)} if a in _bl00_actions else None)
                          for a in ("dcGuardsAreMembersPerFamily", "dcOneRepoRootHelper", "dcSuiteIsAMember", "dcGuardsCatalogueNamesTheFamily",
                                    "dcBuildScriptHasOneHomeBelowItsUsers", "dcFactsAboutARangeReadTheRange")},
    "backlogItems": len(_bl00_actions),
    "sprint733": {k: v for k, v in _s733.items() if k != "text"} | ({
        "retroScansAvoidable": "Avoidable issues scanned (issue011)" in _retro04,
        "retroNamesIssues": [n for n in ("583", "584", "585", "586", "587") if "issue" + n in _retro04],
        "retroNamesAnchorControlFired": "no_member_test_anchors_on_a_cwd_relative_path refused all ten" in _retro04,
        "retroNamesRootHelperMoved": "MOVED to the head of the D0479 cluster" in _retro04,
        "retroNamesProbeRunnerFired": "script-probe runner (D0496) aborted the commit as designed" in _retro04,
        "retroFindings": len(re.findall(r"(?:^|[.;:] )\((\d)\) ", _retro04)),
        "dodResults": _dod_results("dcGuardsAreMembersPerFamily"),
    } if _s733.get("exists") else {}),
} if _d0504 and _enf04 and _lib04 else None,
     "the held record for the guard-source lock's new shape: Decision, the two lock constants and the prefix rule in source, the coverage and lock tests, the family tables, the landed diff, the catalogue's grouping, live lock verdict and guard count, issues 583-587, sprint 733",
     _DEC_HOW + " Names by literal search in the field named; dependsOn = the `#DependsOn dependency from d0504 to dNNNN;` lines. source: "
     "members/keel-guards/src/enforcement.rs searched for GUARD_SOURCE_FILES' and GUARD_SOURCE_DIRS' quoted members and is_enforcement_surface's body; "
     "members/keel-guards/src/lib.rs for the bodies of enforcement_surface_covers_every_guard_source (manifest read, member loop, >= 12, identity.rs reach), "
     "enforcement_surface_locks_workflows_hooks_and_guard_source (each asserted path, locked and free), every_enforced_guard_dispatches (FAMILIES iteration, no "
     "read_to_string), the `mod family_union_tests` line and its use line, and `pub const FAMILIES` members; the .rs files under members/keel-guards/src, their "
     "line count and which contain `-> GuardReport`; the root Cargo.toml `members = [...]` list; every member's src walked for `-> GuardReport` files and each held "
     "against the two constants as read (uncoveredGuardDefiningFiles); the guards module resolved through module_home (None once it is a member - the old file is gone) and the old file's line count at the range's base via `git show`; "
     "module_home.py's three crate helpers; split_guards.py's not-an-instrument line and WRONG FIX comment. landed = `git diff --name-status -M " + _LAND733_FROM + " " + _LAND733_TO +
     "` first letters counted and `git diff --shortstat` over the same range. declared: .engine/docs/guards.md `## ` headings (tier grouping = the first two), the member path, the old path. "
     "live: `" + KEEL + " gate guard process-change --no-receipt .` last line; `" + KEEL + " version`'s `guards:` line; the touched receipt as section 32 reads it. "
     "issues 583-587 as section 34, with the issues guard's naming rule read from the backlog DoD (584-586) or the issue's own description (583, 587). "
     "resolverPositions as section 34; sprint733 from its delivery file, charter d0479, plus literal spans in the retro gate's procedureText; retroFindings = "
     "`(n) ` markers opening a sentence or following a label's colon in that text; dodResults = the story's DoDRn outcomes and shas.")
# ================================================================ 41. D0505 - a locked guard source's test anchor repoints through keel-fs (sprint 734, brief 41)
_d0505 = _dec_file("0505-")
_f505 = _decision_facts(_d0505, "d0505")
_LAND734_FROM, _LAND734_TO = "e3714e3", "1736ff4"   # the sprint's landing range, fixed (issue586: a fact about a range reads the range)
_ts05_path = os.path.join(REPO, "members", "keel-fs", "src", "test_support.rs")
_ts05 = read(_ts05_path) or ""
_fslib05 = read(os.path.join(REPO, "members", "keel-fs", "src", "lib.rs")) or ""
_adh05 = read(_mh("adherence") or "") or ""   # issue559: the resolver, never a keel-cli/src anchor
_adh05_test = _tb04(_adh05, "this_repo_yields_the_empty_prefix")
_tch05 = read(_mh("touched") or "") or ""
_scan05 = _tb04(_tch05, "no_member_test_anchors_on_a_cwd_relative_path")
_enf05 = read(os.path.join(REPO, "members", "keel-guards", "src", "enforcement.rs")) or ""
_gsf05_paths = re.findall(r'"([^"]+)"', (re.search(r"const GUARD_SOURCE_FILES: &\[&str\] = &\[(.*?)\];", _enf05) or [None, ""])[1])
# the four anchor shapes the scan reads (keel-cli/src/touched.rs ANCHORS), counted over non-comment lines per tree
_ANCH05 = ('Path::new("..")', 'read_to_string("src/', 'CARGO_MANIFEST_DIR")).join("..")', 'read_to_string("../')
def _rs05(rel_dirs):
    out = []
    for _rd in rel_dirs:
        for _dp, _, _fs in os.walk(os.path.join(REPO, _rd)):
            for _f in _fs:
                if _f.endswith(".rs"):
                    out.append(os.path.join(_dp, _f))
    return sorted(out)
def _anchors05(paths):
    hits = []
    for _p in paths:
        for _i, _l in enumerate((read(_p) or "").splitlines(), 1):
            if _l.strip().startswith("//"):
                continue
            for _a in _ANCH05:
                if _a in _l:
                    hits.append(os.path.relpath(_p, REPO).replace(os.sep, "/") + ":" + str(_i))
    return hits
_ws05_members = re.findall(r'^\s*"([^"]+)",', (re.search(r"members\s*=\s*\[(.*?)\]", read(os.path.join(REPO, "Cargo.toml")) or "", re.S) or [None, ""])[1], re.M)
_member_src05 = [m + "/src" for m in _ws05_members if m.startswith("members/")]
_cli05, _mem05, _tests05 = _rs05(["keel-cli/src"]), _rs05(_member_src05), _rs05(["keel-cli/tests"])
_defs05 = []
for _p in _cli05 + _mem05:
    for _i, _l in enumerate((read(_p) or "").splitlines(), 1):
        if _l.lstrip().startswith("pub fn repo_root(") or _l.lstrip().startswith("fn repo_root("):
            _defs05.append(os.path.relpath(_p, REPO).replace(os.sep, "/") + ":" + str(_i))
_uses05 = [os.path.relpath(_p, REPO).replace(os.sep, "/") for _p in _cli05 + _mem05 if "use keel_fs::test_support::repo_root;" in (read(_p) or "")]
_calls05 = [os.path.relpath(_p, REPO).replace(os.sep, "/") for _p in _cli05 if "keel_fs::test_support::repo_root()" in (read(_p) or "")]
def _devdep05(member):
    _c = read(os.path.join(REPO, "members", member, "Cargo.toml")) or ""
    _dd = _c.split("[dev-dependencies]", 1)[1] if "[dev-dependencies]" in _c else ""
    return bool(re.search(r"^keel-fs\s*=", _dd.split("\n[", 1)[0], re.M))
_collapse05 = read(os.path.join(REPO, "scripts", "collapse_repo_root.py")) or ""
# the landed commit's shape over the fixed range, and the locked file's own numstat inside it
_ok05d, _out05d = run(["git", "diff", "--name-status", "-M", _LAND734_FROM, _LAND734_TO])
_codes05 = [l.split("\t")[0][:1] for l in (_out05d or "").splitlines() if l.strip()]
_names05 = [l.split("\t")[-1] for l in (_out05d or "").splitlines() if l.strip()]
_ok05s, _out05s = run(["git", "diff", "--shortstat", _LAND734_FROM, _LAND734_TO])
_short05 = re.search(r"(\d+) files? changed(?:, (\d+) insertions?\(\+\))?(?:, (\d+) deletions?\(-\))?", _out05s or "")
_ok05a, _out05a = run(["git", "diff", "--numstat", _LAND734_FROM, _LAND734_TO, "--", "keel-cli/src/adherence.rs"])
_adh05_num = re.match(r"(\d+)\s+(\d+)", (_out05a or "").strip())
_ok05p, _out05p = run(["git", "diff", "-U0", _LAND734_FROM, _LAND734_TO, "--", "keel-cli/src/adherence.rs"])
_adh05_added = [l[1:] for l in (_out05p or "").splitlines() if l.startswith("+") and not l.startswith("+++")]
_adh05_removed = [l[1:] for l in (_out05p or "").splitlines() if l.startswith("-") and not l.startswith("---")]
_locked05_touched = [n for n in _names05 if n in _gsf05_paths or n.startswith("members/keel-guards/src/")]
# live: the lock's own verdict on this tree, the landing run's receipt
_rc05g, _out05g = run_rc([KEEL, "gate", "guard", "process-change", "--no-receipt", "."], timeout=300)
_pc05_last = (_out05g or "").strip().splitlines()[-1] if (_out05g or "").strip() else ""
_pc05 = re.search(r"\[guard:process-change\] (PASS|FAIL|WARN)[^\d]*(\d+) scanned[^\d]*(\d+) warning\(s\), (\d+) violation\(s\)", _pc05_last)
_tr05 = _receipt94("touched-receipt.toml")
_i557 = _issue_facts("557", "dcOneRepoRootHelper")
_s734 = _sprint_facts("sprint734_oneRepoRootHelper.sysml", "d0479")
_retro05 = (re.search(r'oneRepoRootHelperRetroGate[^\n]*procedureText = "([^"]*)"', _s734.get("text") or "") or [None, ""])[1]
fact("lockedTestAnchorRepointsThroughKeelFs", {
    **_f505,
    "dependsOnD0479": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0505\s+to\s+d0479\s*;", _d0505)),
    "dependsOnD0504": bool(re.search(r"#DependsOn\s+dependency\s+from\s+d0505\s+to\s+d0504\s*;", _d0505)),
    "namesBreaksTwoLevelsDown": "the shape that breaks when a crate moves two levels down" in (_f505["context"] or ""),
    "namesSecondDoorClosed": "carries no marker, so the guard's second door is closed" in (_f505["context"] or ""),
    "namesOneLineOfTestCode": "The only edit inside keel-cli/src/adherence.rs is one line of test code" in (_f505["decision"] or ""),
    "namesLogicUnchanged": "every guard's dispatch, severity and gate set are unchanged" in (_f505["decision"] or ""),
    "namesSevenOtherAnchors": "The same repoint lands in the seven other keel-cli/src anchors" in (_f505["decision"] or ""),
    "namesScanWidens": "widens its population from members/*/src to keel-cli/src as well" in (_f505["decision"] or ""),
    "namesLockOnTheFile": "The lock is on the file, not on the logic, by design" in (_f505["rationale"] or ""),
    "namesNoException": "the scan would have to carve an exception for a locked file" in (_f505["rationale"] or ""),
    "namesD0388Class": "the class of silent hole D0388 names" in (_f505["rationale"] or ""),
    "namesFirstTestOnlyFire": "the first time since D0209 that a marked Decision governs no change to enforcement logic" in (_f505["consequences"] or ""),
    "namesWorkingAsSpecified": "This is the lock working as specified, not a defect" in (_f505["consequences"] or ""),
    "namesCfgTestForkIsSeparate": "whether the lock should exempt lines inside a cfg(test) module; that is a separate process-change Decision" in (_f505["consequences"] or ""),
    "source": {
        "helperFileExists": bool(_ts05),
        "helperIsPub": "pub fn repo_root() -> std::path::PathBuf" in _ts05,
        "helperDocHidden": "#[doc(hidden)]" in _ts05,
        "helperWalksToGit": 'a.join(".git").exists()' in _ts05,
        "helperHasOwnTest": "fn repo_root_holds_the_workspace_manifest_and_the_engine" in _ts05,
        "libExportsTestSupport": bool(re.search(r"^pub mod test_support;", _fslib05, re.M)),
        "repoRootDefinitions": _defs05, "repoRootDefinitionCount": len(_defs05),
        "oneDefinitionInKeelFs": _defs05 == [d for d in _defs05 if d.startswith("members/keel-fs/src/test_support.rs:")] and len(_defs05) == 1,
        "useLineFiles": _uses05, "useLineCount": len(_uses05),
        "cliCallerFiles": _calls05, "cliCallerCount": len(_calls05),
        "anchorsInCliSrc": _anchors05(_cli05), "anchorsInMemberSrc": _anchors05(_mem05), "anchorsInCliTests": _anchors05(_tests05),
        "adherenceOnTheLockList": "keel-cli/src/adherence.rs" in _gsf05_paths,
        "adherenceTestFound": bool(_adh05_test),
        "adherenceTestCallsHelper": "let root = keel_fs::test_support::repo_root();" in _adh05_test,
        "adherenceTestHoldsEmptyPrefix": "vec![String::new()]" in _adh05_test,
        "scanTestFound": bool(_scan05),
        "scanReadsTestBearingSources": "test_bearing_sources(&root)" in _scan05,
        "scanExcludesAlwaysTest": "!always_test" in _scan05,
        "scanAssertsCliInPopulation": 'ends_with("keel-cli/src/touched.rs")' in _scan05,
        "scanAssertsPopulationSize": "files.len() > 40" in _scan05,
        "scanNamesHelperInMessage": "use keel_fs::test_support::repo_root() instead" in _scan05,
        "devDepKeelFs": {"keel-github": _devdep05("keel-github"), "keel-schema": _devdep05("keel-schema"), "keel-actor": _devdep05("keel-actor")},
        "collapseScriptDeclaresItself": "not-an-instrument:" in _collapse05 and "one-shot codemod" in _collapse05,
        "collapseScriptIdempotent": "Idempotent" in _collapse05,
        "workspaceMemberCount": len(_ws05_members),
    },
    "landed": {
        "range": [_LAND734_FROM, _LAND734_TO],
        "renames": _codes05.count("R"), "modified": _codes05.count("M"), "added": _codes05.count("A"), "deleted": _codes05.count("D"),
        "filesChanged": int(_short05.group(1)) if _short05 else None,
        "insertions": int(_short05.group(2)) if _short05 and _short05.group(2) else None,
        "deletions": int(_short05.group(3)) if _short05 and _short05.group(3) else None,
        "adherenceInsertions": int(_adh05_num.group(1)) if _adh05_num else None,
        "adherenceDeletions": int(_adh05_num.group(2)) if _adh05_num else None,
        "adherenceAddedLines": _adh05_added, "adherenceRemovedLines": _adh05_removed,
        "adherenceDiffIsTheRepoint": (len(_adh05_added) == 1 and "keel_fs::test_support::repo_root()" in _adh05_added[0]
                                      and len(_adh05_removed) == 1 and 'CARGO_MANIFEST_DIR")).join("..")' in _adh05_removed[0]),
        "lockedFilesTouched": _locked05_touched,
        "newDecisionInRange": any(n.startswith(".engine/decisions/0505-") for n in _names05),
        "sprintInRange": any(n.endswith("sprint734_oneRepoRootHelper.sysml") for n in _names05),
    } if _ok05d and _ok05s and _ok05a and _ok05p else None,
    "live": {
        "processChange": {"exit0": _rc05g == 0, "verdict": _pc05.group(1) if _pc05 else None, "scanned": int(_pc05.group(2)) if _pc05 else None,
                          "violations": int(_pc05.group(4)) if _pc05 else None, "line": _pc05_last[:240] or None},
        "landingReceipt": _tr05,
    },
    "issue557": _i557,
    "resolverPositions": {a: ({"place": _bl00_actions.index(a) + 1, "def": _bl00_defs.get(a)} if a in _bl00_actions else None)
                          for a in ("dcOneRepoRootHelper", "dcSuiteIsAMember", "dcProcessIsAMember", "dcGuardsCatalogueNamesTheFamily",
                                    "dcBuildScriptHasOneHomeBelowItsUsers", "dcFactsAboutARangeReadTheRange")},
    "backlogItems": len(_bl00_actions),
    "sprint734": {k: v for k, v in _s734.items() if k != "text"} | ({
        "retroScansAvoidable": "avoidable issues scanned (issue011)" in _retro05,
        "retroNamesGluedDocs": "five tests' doc comments" in _retro05,
        "retroNamesStaleCounts": "ten helpers, eight anchors" in _retro05,
        "retroNamesCliTestsOutOfScope": "keel-cli/tests holds three cwd-relative anchors outside the widened population" in _retro05,
        "retroNamesProbeFalseStart": "the D0388 discipline caught the probe, not the tree" in _retro05,
        "retroNamesScanNameKept": "keeps the name no_member_test_anchors_on_a_cwd_relative_path" in _retro05,
        "retroNoNewItemCount": _retro05.count("No new item - "),
        "retroFindings": len(re.findall(r"(?:^|[.;:] )\((\d)\) ", _retro05)),
        "dodResults": _dod_results("dcOneRepoRootHelper"),
    } if _s734.get("exists") else {}),
} if _d0505 and _ts05 and _adh05 and _tch05 else None,
     "the held record for a test-only edit to a locked guard source: Decision, the one helper and its callers, the anchor census over three trees, the locked file's own two-line diff, live lock verdict, issue557, sprint 734",
     _DEC_HOW + " Names by literal search in the field named; dependsOn = the `#DependsOn dependency from d0505 to dNNNN;` lines. source: "
     "members/keel-fs/src/test_support.rs for the pub fn, doc(hidden), the .git walk and its own test; members/keel-fs/src/lib.rs for `pub mod test_support;`; "
     "every .rs under keel-cli/src and each members/*/src (from the root Cargo.toml members list) scanned for a `fn repo_root(` definition, a "
     "`use keel_fs::test_support::repo_root;` line and a `keel_fs::test_support::repo_root()` call; the four anchor shapes of keel-cli/src/touched.rs ANCHORS counted "
     "over non-comment lines in keel-cli/src, members/*/src and keel-cli/tests (file:line each); keel-cli/src/adherence.rs's this_repo_yields_the_empty_prefix body and "
     "GUARD_SOURCE_FILES in members/keel-guards/src/enforcement.rs; the scan's body in touched.rs for its population call, filter, size and membership assertions and its message; "
     "each member Cargo.toml's [dev-dependencies] table for a keel-fs row; collapse_repo_root.py's not-an-instrument and Idempotent lines. landed = `git diff --name-status -M " +
     _LAND734_FROM + " " + _LAND734_TO + "` first letters counted, `--shortstat` over the range, `--numstat` and `-U0` over the range for keel-cli/src/adherence.rs alone "
     "(added and removed lines verbatim), the changed names held against the lock's file list and directory prefix. live: `" + KEEL +
     " gate guard process-change --no-receipt .` last line; the touched receipt as section 32 reads it. issue557 as section 34. resolverPositions as section 34; "
     "sprint734 from its delivery file, charter d0479, plus literal spans in the retro gate's procedureText; retroFindings = `(n) ` markers opening a sentence or "
     "following a label's colon in that text; retroNoNewItemCount = the guard's justification phrase counted; dodResults = the story's DoDRn outcomes and shas.")
# every fact above reads the WORKING TREE while `tree` names HEAD; when the two differ the page must say so
_DIRTY_HOW = ("`git status --porcelain --untracked-files=all`: lines beginning with a change code other than `??` are "
              "tracked files with uncommitted edits, `??` lines are untracked files. Every file-reading fact in this "
              "run (the CLI surface, the censuses, hardening) reads the working tree, so when `modified` is not zero "
              "the numbers describe HEAD plus these edits and the page's provenance names the count.")
ok, out = run(["git", "status", "--porcelain", "--untracked-files=all"])
if ok:
    _lines = [l for l in out.splitlines() if l.strip()]
    fact("treeUncommitted", {"modified": len([l for l in _lines if not l.startswith("??")]),
                             "untracked": len([l for l in _lines if l.startswith("??")])},
         "files differing from HEAD", _DIRTY_HOW)
else:
    fact("treeUncommitted", None, "files differing from HEAD", _DIRTY_HOW + " git status failed: " + out)

# ================================================================ emit
DOC = {
    "generatedAt": NOW.replace(microsecond=0).isoformat(),
    "tree": TREE,
    "facts": FACTS,
}

# A FAILED RUN MUST NOT LEAVE A CURRENT-LOOKING FILE (D0387/issue399). The file was claimed with a running
# stub before section 1 ran; a raise above rewrote it as failed; this is the one place `complete` becomes
# true, and the stdout copy carries the same field so a redirected copy is checked the same way.
DOC["complete"] = True
# the files this instrument is made of: a reader refuses this answer once any of them changes (artefact.py, issue560)
DOC = finish_json(OUT_PATH, DOC, sources=[os.path.abspath(__file__),
                                          os.path.join(REPO, "scripts", "module_home.py"),
                                          os.path.join(REPO, "scripts", "artefact.py")])
print(json.dumps(DOC, indent=2, sort_keys=False))   # the same document the file holds, instrument hashes included

