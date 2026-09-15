#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-15 (thirtieth) queue change: the queue emptied on the
2026-09-14 marks and five held Decisions now stand in it. Four came out of the Architecture phase of the modular-
members work - one fork (where a project's own Decision lives when it runs the items member alone) and a chain of
three one-clause process changes (a project's processes resolve from its tree; a step binds to a project Test;
process-gates is one rung at all three tiers) - and one is a safety change recorded on the human's own words the
same day (a starved cat is denied by the pre-bash hook). One tab per ask (D0404); every count is a facts.py fact,
never typed; each metric says in plain words what it is doing (issue562).

Usage: python scripts/exec_brief/build_2026_09_15_process_hook_chain_and_the_deny.py <facts.json> <previous.html> <out.html>
"""
import re
import sys
from html import escape

sys.path.insert(0, __file__.rsplit("/", 1)[0] if "/" in __file__ else __file__.rsplit("\\", 1)[0])
from charts import bars                          # noqa: E402
from logic_exhibits import downstream, logic_lanes     # noqa: E402
sys.path.insert(0, "scripts")
from artefact import claim, require_complete   # noqa: E402  (D0387: a stale answer is refused, a dead run leaves no page)
from check_templates import field_text, markup_only  # noqa: E402
from fit_check import assert_fits              # noqa: E402  (D0402: a page whose exhibits hide text is removed, not published)

facts_path, prev_path, out_path = sys.argv[1:4]
if prev_path == out_path:
    sys.exit("refusing: the style source and the output are the same file - copy the previous page aside first")
J = require_complete(facts_path)
if not (J.get("instrument") or {}).get("sources"):
    sys.exit("refusing: the facts carry no instrument hashes - they predate the issue560 control; re-run facts.py")
claim(out_path)
TREE = J["tree"]
DATE = J["generatedAt"][:10]
prev = open(prev_path, encoding="utf-8").read()
prev = prev[prev.index("<title>"):]                      # the host's own wrapper line, if the copy carries one, is not ours
head = prev[: prev.index('<div class="page">')]
tail = prev[prev.index("<script>"):]
tail = tail[: tail.index("</script>") + len("</script>")] + "\n"   # the host wraps the page in its own body; ours ends with the script


def v(name):
    x = J["facts"][name]["value"]
    if x is None:
        sys.exit(f"refusing: fact {name} is null - {J['facts'][name]['how']}")
    return x


# ---- the queue: one fork, a chain of three, one safety change ------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
QUEUE = ["d0487", "d0488", "d0489", "d0490", "d0491"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != 1 or not members[0]["fork"] or any(p["fork"] for p in members[1:]):
    sys.exit("refusing: the page says the scope Decision is the one fork; the lens disagrees")

CH = v("processHookChain")
D88, D89, D90 = CH["d0488"], CH["d0489"], CH["d0490"]
for d in (D88, D89, D90):
    if d["status"] != "proposed" or d["marker"] != "#ProspectiveChange" or not d["derivedFromSt126"]:
        sys.exit(f"refusing: a chain link is not a held process change derived from the statement: {d['title'][:60]}")
if D88["dependsOn"] != [] or D89["dependsOn"] != ["d0488"] or D90["dependsOn"] != ["d0489"]:
    sys.exit(f"refusing: the page draws the chain read -> bind -> rung; the edges are {D88['dependsOn']}, {D89['dependsOn']}, {D90['dependsOn']}")
if not D90["measuredMs"] or len(D90["measuredMs"]) != 2:
    sys.exit("refusing: the rung Decision carries no measured cost range")
MS_LO, MS_HI = D90["measuredMs"]

CE = v("processCursorCensus")
N_PROC, N_ADOPTED = CE["processFiles"], CE["adoptedCount"]
BOUND = CE["processesWithBoundSteps"]
N_BOUND, N_KINDS, N_GATE_KINDS = len(BOUND), len(CE["boundStepKinds"]), CE["gatePrefixedBindings"]
if CE["resolverProcessDirs"] != [".engine"] or CE["projectProcessDirExists"]:
    sys.exit(f"refusing: the page says the resolver walks the engine directory only and no project process dir exists; facts {CE['resolverProcessDirs']}, {CE['projectProcessDirExists']}")
if not CE["cursorStoredNowhere"] or CE["guardsReadingCursor"] or CE["preCommitRunsAdvance"] or CE["ciRunsAdvance"]:
    sys.exit("refusing: the page says no guard, hook or CI step reads the process cursor today; the facts say one does")
if any(k.startswith("test:") for k in CE["boundStepKinds"]):
    sys.exit("refusing: the page says no step binds to a project Test today; one does")
PRE_GATES, CI_GATES = CE["preCommitGateCalls"], CE["ciGateCalls"]

SF = v("decisionScopeFork")
if SF["status"] != "proposed" or SF["marker"] is not None or not SF["derivedFromSt126"] or SF["dependsOn"] != ["d0480"]:
    sys.exit("refusing: the fork must be held, unmarked, derived from the statement and depend on the member-boundary Decision")
OPTS = {o["letter"]: o for o in SF["options"]}
if sorted(OPTS) != ["A", "B", "C"] or SF["costsStated"] != 3 or not OPTS["A"]["recommended"] or OPTS["B"]["recommended"] or OPTS["C"]["recommended"]:
    sys.exit(f"refusing: three costed options with A recommended are expected; the record reads {[(o['letter'], o['recommended'], o['hasCost']) for o in SF['options']]}")
if SF["d0480Status"] != "accepted" or not SF["resolverKindAllowsDecision"]:
    sys.exit("refusing: the page says the member-boundary Decision is accepted and an Issue may be resolved by a Decision")

SC = v("starvedCatControl")
if SC["status"] != "proposed" or SC["marker"] != "#SafetyChange" or not SC["derivedFromSt127"]:
    sys.exit("refusing: the deny must be a held safety change derived from the human's statement")
ST = SC["statement"]
if not ST or ST["saidBy"] != "wweatherholtz" or "wrapper" not in ST["text"]:
    sys.exit("refusing: the page quotes the human's words; the record does not carry them")
I564 = SC["issue564"]
if not (I564["exists"] and I564["resolver"] == "d0491" and SC["detectorPresent"] and SC["detectorTest"] and SC["controlMapRow"]):
    sys.exit(f"refusing: the deny's Issue, detector, test or control row is missing: {I564}, {SC['detectorPresent']}, {SC['detectorTest']}, {SC['controlMapRow']}")
if SC["preBashDenies"] != 2 or SC["preBashDenyControls"][:2] != ["heredoc-backslash", "stdin-starved-write"] or not SC["backslashDenyNamesEditTool"]:
    sys.exit(f"refusing: the page says two denies, backslash then starved, and the first names the Edit tool: {SC['preBashDenies']}, {SC['preBashDenyControls']}")
N_DENIES = SC["preBashDenies"]
LEDGER_DENY = SC["ledgerRows"]["deny"]

AR = v("architectureReadBack")
N_SR, N_SSR = len(AR["systemRequirements"]), len(AR["subsystemRequirements"])
N_SR_ALLOC, N_SR_UNVERIFIED = len(AR["srAllocated"]), len(AR["srInVerifyGaps"])
if AR["needsInGaps"] or not AR["gateDeclared"] or AR["satisfyEdges"] != N_SR:
    sys.exit(f"refusing: the page says every Need is satisfied by one requirement: gaps {AR['needsInGaps']}, edges {AR['satisfyEdges']} of {N_SR}")
REQ_OF = {"d0487": SF["requirementsDerived"], "d0488": D88["requirementsDerived"], "d0489": D89["requirementsDerived"], "d0490": D90["requirementsDerived"]}
if any(not r for r in REQ_OF.values()):
    sys.exit(f"refusing: each Architecture Decision charters at least one requirement: {REQ_OF}")

tests, failing = v("suiteTests"), v("suiteFailed")
dirty = v("treeUncommitted")["modified"]


def courses(rows):
    body = "".join(f"<tr><td>{a}</td><td>{b}</td><td>{c}</td></tr>" for a, b, c in rows)
    return (f'<div class="tbl-wrap"><table><thead><tr><th>course</th><th>what changes</th><th>what it costs</th></tr></thead>'
            f'<tbody>{body}</tbody></table></div>')


# A record id inside a quoted title is rendered as the thing it names (the contract keeps ids out of the reader's
# text); the record keeps the id, the page says the words were glossed.
GLOSS = {"beside D0309": "beside the backslash deny"}
ID_RE = re.compile(r"\b(?:[Dd]0\d{3}|issue\d{3}|st\d{3}|us\d{3}|s{1,3}r[A-Z]\w+)\b")


def glossed(text):
    for k, g in GLOSS.items():
        text = text.replace(k, g)
    if ID_RE.search(text):
        sys.exit(f"refusing: a record id survives the gloss in a quoted statement: {text[:80]}")
    return text


def clause(d):
    """The Decision's title, quoted: `<name>: <clause>` - the reader gets the clause, the record keeps the name."""
    t = d["title"]
    return f'<p class="item"><q>{escape(glossed(t.split(": ", 1)[1] if ": " in t else t))}</q></p>'


def opts(radio, dname, rows):
    body = "".join(f'<label><input type="radio" name="{radio}" value="{escape(val, quote=True)}">{escape(lab)}</label>' for lab, val in rows)
    return f'<div class="opts" data-records="{dname}">{body}</div>'


# ---- exhibits: two per ask; every title is a claim with a verb -----------------------------------------
figS1 = logic_lanes(
    "Today an Issue closed by a Decision needs the whole binary",
    ("today", [
        ("a project tracks items", "", "accent", ""),
        ("Issue names a Decision", "", "warn", ""),
        ("needs the whole binary", "to read or write one", "bad", ""),
    ], ["", "", ""]),
    ("as A reads", [
        ("the same project", "items member", "accent", ""),
        ("reads it: resolved", "computed", "ok", ""),
        ("writes it: governance", "one acceptance channel", "ok", ""),
    ], ["", "", ""]),
)
figS2 = downstream(
    "Each letter lands on the same three records",
    ("the items member", "as its requirement is written", ""),
    [
        ("requirement it charters", "stands under A", f"{len(REQ_OF['d0487'])}", "ok"),
        ("boundary Decision", "narrowed under B", "1", "warn"),
        ("acceptance code paths", "A one, B two", "1", "ok"),
        ("projects on the member alone", "today", "0", "muted"),
    ],
)
figR1 = logic_lanes(
    f"The resolver walks {len(CE['resolverProcessDirs'])} directory, so a project process is never found",
    ("today", [
        ("a project process file", "plain SysML, in its tree", "accent", ""),
        ("resolver walks the engine dir", "only", "warn", ""),
        ("not found", "advance refuses the name", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("the same file", "", "accent", ""),
        ("one resolver, both dirs", "a name in both: refused", "ok", ""),
        ("found; enforced once adopted", "", "ok", ""),
    ], ["", "", ""]),
)
figR2 = downstream(
    f"The engine has {N_PROC} process files and enforces {N_ADOPTED}",
    ("process files", "engine directory", f"{N_PROC}"),
    [
        ("adopted", "activation contract", f"{N_ADOPTED}", "ok"),
        ("with a bound step", "a hook can refuse", f"{N_BOUND}", "ok"),
        ("project process dirs", "none yet", "0", "muted"),
    ],
)
figB1 = bars(
    f"{sum(BOUND.values())} steps are bound across {N_BOUND} processes, all by engine code",
    [(p, n, "ok", "") for p, n in sorted(BOUND.items(), key=lambda kv: (-kv[1], kv[0]))],
    unit=("step", "steps"),
)
figB2 = logic_lanes(
    f"All {N_KINDS} binding kinds are engine-owned; the clause makes a project Test one",
    ("today", [
        (f"gate:<phase> ({N_GATE_KINDS}) or a guard", "engine code decides", "accent", ""),
        ("a project's own Test", "cannot decide a step", "bad", ""),
    ], ["", ""]),
    ("as the clause reads", [
        ("test:<name>", "a Test in the project", "accent", ""),
        ("latest result under HEAD", "pass green; none: hard", "ok", ""),
    ], ["", ""]),
)
figG1 = logic_lanes(
    f"Nothing reads step order over the record today: {CE['guardsReadingCursor']} guards do",
    ("today", [
        ("advance refuses when asked", "", "ok", ""),
        ("a result by another command", "past the red step", "warn", ""),
        ("no guard reads the order", "", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("the same record", "", "accent", ""),
        ("one rung, one fixed line", "", "ok", ""),
        ("hook, commit gate, CI", "print it unchanged", "ok", ""),
    ], ["", "", ""]),
)
figG2 = downstream(
    "The rung lands on three surfaces that already call the gate",
    ("one function", "guards member", ""),
    [
        ("editor hook", f"{MS_LO}-{MS_HI} ms per adopted process", f"{N_ADOPTED}", "warn"),
        ("commit gate", "gate calls today", f"{PRE_GATES}", "ok"),
        ("CI", "gate calls today", f"{CI_GATES}", "ok"),
    ],
)
figD1 = logic_lanes(
    "A starved cat waits the whole timeout; the hook now stops it first",
    ("what happened", [
        ("cat > file, nothing feeding it", "", "accent", ""),
        ("waits on stdin", "never closes", "bad", ""),
        ("empty file; no deny fired", "", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("the same command", "", "accent", ""),
        ("denied before it runs", "", "ok", ""),
        ("reason names Edit / Write", "byte-exact channels", "ok", ""),
    ], ["", "", ""]),
)
figD2 = downstream(
    f"The hook has {N_DENIES} blocking verdicts now, each ledgered by name",
    ("pre-bash denies", "built hook", f"{N_DENIES}"),
    [
        ("detector with its pair", "shell-check module", "1", "ok"),
        ("control row on the hang hazard", "beside the backslash deny", "1", "ok"),
        ("Issue, this Decision resolves", f"severity {I564['severity']}", "1", "ok"),
        ("live denies ledgered", "this machine", f"{LEDGER_DENY}", "ok"),
    ],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0487", "ask-scope", "Scope fork", "scope"),
    ("d0488", "ask-read", "Read", "read"),
    ("d0489", "ask-bind", "Bind", "bind"),
    ("d0490", "ask-rung", "Rung", "rung"),
    ("d0491", "ask-deny", "Deny", "deny"),
]
panel_bodies = {
    "scope": f"""<h2>Where a project's own Decision lives on the items member</h2>
{clause(SF)}
<p><strong>The fork:</strong> your statement kept <q>decisions about keel itself</q> out of the items member. Read wide, the member only reads Decisions (A). Read narrow, a project's own <q>we will not do X</q> is an item it records and accepts (B), narrowing yesterday's boundary.</p>
{figS1}
{figS2}
{courses([
    ("A: read-only (recommended)", "resolves an Issue by a Decision it reads; writing needs governance", "an items-only project installs the larger build to say 'we won't'"),
    ("B: own Decision is an item", "record and accept ship in the member", "the acceptance channel compiled twice; the boundary narrowed"),
    ("C: no Decision type", "an Issue closes on a backlog action only", "'decided not to' becomes a stored verdict"),
    ("Do nothing", "nothing", "the member's verb list stays open"),
])}
<p><strong>True:</strong> the boundary is accepted; an Issue may be resolved by a Decision; no project runs the member alone. <strong>Mine:</strong> A. <strong>What decides it:</strong> may an items-only project say <q>we won't</q> without the larger build. <strong>Wrong if</strong> the first such project needs a Decision early.</p>
{opts("ask-scope", "d0487", [
    ("A: read-only (recommended)", "Option A: keel-issues knows the Decision type read-only - it resolves an Issue by a Decision and computes it resolved - but carries no record decision and no accept; a project that authors Decisions depends on the governance member (recommended)"),
    ("B: own Decision is an item", "Option B: keel-issues carries record decision and accept for a project's own Decisions; governance markers stay in the governance member; the boundary clause is narrowed by a SupersedeClause"),
    ("C: no Decision type", "Option C: keel-issues does not know the Decision type; an Issue there is resolved by a backlog action only"),
    ("Do nothing", "Do nothing: the fork stays open and the items member's verb list stays unclosed"),
])}""",
    "read": f"""<h2>A project's processes resolve from its own tree</h2>
{clause(D88)}
<p><strong>What changes:</strong> one resolver walks the engine's process directory and the project's; a name in both is refused with both paths named, never shadowed. Adoption stays separate: a file enforces nothing until the activation contract names it. <strong>Contested:</strong> refuse a collision, against letting the project's copy win.</p>
{figR1}
{figR2}
{courses([
    ("Accept (recommended)", "both directories, one resolver, collision refused", "a project supersedes an engine step; it cannot shadow it"),
    ("Project wins", "the project's definition takes precedence", "an engine gate silently redefined by a project file"),
    ("Do nothing", "resolver stays engine-only", "a project process is enforced only by copying it into the engine"),
])}
<p><strong>True:</strong> one directory walked; no project process directory exists; {N_ADOPTED} of {N_PROC} adopted. <strong>Mine:</strong> the collision rule. <strong>What decides it:</strong> may a project redefine an engine step by name - <q>absolute discipline</q> says no. <strong>Wrong if</strong> you meant override as a feature.</p>
{opts("ask-read", "d0488", [
    ("Accept (recommended)", "Accept: the process resolver reads both .engine/processes and the project's .tracking/processes through one function; a name in both is refused; adoption stays with the activation contract (recommended)"),
    ("Accept, project wins", "Accept with a change: the project's definition takes precedence over an engine process of the same name instead of a refusal"),
    ("Do nothing", "Do nothing: the resolver stays engine-only; a project process must be copied into .engine to be enforced"),
])}""",
    "bind": f"""<h2>A step binds to a Test in the project's own model</h2>
{clause(D89)}
<p><strong>What changes:</strong> a step's check may name <q>test:&lt;name&gt;</q>, any Test in the project. Green when that Test's latest result on a commit under HEAD passes with no later red; red otherwise; a name resolving to no Test is a hard failure, not a step gone quietly unchecked. <strong>Plainly:</strong> today every check is engine code; a project cannot make its own Test decide a step.</p>
{figB1}
{figB2}
{courses([
    ("Accept (recommended)", "test:<name>; latest result under HEAD decides; unknown is hard", "a Test renamed without its step is red until the step follows"),
    ("Newest by date", "the newest result anywhere decides", "a pass on an abandoned branch greens a step here"),
    ("Unknown warns", "an unresolvable name warns", "a step can silently go unchecked"),
    ("Do nothing", "checks stay engine-owned", "custom discipline stays a guard the engine must ship"),
])}
<p><strong>True:</strong> {N_KINDS} kinds, none a project Test. <strong>Mine:</strong> under-HEAD over newest-by-date, hard over warn. <strong>What decides it:</strong> should an unresolvable check stop a commit or note it - your value. <strong>Wrong if</strong> your tests are recorded on branches meant to green steps on main.</p>
{opts("ask-bind", "d0489", [
    ("Accept (recommended)", "Accept: a ProcessStep's check binding may read test:<name>; the latest TestResult on an ancestor of HEAD decides; an unresolvable name is a hard step-check failure (recommended)"),
    ("Accept, newest by date", "Accept with a change: the newest TestResult by date decides, regardless of which commit it was judged against"),
    ("Accept, unknown warns", "Accept with a change: a test name that resolves to no Test is a warning, not a hard failure"),
    ("Do nothing", "Do nothing: step checks stay engine-owned guard names and phase gates"),
])}""",
    "rung": f"""<h2>One gate rung refuses a step recorded past a red, alike in three places</h2>
{clause(D90)}
<p><strong>What changes:</strong> one forward guard, <q>process-gates</q>, flags each later step holding a result where an earlier step's check is red: one fixed-format line. Hook, commit gate and CI call the same function and print it unchanged. <strong>The metric:</strong> {MS_LO}-{MS_HI} ms per adopted process is how much slower each editor-hook fire gets, at most, until the rung shares one tree read.</p>
{figG1}
{figG2}
{courses([
    ("Accept (recommended)", "the rung joins all three tiers", f"up to {MS_HI} ms per adopted process per hook fire"),
    ("Refuse at asking only", "advance refuses; nothing reads the record", "a result written past a red elsewhere is never caught"),
    ("Do nothing", "nothing", "a process is read and bound, never enforced over its record"),
])}
<p><strong>True:</strong> {CE['guardsReadingCursor']} guards read step order; the gate fires at {PRE_GATES} commit-hook sites and {CI_GATES} CI sites; the cost is measured. <strong>Mine:</strong> a guard over the record. <strong>What decides it:</strong> may the editor hook pay that cost, observable in its latency after landing. <strong>Wrong if</strong> the fast tier goes past budget.</p>
{opts("ask-rung", "d0490", [
    ("Accept (recommended)", "Accept: one forward guard process-gates over every adopted process, one fixed-format violation line, called unchanged by gate --fast, gate --workspace and CI (recommended)"),
    ("Refuse at asking only", "Reject the rung: keel advance keeps refusing at the moment of asking and no guard reads the record"),
    ("Do nothing", "Do nothing: the rung is not built; custom processes are read and bound but not enforced over their record"),
])}""",
    "deny": f"""<h2>A starved cat is denied before it hangs the tool</h2>
<p><strong>Your words</strong> ({escape(ST['saidAt'])}): <q>{escape(ST['text'])}</q></p>
{clause(SC)}
<p><strong>What hung</strong> was a redirected <q>cat</q> with nothing feeding it, waiting on a stdin this harness never closes; the one deny did not apply. <strong>The wrapper</strong> is the harness's byte-exact channel, Edit and Write, and both denies now name it. Shipped; held because a blocking verdict is a safety change.</p>
{figD1}
{figD2}
{courses([
    ("Accept (recommended)", "second deny stands; both name Edit and Write", "one false positive, a shape never used here"),
    ("Warn, don't block", "starved shape advisory", "the hang recurs when advice goes unread"),
    ("Deny, old message", "backslash reason unchanged", "the remedy text reproduces the two-step"),
    ("Do nothing", "rule withdrawn", "a control that fired is removed by hand"),
])}
<p><strong>True:</strong> detector, pair, control row, Issue, {LEDGER_DENY} live deny. <strong>Mine:</strong> deny over warn; a starved cat is never useful here. <strong>What decides it:</strong> does <q>be done with this issue</q> mean block, or tell. <strong>Wrong if</strong> a shape you use is caught; the ledger would name it.</p>
{opts("ask-deny", "d0491", [
    ("Accept (recommended)", "Accept: the pre-bash hook denies a pipeline-head cat or tee with nothing feeding it, and the backslash deny's reason names the Edit tool for a replacement and the Write tool for a new file (recommended)"),
    ("Warn, don't block", "Accept with a change: the starved-cat rule is advisory, not a deny; the message change stands"),
    ("Deny, old message", "Accept with a change: the deny stands but the backslash deny's reason text is left as it was"),
    ("Do nothing", "Do nothing: withdraw the rule from the hook; the Issue stays open"),
])}""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Accept the three process-hook clauses in order</strong> - Read, Bind, Rung - <strong>and the starved-cat deny</strong>; on the scope fork <strong>take A</strong>: the items member reads Decisions, never writes them. <strong>Wrong if</strong> a project on that member alone needs its own <q>we won't</q> early.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> 1</span><span class="chip"><b>Process changes, chained</b> 3</span><span class="chip"><b>Safety change</b> 1</span><span class="chip"><b>Requirements they govern</b> {N_SR}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing. Clauses quoted; record names glossed. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept the process-hook chain and the deny; take A on the scope fork"
SUB = "Five held Decisions: four from the Architecture phase, one from your words today"


def render(panel_bodies):
    tabs = "".join(
        f'<button role="tab" aria-selected="{"true" if i == 0 else "false"}" aria-controls="panel-{key}" id="tab-{key}" type="button">{label}</button>'
        for i, (_d, _n, label, key) in enumerate(ASKS))
    panels = "".join(
        f'<section role="tabpanel" id="panel-{key}" aria-labelledby="tab-{key}"{"" if i == 0 else " hidden"}>{panel_bodies[key]}</section>'
        for i, (_d, _n, _label, key) in enumerate(ASKS))
    return frame(panels, tabs, TITLE, SUB)


body = render(panel_bodies)
panel_words = {k: words(p) for k, p in panel_bodies.items()}
frame_words = words(body) - sum(panel_words.values())

# ---- the shell's tab strip: CSS into the head, wiring into the tail (added once; the source page may lack it) ----
TAB_CSS = '''
.tabs{display:flex;gap:4px;flex-wrap:wrap;border-bottom:1.5px solid var(--line);margin:22px 0 6px}
.tabs [role="tab"]{background:none;border:0;border-bottom:3px solid transparent;margin-bottom:-1.5px;padding:10px 12px;cursor:pointer;color:var(--muted);font:500 12px/1.3 "Roboto Condensed",sans-serif;letter-spacing:.12em;text-transform:uppercase;min-height:44px}
.tabs [role="tab"][aria-selected="true"]{color:var(--head);border-bottom-color:var(--accent)}
.tabs [role="tab"]:hover{color:var(--head)}
.tabs [role="tab"]:focus-visible{outline:2px solid var(--accent);outline-offset:-2px}
[role="tabpanel"]{display:block}
[role="tabpanel"][hidden]{display:none}
.m{display:inline-block;font:400 12.5px/1.2 "Roboto Mono",Consolas,monospace;border:1px solid var(--line);border-radius:3px;padding:1px 5px;margin:1px 2px}
'''
if ".tabs{" not in head:
    assert head.count("</style>") == 1
    head = head.replace("</style>", TAB_CSS + "</style>")
ITEM_CSS = ".item{border-left:3px solid var(--accent);padding:2px 0 2px 12px;margin:10px 0}\n"
if ".item{" not in head:
    head = head.replace("</style>", ITEM_CSS + "</style>")
TAB_JS = '''
(function(){"use strict";
var tabs=Array.prototype.slice.call(document.querySelectorAll('[role="tab"]'));
function show(tab){tabs.forEach(function(t){var on=t===tab;t.setAttribute('aria-selected',on?'true':'false');t.tabIndex=on?0:-1;
  var p=document.getElementById(t.getAttribute('aria-controls'));if(p)p.hidden=!on});tab.focus();
  try{localStorage.setItem('keel-brief-tab:'+document.title,tab.id)}catch(e){}}
tabs.forEach(function(t,i){t.addEventListener('click',function(){show(t)});
  t.addEventListener('keydown',function(e){var j=e.key==='ArrowRight'?i+1:e.key==='ArrowLeft'?i-1:e.key==='Home'?0:e.key==='End'?tabs.length-1:null;
    if(j===null)return;e.preventDefault();show(tabs[(j+tabs.length)%tabs.length])})});
try{var k=localStorage.getItem('keel-brief-tab:'+document.title);var t=k&&document.getElementById(k);if(t)show(t)}catch(e){}
})();
</script>'''
if "keel-brief-tab:" not in tail:
    assert tail.count("</script>") == 1
    tail = tail.replace("</script>", TAB_JS)
DIGEST_OLD = ".page > h2, .page > h3, .page > p, .page > ul, .page > blockquote, .page > .ask, .page > .turn, .page > .tbl-wrap, .page > .opts, figure.diagram > .msg"
DIGEST_NEW = ".page > .ask, .page > p, [role=\"tabpanel\"] > h2, [role=\"tabpanel\"] > p, [role=\"tabpanel\"] > .tbl-wrap, [role=\"tabpanel\"] > .opts, figure.diagram > .msg"
if DIGEST_OLD in tail:
    tail = tail.replace(DIGEST_OLD, DIGEST_NEW)
assert DIGEST_NEW in tail, "the digest must walk every panel, hidden or not (D0404)"

page = head + body + tail
open(out_path, "w", encoding="utf-8").write(page)
print(f"wrote {out_path}: {len(page)} bytes; frame {frame_words} words; panels " +
      ", ".join(f"{k} {n}" for k, n in sorted(panel_words.items(), key=lambda kv: -kv[1])) +
      f"; per tab " + ", ".join(f"{k} {frame_words + n}" for k, n in panel_words.items()) +
      f"; summed {words(page)}; {pending} held, the chain {D88['dependsOn']}<-{D89['dependsOn']}<-{D90['dependsOn']}; rung {MS_LO}-{MS_HI} ms")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
