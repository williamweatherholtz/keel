#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-19 (fifty-third) queue change: the two held records of the fifty-second
page were accepted from chat and recorded; six held forks take their place. The first is the human's own sentence - "We need a
permanent solution to decision capture" - answered as a process-change fork: the brief page captures the verdict itself, a tap
written into the page's next version by the viewer's browser and read back by the keel session that published it (OPTION A),
or the serve console is hosted remotely (B), or nothing changes (C). THIS PAGE CARRIES THE PROTOTYPE: a Record button writes
every chosen course into a JSON block the page itself carries and republishes the page through the host's runtime; the human's
answer to that fork is its first capture. The other five are architecture asks the downstream batch (GH#9..GH#95, routed by
one accepted Decision) surfaced, each a fork of its own: project-supplied critique lenses, a per-sprint ceremony chain, marker
edges traversable by name, analysis suspicion along declared edges, a DesignInput item type. Six tabs, six asks (D0404); every
count is a facts.py fact, never typed; each metric says what it does.

Usage: python scripts/exec_brief/build_2026_09_19_six_held.py <facts.json> <previous.html> <out.html>
"""
import re
import sys
from html import escape

sys.path.insert(0, __file__.rsplit("/", 1)[0] if "/" in __file__ else __file__.rsplit("\\", 1)[0])
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


# ---- the queue: six held forks ---------------------------------------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
QUEUE = ["d0525", "d0527", "d0528", "d0529", "d0530", "d0531"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != len(QUEUE) or not all(m["fork"] for m in members):
    sys.exit(f"refusing: the page says every held Decision is a fork; the lens says {[m['slug'] for m in members if not m['fork']]} are not")
tests, failing = v("suiteTests"), v("suiteFailed")
dirty = v("treeUncommitted")["modified"]

F = v("oneSurfaceFiveForks")
DEC = {"d0525": F["surface"], **F["forks"]}
for _d in QUEUE:
    _x = DEC[_d]
    if not (_x["fileExists"] and _x["held"] and _x["fork"] and _x["options"] == ["A", "B", "C"] and _x["recommended"] == "A"
            and _x["costPerOption"] == 3 and _x["research"] and _x["derivedFrom"]):
        sys.exit(f"refusing: {_d} must be a held three-way fork with a recommended A, a COST per option, a RESEARCH line and a derivation: "
                 f"{ {k: _x[k] for k in ('fileExists', 'held', 'fork', 'options', 'recommended', 'costPerOption', 'research', 'derivedFrom')} }")
if DEC["d0525"]["marker"] != "#ProspectiveChange":
    sys.exit(f"refusing: the surface fork is a process change and must carry the marker: {DEC['d0525']['marker']}")
for _d in QUEUE[1:]:
    if DEC[_d]["marker"] is not None:
        sys.exit(f"refusing: {_d} is an architecture fork, not a process change; it carries {DEC[_d]['marker']}")
    if len(DEC[_d]["ghIssues"]) < 1 or DEC[_d]["storiesRouted"] < 1:
        sys.exit(f"refusing: {_d} must trace to a downstream issue and carry a routed story: {DEC[_d]['ghIssues']}, {DEC[_d]['storiesRouted']}")
RT = F["routing"]
if not (RT["accepted"] and not RT["fork"] and RT["namesNotAProcessChange"] and RT["storiesRouted"] >= 1):
    sys.exit(f"refusing: the batch routing Decision must be accepted, unforked, say it is not a process change and carry the routed stories: "
             f"{ {k: RT[k] for k in ('accepted', 'fork', 'namesNotAProcessChange', 'storiesRouted')} }")
B = F["batch"]
if not (B["everyIssueHasResolver"] and B["everyResolverDodNamesIssue"] and B["intakeExit0"] and B["intakeUnparsed"] == 0):
    sys.exit(f"refusing: every Issue minted today must have a resolver whose DoD names it, and intake must parse clean: {B}")
L = F["live"]
for _g in ("untrustedRouting", "issues", "judgmentRequestQuality"):
    if not (L[_g]["exit0"] and L[_g]["verdict"] == "PASS" and L[_g]["violations"] == 0):
        sys.exit(f"refusing: the page says {_g} is green on this tree: {L[_g]}")
T = F["today"]
if not (T["issue632"]["exists"] and T["issue632"]["resolverAsExpected"] and T["issue632"]["resolverDodNamesIssue"] and T["resolver632"]["onBacklog"]):
    sys.exit(f"refusing: the capture finding, its resolver edge and the resolver on the backlog are what the surface tab rests on: {T['issue632']} {T['resolver632']}")
if not T["policyHasDelegatedRecording"]:
    sys.exit("refusing: the surface tab says the delegation lives in attestation-policy.toml today")
N_ISSUES, N_STORIES_ROUTED = B["issuesToday"], B["storiesToRouting"] + B["storiesToAFork"]
N_GH, N_OPEN = B["ghIssuesIngestedEver"], L["openIssueCount"]
N_UR_SCANNED, N_JR_SCANNED = L["untrustedRouting"]["scanned"], L["judgmentRequestQuality"]["scanned"]
N_STATEMENTS = B["intakeStatements"]
SEV = B["severity"]

ID_RE = re.compile(r"\b(?:[Dd]0\d{3}|issue\d{3}|GH#\d+|st\d{3}|us\d{3}|dc[A-Z][A-Za-z]{4,})\b")


def glossed(text):
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


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


# ---- exhibits: the two mandatory logic exhibits per ask; every title is a claim with a verb ----------------
figS1 = logic_lanes(
    "Today the verdict is retyped; as the clause reads, the page carries it",
    ("today", [
        ("brief published", "read only", "accent", ""),
        ("human chooses", "types in a terminal", "bad", ""),
        ("session records", "retyped words", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("brief published", "courses live", "accent", ""),
        ("tap on the page", "next version written", "ok", ""),
        ("session reads back", "records, cites version", "ok", ""),
    ], ["then", "so"]),
)
figS3 = downstream(
    "One tap lands on the record, the delegation, the fallbacks and this page",
    ("the page is the verdict surface", "", ""),
    [
        ("acceptance record", "words, viewer id, version", "1", "ok"),
        ("recording delegation", "the first tap binds you", "1", "ok"),
        ("fallback transports", "console, gesture, chat", "3", "muted"),
        ("inbox design", "set aside", "1", "bad"),
    ],
)
figL1 = logic_lanes(
    "A lens the canon cannot name is red, or critique is off",
    ("today", [
        ("project declares a lens", "own package", "accent", ""),
        ("guard reads one enum", "unknown, red", "bad", ""),
        ("project's choice", "wrong lens, or none", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("policy names the package", "one key", "accent", ""),
        ("both readers accept it", "canon and project", "ok", ""),
        ("unknown name", "still fails loud", "ok", ""),
    ], ["then", "so"]),
)
figL3 = downstream(
    "One policy key lands on two readers and one onboarding step",
    ("a project supplies lenses", "policy names the package", ""),
    [
        ("critique guard", "reads both enums", "1", "ok"),
        ("coverage lens", "reads both enums", "1", "ok"),
        ("onboarding step", "states the rule", "1", "ok"),
        ("lens no question names", "the project's defect", "0", "muted"),
    ],
)
figC1 = logic_lanes(
    "A renamed ceremony step is dropped today with no refusal",
    ("today", [
        ("project renames a step", "its workflow", "accent", ""),
        ("advance matches baked names", "no match", "bad", ""),
        ("step dropped", "silently", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("sprint file declares gates", "in order", "accent", ""),
        ("each names a process", "the workflow declares", "ok", ""),
        ("unknown gate", "refused, names listed", "ok", ""),
    ], ["then", "so"]),
)
figC3 = downstream(
    "A per-sprint chain lands on advance, orient and two ceremony briefs",
    ("the chain is the project's", "gates the sprint file declares", ""),
    [
        ("advance", "derives the chain", "1", "ok"),
        ("orient", "shows which chain", "1", "ok"),
        ("closure brief", "reads the chain", "1", "ok"),
        ("delegated-ceremony brief", "reads the chain", "1", "ok"),
    ],
)
figM1 = logic_lanes(
    "A project's own edge kind is validated, guarded and invisible to every view",
    ("today", [
        ("project declares a marker", "own edge kind", "accent", ""),
        ("views know embedded kinds", "traverse errors", "bad", ""),
        ("safety view", "falls back to verify", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("kinds derived from the model", "embedded and project", "accent", ""),
        ("traverse by name", "works on declaration", "ok", ""),
        ("name collides", "refused at declaration", "ok", ""),
    ], ["then", "so"]),
)
figM3 = downstream(
    "One derived vocabulary lands on the view engine and the schema reader",
    ("a marker is a traversable kind", "every declared def", ""),
    [
        ("view engine", "knows the project's kinds", "1", "ok"),
        ("schema reader", "reads the model", "1", "ok"),
        ("colliding name", "refused", "1", "ok"),
        ("existing views", "unchanged", "0", "muted"),
    ],
)
figA1 = logic_lanes(
    "The architecture changes and the safety analysis stays green",
    ("today", [
        ("architecture element changes", "a commit", "accent", ""),
        ("suspect follows verify edges", "analysis edges are markers", "bad", ""),
        ("analysis", "diverges, no signal", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("def carries suspicion", "direct or transitive", "accent", ""),
        ("suspect follows the path", "stops at a later stamp", "ok", ""),
        ("hazards, losses", "reached by no edge", "ok", ""),
    ], ["then", "so"]),
)
figA3 = downstream(
    "A declared edge property lands on the schema, the analyst and the suspect lens",
    ("suspicion follows declared edges", "a schema addition", ""),
    [
        ("schema", "one property, one stamp", "2", "ok"),
        ("analyst", "stamps a reassessment", "1", "ok"),
        ("suspect lens", "names the marker path", "1", "ok"),
        ("hazards and losses", "untouched", "0", "muted"),
    ],
)
figD1 = logic_lanes(
    "A value the design consumes is typed as something nothing may gate on",
    ("today", [
        ("available flow", "typed as an indicator", "accent", ""),
        ("constraint gates on it", "invariant says no threshold", "bad", ""),
        ("calculations", "cite nothing", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("design input item", "value, unit, source, as of", "accent", ""),
        ("constraint consumes it", "typed edge", "ok", ""),
        ("gated", "core untouched", "ok", ""),
    ], ["then", "so"]),
)
figD3 = downstream(
    "One non-core type lands on a package, a guard, a lens and not the frozen core",
    ("a design input is an item type", "a fact about the world", ""),
    [
        ("schema package", "declares the type", "1", "ok"),
        ("guard", "no source, refused", "1", "ok"),
        ("lens", "inputs and consumers", "1", "ok"),
        ("frozen core", "untouched", "0", "muted"),
    ],
)

ASKS = [
    ("d0525", "ask-surface", "Verdict surface", "surface"),
    ("d0527", "ask-lens", "Project lenses", "lens"),
    ("d0528", "ask-chain", "Ceremony chain", "chain"),
    ("d0529", "ask-marker", "Marker edges", "marker"),
    ("d0530", "ask-suspect", "Analysis suspicion", "suspect"),
    ("d0531", "ask-input", "Design input", "input"),
]
_SEVS = ", ".join(f"{n} {s.lower()}" for s, n in sorted(SEV.items(), key=lambda kv: -kv[1]))
panel_bodies = {
    "surface": f"""<h2>The brief page captures the verdict itself</h2>
{clause(DEC["d0525"])}
<p><strong>What happened:</strong> your words - <q>we need a permanent solution to decision capture</q> - after a fresh project's Decisions sat proposed behind a terminal.</p>
<p><strong>What changes:</strong> the courses here are live. <em>Record</em> writes each chosen course, your note, viewer id and time into this page's state and republishes it from your browser; the publishing session reads it back and records the acceptance citing that version. Your first tap binds your viewer id to you.</p>
{figS1}
{figS3}
<p><strong>True:</strong> the host lets a page publish its next version and read the viewer's opaque id; the delegation is hand-edited today. <strong>Mine:</strong> a vendor page beats a hosted console. <strong>Decides it:</strong> whether a tap by anyone who can edit the page is signature enough. <strong>Wrong if</strong> the write fails here - the status line says how.</p>
{opts("ask-surface", "d0525", [
    ("Accept: the page is the surface (recommended)", "OPTION A: the decision brief page is the verdict surface - a tap writes a verdict record into the page's own state, the page publishes its next version from the human's browser, the publishing keel session reads it back and records the acceptance citing that version; the first tap binds the viewer id to a Person as the recording delegation; serve console, GitHub gesture and chat words are fallbacks; the inbox design is retired (recommended)"),
    ("Host the serve console remotely", "OPTION B: host the existing keel serve console remotely per project so the device-receipt tap is reachable from a phone"),
    ("Do nothing", "OPTION C: do nothing - the brief stays read-only and verdicts stay quoted chat words where the policy grants them"),
])}""",
    "lens": f"""<h2>A project supplies its own critique lenses</h2>
{clause(DEC["d0527"])}
<p><strong>What happened:</strong> a domain project declared a non-core package with the lens it needed; the critique guard reads one enum, so the name is red however the project spells it. Its choices were a software lens on a plot, or critique off.</p>
<p><strong>What changes:</strong> the critique policy gains one key naming a project package whose members are accepted lens names beside the canon. Guard and coverage lens read both and still fail loud on a name neither declares. Onboarding states the rule.</p>
{figL1}
{figL3}
<p><strong>True:</strong> the guard reads one enum; the reporter's package exists downstream; the story is routed and the routing guard is green over {N_UR_SCANNED} stories. <strong>Mine:</strong> a lens no critique question names is the project's defect. <strong>Decides it:</strong> whether the engine may accept a lens it cannot judge. <strong>Wrong if</strong> a project lens ever needs an engine check behind it.</p>
{opts("ask-lens", "d0527", [
    ("Accept: the policy names a lens package (recommended)", "OPTION A: critique-policy.toml gains lensPackage naming a non-core schema package whose enum members are accepted lens names alongside CritiqueLens; guard critique and critique-coverage read both and still fail loud on a name neither declares; project-onboarding step 3 states the rule (recommended)"),
    ("Document the path only", "OPTION B: onboarding step 3 and the policy header say a lens the canon cannot express is a new non-core package; the readers stay as they are"),
    ("Do nothing", "OPTION C: nothing changes - a domain project critiques a plot for testability or turns critique off"),
])}""",
    "chain": f"""<h2>The ceremony chain is the project's</h2>
{clause(DEC["d0528"])}
<p><strong>What happened:</strong> a downstream project renamed a ceremony step in its workflow. <em>advance</em> matches six baked names, found none, and moved on: a gate vanished with no refusal - the failure the honest-state rule forbids.</p>
<p><strong>What changes:</strong> <em>advance</em> derives the chain from the gates the sprint file declares, in order, each matched to a process the active workflow declares by name; a gate naming no declared process is refused with the names available.</p>
{figC1}
{figC3}
<p><strong>True:</strong> the matching is by baked name in the cursor code; the reporter's transcripts show the drop; the story is routed. <strong>Mine:</strong> one project's sprints running different ceremonies is a feature orient must show. <strong>Decides it:</strong> whether a project may substitute a ceremony, or only omit one. <strong>Wrong if</strong> a file declaring no gates must still run the six.</p>
{opts("ask-chain", "d0528", [
    ("Accept: the sprint file declares the chain (recommended)", "OPTION A: advance derives a sprint's chain from the gates its sprint file declares, in declaration order, each gate matched to a Process the active workflow declares by the name it carries; a gate naming no declared Process is refused with the names available (recommended)"),
    ("Keep the six baked names", "OPTION B: keep the six names baked and document that a project omits, never substitutes"),
    ("Do nothing", "OPTION C: nothing changes - a rename keeps silently dropping a step"),
])}""",
    "marker": f"""<h2>A project's marker edges are traversable by name</h2>
{clause(DEC["d0529"])}
<p><strong>What happened:</strong> a project declared its own edge kind, validated and guarded, and asked a view to traverse it. The view vocabulary is the embedded kinds; the error named them and the safety viewpoint fell back to the verify slice.</p>
<p><strong>What changes:</strong> the edge-kind vocabulary is derived from every metadata def the model declares, embedded and project alike, so a traverse works the day the def lands. An undeclared kind is still an error naming the known set; a name colliding with an embedded kind is refused at declaration.</p>
{figM1}
{figM3}
<p><strong>True:</strong> the schema reader reads one directory; the marker-vocabulary guard exists; the story is routed. <strong>Mine:</strong> deriving beats a target filter on <em>dependency</em>, which widens every view silently. <strong>Decides it:</strong> whether a project may extend the view vocabulary at all. <strong>Wrong if</strong> two projects in one repository declare the same name.</p>
{opts("ask-marker", "d0529", [
    ("Accept: kinds are derived from every declared def (recommended)", "OPTION A: the view engine's edge-kind vocabulary is derived from every metadata def the model declares, embedded and project alike, lower-cased by the same rule; an undeclared kind is still an error naming the known set; a project marker colliding with an embedded kind is refused at declaration (recommended)"),
    ("A dependency filter instead", "OPTION B: dependency covers every marker edge and views filter with a target on the marker name"),
    ("Do nothing", "OPTION C: nothing changes - a project's own edge vocabulary is authored, validated, guarded and invisible to every view"),
])}""",
    "suspect": f"""<h2>Analysis suspicion follows the edges the schema declares</h2>
{clause(DEC["d0530"])}
<p><strong>What happened:</strong> the project carrying a safety analysis of twenty-five requirements changed an architecture element. Suspicion follows verify edges; the analysis edges are markers, so the analysis stayed green and diverged with no signal.</p>
<p><strong>What changes:</strong> a metadata def may declare that it carries suspicion, direct or transitive; analysis elements gain a reassessment stamp; suspicion follows every such edge from a changed element downstream, stopping where the stamp is later than the change. Hazards and losses are reached by no such edge.</p>
{figA1}
{figA3}
<p><strong>True:</strong> the analysis edges are markers; the suspect lens reads verify edges; the story is routed. <strong>Mine:</strong> a stamp the analyst writes is a real obligation, and a weaker proxy is worse. <strong>Decides it:</strong> whether the engine or the project owns this signal. <strong>Wrong if</strong> a transitive path reaches a hazard.</p>
{opts("ask-suspect", "d0530", [
    ("Accept: declared edges carry suspicion (recommended)", "OPTION A: a metadata def may declare carriesSuspicion (direct or transitive), analysis elements gain reassessedAtCommit, and the suspect computation follows every declared suspicion-carrying edge from a changed element to the analysis elements downstream, stopping at any element whose stamp is later than the change; hazards and losses are reached by no such edge (recommended)"),
    ("A project view instead", "OPTION B: the project computes it in a TOML view over its own markers, no schema change and no stamp"),
    ("Do nothing", "OPTION C: nothing changes - the analysis diverges from the code with no signal"),
])}""",
    "input": f"""<h2>A design input is an item type</h2>
{clause(DEC["d0531"])}
<p><strong>What happened:</strong> a project needed to size a zone against available flow. The value fit no type: an indicator has no threshold by invariant, yet the constraint must gate on it; a Decision nobody made and a Statement that is merely true were the alternatives.</p>
<p><strong>What changes:</strong> a non-core package declares a design input with value, unit, source, as-of and an optional uncertainty, referenced by constraints through a typed edge. A guard refuses one with no source; a lens lists inputs and their consumers.</p>
{figD1}
{figD3}
<p><strong>True:</strong> the invariant says an indicator has no threshold; the non-core precedent exists; the story is routed. <strong>Mine:</strong> this is the first non-core type that is a fact about the world, not the work. <strong>Decides it:</strong> whether that boundary should be crossed. <strong>Wrong if</strong> a source-less value must still be gated on.</p>
{opts("ask-input", "d0531", [
    ("Accept: a non-core design input type (recommended)", "OPTION A: a non-core schema package declares DesignInput with value, unit, source, asOf and an optional uncertainty, referenced by constraints through a typed edge; the frozen core is untouched (recommended)"),
    ("Indicator plus measurement instead", "OPTION B: use Indicator plus Measurement and document that an indicator a constraint reads is gated after all"),
    ("Do nothing", "OPTION C: nothing changes - the values live in a Decision nobody made or a Statement that is true, and calculations cite neither"),
])}""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page" data-tree="{TREE}">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Take the recommended course on all six</strong>: choose a course on each tab, then <em>Record</em>. Every ask is a fork; nothing changes until you answer.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> {pending}</span><span class="chip"><b>Process changes</b> 1</span><span class="chip"><b>Findings tracked today</b> {N_ISSUES}</span><span class="chip"><b>Stories routed</b> {N_STORIES_ROUTED}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="capture" id="kv"><div class="capture-row"><button class="copy" id="kv-record" type="button" disabled>&#10003; Record on this page</button><button class="copy" id="kv-bind" type="button" disabled>I am wweatherholtz</button><span id="kv-who" class="kv-who"></span></div><p id="kv-status" class="kv-status" role="status">Checking write access&hellip;</p><ol id="kv-list" class="kv-list"></ol></div>
<script type="application/json" id="keel-verdicts">[]</script>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Repository at {TREE}, {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing; guards live ({N_UR_SCANNED} stories, {N_JR_SCANNED} forks); {N_GH} downstream issues; {N_ISSUES} findings today ({_SEVS}); {N_OPEN} open. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept all six: the page is the verdict surface, and downstream gets its five shapes"
SUB = "One process fork carrying its prototype, five architecture forks"


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
# the prototype capture (D0525 OPTION A, the spike the record names): a row of two buttons, a status line, the verdicts so far
CAPTURE_CSS = '''
.capture{margin:14px 0 6px;padding:12px 14px;border:1.5px solid var(--line);border-radius:4px}
.capture-row{display:flex;gap:10px;flex-wrap:wrap;align-items:center}
.capture .copy[disabled]{opacity:.45;cursor:default}
.kv-who{color:var(--muted);font:400 12.5px/1.3 "Roboto Condensed",sans-serif;letter-spacing:.04em}
.kv-status{margin:8px 0 0;color:var(--muted);font-size:13.5px;line-height:1.45;overflow-wrap:anywhere}
.kv-status.ok{color:var(--head)}
.kv-status.bad{color:#b23b2e}
.kv-list{margin:8px 0 0;padding-left:20px;font-size:13.5px;line-height:1.45}
.kv-list li{margin:3px 0}
.kv-list time{color:var(--muted);font:400 12px/1.2 "Roboto Mono",Consolas,monospace}
'''
if ".capture{" not in head:
    head = head.replace("</style>", CAPTURE_CSS + "</style>")
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
# The capture. Records are appended, never replaced, to the JSON block the page carries; the page republishes ITSELF
# (html form: the page's own source read back over the same origin, the block rewritten, the whole document published),
# falling back to the runtime's files form when the source cannot be read from where the viewer stands. Every refusal
# is shown verbatim on the status line - a failure the human cannot see is the st175 defect again. Nothing is written
# on load; the first publish follows a tap.
CAPTURE_JS = r'''
(function(){"use strict";
var $=function(id){return document.getElementById(id)};
var st=$('kv-status'),rec=$('kv-record'),bind=$('kv-bind'),who=$('kv-who'),list=$('kv-list'),blk=$('keel-verdicts');
if(!st||!rec||!bind||!blk)return;
var PERSON='wweatherholtz',art=null,usr=null,me=null;
function say(t,cls){st.textContent=t;st.className='kv-status'+(cls?' '+cls:'')}
function safe(s){return s.replace(/</g,'\\u003c')}
function stored(){try{return JSON.parse(blk.textContent||'[]')}catch(e){return[]}}
function render(){var rs=stored();list.textContent='';rs.forEach(function(r){var li=document.createElement('li');
  var t=document.createElement('time');t.textContent=(r.at||'').replace('T',' ').slice(0,16)+' ';li.appendChild(t);
  var s=r.kind==='binding'?('viewer bound to '+r.person):((r.course||'')+(r.words?' - "'+r.words+'"':''));
  li.appendChild(document.createTextNode(s+(r.kind==='binding'?'':' (on the '+(r.tab||'')+' tab)')));list.appendChild(li)})}
function chosen(){var out=[];Array.prototype.forEach.call(document.querySelectorAll('.opts[data-records]'),function(o){
  var c=o.querySelector('input:checked');if(!c)return;var sec=o.closest('[role="tabpanel"]');var tab=sec&&document.getElementById(sec.getAttribute('aria-labelledby'));
  out.push({kind:'verdict',decision:o.getAttribute('data-records'),course:(c.parentNode.textContent||'').trim(),value:c.value,tab:tab?tab.textContent.trim():''})});return out}
function note(){var n=document.querySelector('[data-d="note"]');return n?n.value.trim():''}
async function publish(recs){rec.disabled=true;bind.disabled=true;say('Writing the page’s next version…');
  var all=stored(),seq=all.length,now=new Date().toISOString(),tree=(document.querySelector('.page')||{}).getAttribute?document.querySelector('.page').getAttribute('data-tree'):'';
  recs.forEach(function(r){r.seq=++seq;r.at=now;r.viewer=me?me.id:null;r.words=r.words===undefined?note():r.words;r.tree=tree;all.push(r)});
  var json=safe(JSON.stringify(all));var src=null,why='';
  try{var resp=await fetch(location.href,{cache:'no-store',credentials:'same-origin'});if(resp.ok)src=await resp.text();else why='HTTP '+resp.status}catch(e){why=String(e&&e.message||e)}
  var re=/(<script type="application\/json" id="keel-verdicts">)([\s\S]*?)(<\/script>)/;
  if(src&&re.test(src)){try{var out=await art.publish(src.replace(re,function(_m,a,_b,c){return a+json+c}));
      say('Recorded as version '+out.version+'. Every open view reloads to it; the keel session that published this page reads it back and records the acceptance citing that version.','ok');return}
    catch(e){if(e&&e.code==='conflict'){say('Someone published a newer version while you were choosing; this view reloads to it - choose again there.','bad');return}
      why='publish refused: '+(e&&e.code||'')+' '+(e&&e.message||e)}}
  else if(src){why=why||'the page source read back from here carries no verdict block'}
  try{await art.publish({'data/verdicts.json':json});blk.textContent=json;render();
    say('Recorded to the page’s data file (the page source could not be republished from here: '+why+'). The session reads the data file back.','ok')}
  catch(e){say('Neither write path worked - '+why+'; then the data file: '+(e&&e.code||'')+' '+(e&&e.message||e)+'. Use Copy for AI; your choices are in the digest.','bad');rec.disabled=false;bind.disabled=false}}
rec.addEventListener('click',function(){var c=chosen();if(!c.length){say('Choose a course on at least one tab first.','bad');return}publish(c)});
bind.addEventListener('click',function(){publish([{kind:'binding',person:PERSON,words:''}])});
render();
(async function(){
  if(!(window.claude&&typeof window.claude.use==='function')){say('This copy of the page is not hosted where it can write itself; use Copy for AI.');return}
  try{art=await window.claude.use('artifact');usr=await window.claude.use('user')}catch(e){}
  if(usr){try{me=await usr.me();who.textContent=me&&me.name?('viewing as '+me.name):'viewer id read'}catch(e){}}
  if(!art){say('You can read this page but not write it from here (no artifact capability was granted); use Copy for AI.');return}
  var bound=stored().some(function(r){return r.kind==='binding'&&me&&r.viewer===me.id});
  rec.disabled=false;bind.disabled=bound;bind.textContent=bound?'Bound as '+PERSON:'I am '+PERSON;
  say(bound?'Choose a course on each tab, then Record. Your viewer id is bound to '+PERSON+'.':'Choose a course on each tab, then Record. Tap “I am '+PERSON+'” once so the session can bind your viewer id to you.')})();
})();
</script>'''
if "keel-verdicts" not in tail:
    assert tail.count("</script>") == 1
    tail = tail.replace("</script>", CAPTURE_JS)
DIGEST_OLD = ".page > h2, .page > h3, .page > p, .page > ul, .page > blockquote, .page > .ask, .page > .turn, .page > .tbl-wrap, .page > .opts, figure.diagram > .msg"
DIGEST_NEW = ".page > .ask, .page > p, [role=\"tabpanel\"] > h2, [role=\"tabpanel\"] > p, [role=\"tabpanel\"] > .tbl-wrap, [role=\"tabpanel\"] > .opts, figure.diagram > .msg"
if DIGEST_OLD in tail:
    tail = tail.replace(DIGEST_OLD, DIGEST_NEW)
assert DIGEST_NEW in tail, "the digest must walk every panel, hidden or not (D0404)"

page = head + body + tail
open(out_path, "w", encoding="utf-8").write(page)
print(f"wrote {out_path}: {len(page)} bytes; frame {frame_words} words; panels " +
      ", ".join(f"{k} {n}" for k, n in sorted(panel_words.items(), key=lambda kv: -kv[1])) +
      f"; summed {words(page)}; {pending} held; findings today {N_ISSUES} ({_SEVS}); stories routed {N_STORIES_ROUTED}; "
      f"guards scanned {N_UR_SCANNED}/{N_JR_SCANNED}; downstream issues {N_GH}; open issues {N_OPEN}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
