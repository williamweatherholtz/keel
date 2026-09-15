#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-15 (thirty-first) queue change: the five marks of the
morning were all answered and one held process change now stands alone in the queue - the recorder's report
check refuses a report that accounts for fewer records than the dispatch owed, or for none. It corrects the
sprint 723 finding (a recorder wrote nothing, said NONE, and the check passed it) and is shipped on the tree,
its own sprint's ceremony recorded under the very control it adds. One tab, one ask (D0404); every count is
a facts.py fact, never typed; each metric says in plain words what it is doing (issue562).

Usage: python scripts/exec_brief/build_2026_09_15_recorder_owed.py <facts.json> <previous.html> <out.html>
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
QUEUE = ["d0492"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != 0 or members[0]["fork"]:
    sys.exit("refusing: the page says the held Decision is not a fork; the lens disagrees")

OC = v("recorderOwedControl")
if OC["status"] != "proposed" or OC["marker"] != "#ProspectiveChange" or OC["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {OC['status']}, {OC['marker']}, {OC['acceptance']}")
if not (OC["namesIssue568"] and OC["namesFirstReportPassed"] and OC["namesReminder"] and OC["namesBeforeAfter"]):
    sys.exit("refusing: the Decision's context must name the Issue, the passed report, the reminder clause and the before/after exits")
if not (OC["notAForkInRationale"] and OC["measuredPhraseInRationale"]):
    sys.exit("refusing: the page says the rationale carries NOT A FORK and a measurement; it does not")
S = OC["source"]
if not (S["owedFlagParsed"] and S["refusalsTakeOwed"] and S["nothingWrittenRefusal"] and S["shortfallRefusal"] and S["refusedLinePattern"]):
    sys.exit(f"refusing: the checker source lacks a piece the page describes: {S}")
if S["pairs"] != 8 or S["pairsUnderOwed"] != 2 or len(S["sprint723Fixtures"]) != 3 or S["fixturesOnDisk"] != S["pairs"] or not S["claudeCopyIdentical"]:
    sys.exit(f"refusing: the page says eight pairs, two under --owed, three from sprint 723, all on disk, .claude identical: {S}")
N_PAIRS, N_POS, N_NEG, N_OWED_PAIRS = S["pairs"], S["positives"], S["negatives"], S["pairsUnderOwed"]
L = OC["live"]
if not (L["probeExit0"] and L["pairsHolding"] == N_PAIRS):
    sys.exit(f"refusing: the live probe does not hold every pair: {L['probeLastLine']}, {L['pairsHolding']} of {N_PAIRS}")
if L["nothingWrittenNoOwed"]["exit"] != 1 or L["nothingWrittenOwed7"]["exit"] != 1 or "zero WROTE" not in L["nothingWrittenNoOwed"]["namedRefusal"]:
    sys.exit(f"refusing: the page says sprint 723's first report is refused with and without --owed: {L['nothingWrittenNoOwed']}, {L['nothingWrittenOwed7']}")
if L["sevenOwed7"]["exit"] != 0 or L["sixOfSevenOwed7"]["exit"] != 1 or "owed 7, accounted 6" not in L["sixOfSevenOwed7"]["namedRefusal"]:
    sys.exit(f"refusing: the page says seven of seven pass and six of seven are refused under --owed 7: {L['sevenOwed7']}, {L['sixOfSevenOwed7']}")
if L["sevenNoOwed"]["exit"] != 0 or L["sixOfSevenNoOwed"]["exit"] != 0:
    sys.exit("refusing: the page says a shortfall is only visible under --owed; without it both seven-line and six-line reports pass")
I = OC["issue568"]
if not (I["exists"] and I["resolverAsExpected"] and I["resolverDodNamesIssue"]):
    sys.exit(f"refusing: the Issue, its resolver edge or the resolver's DoD is missing: {I}")
SP = OC["sprint724"]
if not (SP["exists"] and SP["chartersAsExpected"]):
    sys.exit(f"refusing: sprint 724 must exist and be chartered by the held Decision: {SP}")
N_RESULTS, N_GATE_RESULTS = SP["results"], SP["gateResults"]
SU = OC["surfaces"]
if not all(SU[k] for k in ("briefRuleSix", "briefPassesOwed", "briefListsSevenRefusals", "briefClaudeIdentical", "processDispatchNamesCount", "processReportNamesRefusals")):
    sys.exit(f"refusing: a surface the page names does not carry the change: {SU}")
POS = OC["resolverPosition"]
N_ITEMS = OC["engineBuildItems"]
if POS is None:
    sys.exit("refusing: the resolver is not on the engine backlog")

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
figO1 = logic_lanes(
    "The check catches only what is on the page, so an empty report gets through",
    ("what happened", [
        ("seven owed", "listed in the brief", "accent", ""),
        ("none written", "reported NONE", "bad", ""),
        ("check passed it", "five refusals, vacuous", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("same dispatch", "--owed 7", "accent", ""),
        ("nothing accounted", "refused, any count", "ok", ""),
        ("six of seven", "refused, shortfall named", "ok", ""),
    ], ["", "", ""]),
)
figO2 = bars(
    f"The check now runs {N_PAIRS} known cases, {N_OWED_PAIRS} of them under an owed count",
    [
        ("known-positive, must be refused", N_POS, "warn", ""),
        ("known-negative, must pass", N_NEG, "ok", ""),
        ("run under --owed", N_OWED_PAIRS, "accent", ""),
        ("from sprint 723's own reports", len(S["sprint723Fixtures"]), "accent", ""),
    ],
    unit=("case", "cases"),
)
figO3 = downstream(
    "One clause lands on four surfaces",
    ("the report check", "delegated-ceremony skill", ""),
    [
        ("recorder brief", "rule six passes the count", "1", "ok"),
        ("process steps", "dispatch and report name it", f"{SU['processNamesD0492']}", "ok"),
        ("Issue resolved", f"severity {I['severity']}", "1", "ok"),
        ("its own ceremony results", "recorded under this check", f"{N_RESULTS}", "muted" if N_RESULTS == 0 else "ok"),
    ],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0492", "ask-owed", "Owed count", "owed"),
]
panel_bodies = {
    "owed": f"""<h2>The report must account for every record sent</h2>
{clause(OC)}
<p><strong>What happened:</strong> a recorder sent seven records wrote none, returned <q>DISCREPANCIES: NONE</q>, and the check passed it: its five refusals read only the lines present. The owed list was text the check never saw.</p>
<p><strong>What changes:</strong> the dispatch states the count; the check refuses fewer accounted lines than owed (<q>{escape(L['sixOfSevenOwed7']['namedRefusal'].split(' (')[0])}</q>) and any report with no write and no refusal line.</p>
{figO1}
{figO2}
{figO3}
{courses([
    ("Accept (recommended)", "a shortfall or an empty report is refused", "a wrong typed count is still trusted"),
    ("Guard the sprint file instead", "a commit-time guard over a resultless gate", "fires after the report was trusted"),
    ("Do nothing", "nothing", "an empty report passes again"),
])}
<p><strong>True:</strong> {N_PAIRS} cases hold live; the Issue names this as its resolver. <strong>Mine:</strong> refuse at the report, not the commit. <strong>What decides it:</strong> whether a count I type at dispatch is a control by your standard, or only a computed owed set is. <strong>Wrong if</strong> the next fault is a wrong count, not a missing write.</p>
{opts("ask-owed", "d0492", [
    ("Accept (recommended)", "Accept: check_report.py takes --owed N and refuses a recorder report whose WROTE plus REFUSED lines number fewer than N, and with or without --owed refuses a report with zero WROTE lines and no REFUSED line; the recorder brief passes the count (recommended)"),
    ("Guard the sprint file instead", "Reject the report-side check in favour of a commit-time guard over a ceremony gate with no result"),
    ("Do nothing", "Do nothing: withdraw the two refusals; the Issue stays open"),
])}""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Accept the owed-count check</strong>: a recorder's report is refused when it accounts for fewer records than it was sent, or for none. Shipped and probed; held because it changes the ceremony process.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> 0</span><span class="chip"><b>Process change</b> 1</span><span class="chip"><b>Known cases held</b> {N_PAIRS}</span><span class="chip"><b>Issue it closes</b> 1</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing. Clauses quoted; record names glossed. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept the owed-count check on the recorder's report"
SUB = "One held process change: the correction for a recorder that wrote nothing and said NONE"


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
      f"; summed {words(page)}; {pending} held; {N_PAIRS} pairs hold; resolver {POS} of {N_ITEMS}; sprint results {N_RESULTS}+{N_GATE_RESULTS}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
