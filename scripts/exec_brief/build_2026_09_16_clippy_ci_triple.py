#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-16 (thirty-third) queue change: the held stems row is joined by
a second held process change - every clippy the engine runs before a push lints the triple CI lints, a second time
with --target when the host is another triple, and a host without that std is red naming the remedy, never skipped.
Two tabs, two asks (D0404); every count is a facts.py fact, never typed; each metric says in plain words what it is
doing (issue562).

Usage: python scripts/exec_brief/build_2026_09_16_clippy_ci_triple.py <facts.json> <previous.html> <out.html>
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


# ---- the queue: two held process changes -------------------------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
QUEUE = ["d0494", "d0495"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != 0 or any(m["fork"] for m in members):
    sys.exit("refusing: the page says neither held Decision is a fork; the lens disagrees")

# -- ask 1: the stems row (unchanged from the thirty-second page; re-checked against the tree as it is now) --
VS = v("verifierStemsRow")
if VS["status"] != "proposed" or VS["marker"] != "#ProspectiveChange" or VS["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {VS['status']}, {VS['marker']}, {VS['acceptance']}")
if not (VS["namesIssue530"] and VS["namesEmbeddedStem"] and VS["namesSprint725Mismatch"] and VS["namesReceiptHalfRemains"] and VS["saysProcessChange"]):
    sys.exit("refusing: the stems Decision must name the Issue, the binary's rule, the sprint that hit it, the half that remains and that it is a process change")
K = VS["skill"]
if not (K["rowFound"] and K["rowNamesInit"] and K["rowNamesEngineTree"] and K["rowNamesEmbeddedStem"] and K["rowNamesIssue530"] and K["rowNamesGitStatus"]):
    sys.exit(f"refusing: the corrected row lacks a piece the page describes: {K}")
if K["stemsRowsInSkill"] != 1 or not K["claudeCopyIdentical"]:
    sys.exit(f"refusing: the page says one stems row and a byte-identical .claude copy: {K}")
B = VS["binary"]
if not (B["embeddedStemFound"] and B["stripsEnginePrefix"] and B["returnsInit"] and B["bareTreeIsNone"] and B["ownTestFound"]):
    sys.exit(f"refusing: the binary's rule is not as the page describes it: {B}")
L = VS["live"]
if L["ownTest"]["exit"] != 0 or L["ownTest"]["passed"] != 1 or L["ownTest"]["failed"] != 0:
    sys.exit(f"refusing: the binary's own test for the rule did not pass live: {L['ownTest']}")
LR = L["landingReceipt"]
if not (LR["exists"] and LR["outcome"] == "pass" and LR["stemsIncludeInit"] and LR["failed"] == "0"):
    sys.exit(f"refusing: the page says the latest landing run attributed init and passed: {LR}")
ENGINE_PATHS = L["landingCommitEnginePaths"]
if not ENGINE_PATHS:
    sys.exit("refusing: the page says the landed commit changed paths under .engine; git shows none")
N_TESTS_LANDED = int(LR["passed"])
N_STEMS = len(LR["stems"])
N_SOURCE_STEMS = len([s for s in LR["stems"] if s != "init"])
I530 = VS["issue530"]
if not (I530["exists"] and I530["resolverAsExpected"] and I530["resolverDodNamesIssue"]):
    sys.exit(f"refusing: the stems Issue, its resolver edge or the resolver's DoD is missing: {I530}")
OB = VS["obligations"]
if not all(o["exists"] and o["resolvesFromD0494"] and o["namedInConsequences"] for o in OB.values()):
    sys.exit(f"refusing: the page says both yielded-red obligations are triaged by the stems Decision: {OB}")
N_OBLIGATIONS = len(OB)
POS94, N_NEXT = VS["resolverPosition"], VS["nextWorkItems"]
if POS94 is None:
    sys.exit("refusing: the receipt half's resolver is not on the backlog")

# -- ask 2: clippy lints the triple CI lints --
CT = v("clippyLintsCiTriple")
if CT["status"] != "proposed" or CT["marker"] != "#ProspectiveChange" or CT["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {CT['status']}, {CT['marker']}, {CT['acceptance']}")
if not (CT["namesIssue572"] and CT["namesTouchedLine"] and CT["namesSecondInstance"] and CT["namesD0098"] and CT["namesRemedy"] and CT["saysProcessChange"]):
    sys.exit("refusing: the clippy Decision must name the Issue, the failing line, that it is the second instance, the never-skip rule, the remedy and that it is a process change")
CB = CT["binary"]
if not all(CB[k] for k in ("constFound", "controlFound", "controlOnceForCi", "controlAgainWhenInstalled", "controlRefusesWithRemedy", "positiveTestFound", "negativeTestFound", "rungRunsWorkspace")):
    sys.exit(f"refusing: the pure control, its three arms or its pair are not in the binary as the page describes: {CB}")
CH = CT["hook"]
if not all(CH.values()):
    sys.exit(f"refusing: the hook's second clippy, host read, target test or refusal branch is missing: {CH}")
CK = CT["skill"]
if not all(CK.values()):
    sys.exit(f"refusing: the skill sentence or its .claude copy is not as the page describes: {CK}")
HOST = CT["host"]
if HOST["isCiTriple"] or not HOST["ciStdInstalled"]:
    sys.exit(f"refusing: the page describes a non-CI host with the CI std installed, which is how both commands ran here: {HOST}")
CR = CT["live"]["clippyRung"]
if not (CR and CR["verdict"] == "pass" and CR["exit"] == 0 and CR["namesBothTriples"]):
    sys.exit(f"refusing: the page says the receipt's clippy rung names both commands and passed: {CR}")
VR = CT["live"]["verifierReceipt"]
if not (VR["exists"] and VR["outcome"] == "pass" and VR.get("stoppedAt") == "none"):
    sys.exit(f"refusing: the page says the sprint's ladder is green end to end: {VR}")
RUNG_S = CT["live"]["rungSeconds"]
CLIPPY_S, LADDER_S = CR["seconds"], sum(RUNG_S.values())
N_RUNGS = VR["rungsGreen"]
ISS = CT["issues"]
for n in ("issue572", "issue573", "issue574", "issue575"):
    if not (ISS[n]["exists"] and ISS[n]["resolverAsExpected"] and ISS[n]["resolverDodNamesIssue"]):
        sys.exit(f"refusing: {n}, its resolver edge or the resolver's DoD is missing: {ISS[n]}")
CLASS = CT["classMembers"]
N_CLASS = len(CLASS)
SP = CT["sprint726"]
if not (SP["exists"] and SP["chartersAsExpected"]):
    sys.exit(f"refusing: sprint 726 must exist and be chartered by the clippy Decision: {SP}")
N_RESULTS, N_GATE_RESULTS = SP["results"], SP["gateResults"]
RP = CT["resolverPositions"]
if any(RP[a] is None for a in RP):
    sys.exit(f"refusing: a resolver the page places on the backlog is not there: {RP}")
POS_PROBES, POS_WAIT, N_BACKLOG = RP["dcScriptProbesRunBeforeCommit"]["place"], RP["dcVerifyWaitsForItsPid"]["place"], CT["backlogItems"]
CI = CT["ci"] or []
CI_PUSH = [c for c in CI if c["event"] == "push"]
N_CI_RED = len([c for c in CI_PUSH if c["conclusion"] == "failure"])

tests, failing = v("suiteTests"), v("suiteFailed")
dirty = v("treeUncommitted")["modified"]


def courses(rows):
    body = "".join(f"<tr><td>{a}</td><td>{b}</td><td>{c}</td></tr>" for a, b, c in rows)
    return (f'<div class="tbl-wrap"><table><thead><tr><th>course</th><th>what changes</th><th>what it costs</th></tr></thead>'
            f'<tbody>{body}</tbody></table></div>')


ID_RE = re.compile(r"\b(?:[Dd]0\d{3}|issue\d{3}|st\d{3}|us\d{3}|s{1,3}r[A-Z]\w+)\b")


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


# ---- exhibits: two per ask; every title is a claim with a verb -----------------------------------------
figS1 = logic_lanes(
    "A stale row makes a green tree read as a discrepancy",
    ("what happened", [
        ("engine file changed", "binary adds init", "accent", ""),
        ("row: source only", "init unexplained", "bad", ""),
        ("MISMATCH", f"{N_TESTS_LANDED} tests green", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("engine file changed", "binary adds init", "accent", ""),
        ("row: init too", "names the rule's home", "ok", ""),
        ("MATCH", "read, not re-derived", "ok", ""),
    ], ["", "", ""]),
)
figS2 = bars(
    f"The latest landing run attributed {N_STEMS} stems, one from the embedded tree",
    [
        ("from changed source modules", N_SOURCE_STEMS, "ok", ""),
        ("from the embedded tree", N_STEMS - N_SOURCE_STEMS, "accent", ""),
        ("engine files behind that stem", len(ENGINE_PATHS), "accent", ""),
        ("tests green under it", N_TESTS_LANDED, "ok", ""),
    ],
    unit=("item", "items"),
)
figS3 = downstream(
    "One row lands on three surfaces",
    ("the stems row", "test-verify skill", ""),
    [
        ("verifier over an engine change", "writes MATCH", "1", "ok"),
        ("yielded-red obligations", "triaged", f"{N_OBLIGATIONS}", "ok"),
        ("the Issue's procedure half", f"severity {I530['severity']}", "1", "ok"),
    ],
)

figC1 = logic_lanes(
    "The host's lint cannot see the code CI compiles",
    ("what happened", [
        ("host clippy", "skips the other body", "accent", ""),
        ("local gates", "green", "ok", ""),
        ("CI on Linux", "red", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("host clippy", "then the CI triple", "accent", ""),
        ("std missing", "red, remedy named", "bad", ""),
        ("std present", f"green in {CLIPPY_S} s", "ok", ""),
    ], ["", "", ""]),
)
figC2 = bars(
    f"The second lint is {CLIPPY_S} s of a {LADDER_S} s ladder",
    [(name, secs, "accent" if name == "clippy" else "ok", "") for name, secs in RUNG_S.items()],
    unit=("second", "seconds"),
)
figC3 = downstream(
    "One rule lands on three surfaces",
    ("lint the CI triple", "verify rung + hook", ""),
    [
        ("the clippy rung", "both commands in its row", "1", "ok"),
        ("the pre-commit hook", "second clippy or refusal", "1", "ok"),
        ("the verifier's skill", "names the second command", "1", "ok"),
        ("Issues in the class", f"third's runner {POS_PROBES} of {N_BACKLOG}", f"{N_CLASS}", "accent"),
    ],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0495", "ask-triple", "CI-triple lint", "triple"),
    ("d0494", "ask-stems", "Stems row", "stems"),
]
panel_bodies = {
    "triple": f"""<h2>The lint before a push must see what CI compiles</h2>
{clause(CT)}
<p><strong>What happened:</strong> a delivery landed green through every local clippy; CI's Linux clippy failed it on a body the Windows host never compiles.</p>
<p><strong>What changes:</strong> the clippy rung and the pre-commit hook lint once for the host and again for the Linux triple; a host without that std is red naming the remedy, never skipped.</p>
{figC1}
{figC2}
{figC3}
{courses([
    ("Accept (recommended)", "both lints before every push; a missing std refuses", f"{CLIPPY_S} s more per ladder off Linux"),
    ("Pin the toolchain instead", "one rust version everywhere; that body still unlinted", "a second brief"),
    ("Do nothing", "nothing", "CI stays the first lint of that code"),
])}
<p><strong>True:</strong> the clippy row names both commands and passed; {N_RUNGS} rungs green; {N_CLASS} Issues in the class, each with a resolver. <strong>Mine:</strong> CI's rust version is the remaining gap. <strong>What decides it:</strong> may a check that cannot run pass silently, or must it refuse and name its remedy. <strong>Wrong if</strong> the next CI-only red is a lint the version gap adds.</p>
{opts("ask-triple", "d0495", [
    ("Accept (recommended)", "Accept: every clippy the engine runs before a push lints the triple CI lints - a second run with --target x86_64-unknown-linux-gnu when the host is another triple, and a host without that std is red naming rustup target add, never skipped (recommended)"),
    ("Pin the toolchain instead", "Reject the second lint: bring a toolchain-pin proposal instead and leave the other host's cfg bodies to CI"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the rung and hook stay as committed"),
])}""",
    "stems": f"""<h2>The verifier's rule must be the binary's rule</h2>
{clause(VS)}
<p><strong>What happened:</strong> the row said stems equal the changed source modules; the binary also adds <q>init</q> for a changed engine file. A verifier read a green receipt and wrote MISMATCH.</p>
<p><strong>What changes:</strong> the row adds <q>plus init when any path under .engine/ is changed or untracked</q> and names the function behind it.</p>
{figS1}
{figS2}
{figS3}
{courses([
    ("Accept (recommended)", "the row describes the binary", "a reader still re-derives the rule"),
    ("Revert the row and wait", "the row waits for a receipt that states its attribution", "MISMATCH on every green engine change"),
    ("Do nothing", "nothing", "the row stays as committed; the clause stays open"),
])}
<p><strong>True:</strong> the binary's own test passes live; the latest landing took <q>init</q> from {len(ENGINE_PATHS)} engine files, {N_TESTS_LANDED} tests green. <strong>Mine:</strong> fix the procedure now; the receipt half sits {POS94} of {N_NEXT} in next-work. <strong>What decides it:</strong> may a verifier follow a row naming a function, or only a receipt stating its own attribution. <strong>Wrong if</strong> the next false MISMATCH is a stem the row omits.</p>
{opts("ask-stems", "d0494", [
    ("Accept (recommended)", "Accept: the test-verify skill's stems row expects init in the touched receipt's stems whenever a path under .engine/ is changed or untracked, naming embedded_stem in touched.rs as the rule's home (recommended)"),
    ("Revert the row and wait", "Reject the row correction: revert the stems row and wait for the receipt to state its own attribution before the skill changes"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the row stays as committed"),
])}""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Accept both</strong>: the lint before a push sees what CI compiles and refuses when it cannot; the verifier's stems row says what the binary does.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> 0</span><span class="chip"><b>Second lint, seconds</b> {CLIPPY_S}</span><span class="chip"><b>CI-only reds, the class</b> {N_CLASS}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing; CI red on {N_CI_RED} of the last {len(CI_PUSH)} pushes. Clauses quoted; record names glossed. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept the CI-triple lint and the stems row"
SUB = "Two held process changes, neither a fork; both committed under the process lock"


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
      f"; summed {words(page)}; {pending} held; clippy {CLIPPY_S} s of {LADDER_S} s; {N_RUNGS} rungs; class {N_CLASS}; "
      f"probes runner {POS_PROBES}, wait {POS_WAIT} of {N_BACKLOG}; {N_TESTS_LANDED} tests under the row; sprint results {N_RESULTS}+{N_GATE_RESULTS}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
