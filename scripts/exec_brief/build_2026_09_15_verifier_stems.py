#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-15 (thirty-second) queue change: the owed-count check was
answered and one held process change now stands alone in the queue - the verifier's stems row says what the
binary does, so a changed path under .engine contributes `init` and a correct receipt over such a change reads
MATCH, not MISMATCH. One table row in a locked skill, already committed under the process lock beside the
Decision that authorises it; held because a skill is a process definition. One tab, one ask (D0404); every
count is a facts.py fact, never typed; each metric says in plain words what it is doing (issue562).

Usage: python scripts/exec_brief/build_2026_09_15_verifier_stems.py <facts.json> <previous.html> <out.html>
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


# ---- the queue: one held process change --------------------------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
QUEUE = ["d0494"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != 0 or members[0]["fork"]:
    sys.exit("refusing: the page says the held Decision is not a fork; the lens disagrees")

VS = v("verifierStemsRow")
if VS["status"] != "proposed" or VS["marker"] != "#ProspectiveChange" or VS["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {VS['status']}, {VS['marker']}, {VS['acceptance']}")
if not (VS["namesIssue530"] and VS["namesEmbeddedStem"] and VS["namesSprint725Mismatch"] and VS["namesReceiptHalfRemains"] and VS["saysProcessChange"]):
    sys.exit("refusing: the Decision must name the Issue, the binary's rule, the sprint that hit it, the half that remains and that it is a process change")
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
VR, LR = L["verifierReceipt"], L["landingReceipt"]
if not (VR["exists"] and VR["outcome"] == "pass" and VR.get("stoppedAt") == "none" and VR.get("rungsGreen") == len(VR.get("rungs") or {})):
    sys.exit(f"refusing: the page says the sprint's verifier ladder is green end to end: {VR}")
if not (LR["exists"] and LR["outcome"] == "pass" and LR["stemsIncludeInit"] and LR["failed"] == "0"):
    sys.exit(f"refusing: the page says the landing run attributed init and passed: {LR}")
ENGINE_PATHS = L["landingCommitEnginePaths"]
if not ENGINE_PATHS:
    sys.exit("refusing: the page says the landed commit changed paths under .engine; git shows none")
N_RUNGS, N_TESTS_LANDED = VR["rungsGreen"], int(LR["passed"])
N_STEMS = len(LR["stems"])
N_SOURCE_STEMS = len([s for s in LR["stems"] if s != "init"])
I = VS["issue530"]
if not (I["exists"] and I["resolverAsExpected"] and I["resolverDodNamesIssue"] and I["namesSkillTable"] and I["namesMismatch"]):
    sys.exit(f"refusing: the Issue, its resolver edge or the resolver's DoD is missing: {I}")
OB = VS["obligations"]
if not all(o["exists"] and o["resolvesFromD0494"] and o["namesLockedFile"] and o["namesProcessChangeGuard"] and o["namedInConsequences"] for o in OB.values()):
    sys.exit(f"refusing: the page says both yielded-red obligations are triaged by this Decision and named in it: {OB}")
N_OBLIGATIONS = len(OB)
SP = VS["sprint725"]
if not (SP["exists"] and SP["chartersAsExpected"]):
    sys.exit(f"refusing: sprint 725 must exist and be chartered by the exclusivity Decision: {SP}")
N_RESULTS, N_GATE_RESULTS = SP["results"], SP["gateResults"]
POS, N_ITEMS = VS["resolverPosition"], VS["nextWorkItems"]
if POS is None:
    sys.exit("refusing: the receipt half's resolver is not on the backlog")

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
    f"The landing run attributed {N_STEMS} stems, one from the embedded tree",
    [
        ("from changed source modules", N_SOURCE_STEMS, "ok", ""),
        ("from the embedded tree", N_STEMS - N_SOURCE_STEMS, "accent", ""),
        ("engine files behind that stem", len(ENGINE_PATHS), "accent", ""),
        ("verifier ladder rungs green", N_RUNGS, "ok", ""),
    ],
    unit=("item", "items"),
)
figS3 = downstream(
    "One row lands on four surfaces",
    ("the stems row", "test-verify skill", ""),
    [
        ("verifier over an engine change", "writes MATCH", "1", "ok"),
        ("yielded-red obligations", "triaged", f"{N_OBLIGATIONS}", "ok"),
        ("the Issue's procedure half", f"severity {I['severity']}; receipt half open", "1", "ok"),
        ("this sprint's results", "re-read, not re-run", f"{N_RESULTS}", "ok"),
    ],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0494", "ask-stems", "Stems row", "stems"),
]
panel_bodies = {
    "stems": f"""<h2>The verifier's rule must be the binary's rule</h2>
{clause(VS)}
<p><strong>What happened:</strong> the row said stems equal the changed source modules; the binary also adds <q>init</q> for a changed engine file. A verifier read a green receipt and wrote MISMATCH, again.</p>
<p><strong>What changes:</strong> the row adds <q>plus init when any path under .engine/ is changed or untracked</q> and names the function behind it.</p>
{figS1}
{figS2}
{figS3}
{courses([
    ("Accept (recommended)", "the row describes the binary", "a reader still re-derives the rule"),
    ("Revert the row and wait", "the row waits for a receipt that states its attribution", "MISMATCH on every green engine change"),
    ("Do nothing", "nothing", "the row stays as committed; the clause stays open"),
])}
<p><strong>True:</strong> the binary's own test passes live; the landing run took <q>init</q> from {len(ENGINE_PATHS)} engine files. <strong>Mine:</strong> fix the procedure now; the receipt half sits {POS} of {N_ITEMS} on the backlog. <strong>What decides it:</strong> may a verifier follow a row naming a function, or only a receipt stating its own attribution. <strong>Wrong if</strong> the next false MISMATCH is a stem the row omits.</p>
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

<div class="ask"><p class="verdict"><strong>Accept the corrected stems row</strong>: the verifier expects <q>init</q> whenever a path under .engine changed, as the binary already attributes it. One row in a locked skill; held because a skill is a process definition.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> 0</span><span class="chip"><b>Process change</b> 1</span><span class="chip"><b>Tests green under the row</b> {N_TESTS_LANDED}</span><span class="chip"><b>Obligations it triages</b> {N_OBLIGATIONS}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing. Clauses quoted; record names glossed. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept the verifier's stems row"
SUB = "One held process change: the row a verifier reads a receipt against now says what the binary does"


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
      f"; summed {words(page)}; {pending} held; {N_TESTS_LANDED} tests under the row; {N_RUNGS} rungs; receipt half {POS} of {N_ITEMS}; sprint results {N_RESULTS}+{N_GATE_RESULTS}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
