#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-14 (twenty-ninth) queue change: fourteen held Decisions
left the page on the human's words; one carries over - the weighted recall rule picks a different setting at the
bench's two sample sizes (D0466) - read it at 50, at 100, or require both. New on the page is a human obligation
the authority-queue lens does not enumerate (issue561): the Business gate of the modular-members work, eight
Needs derived from the human's own statement (st126) that wait for their word before any architecture is
decided. They are read back in three tabs, by story, each quoting the Needs verbatim and asking what is WIDER
than they meant (beConfirmNarrower). One tab per ask (D0404); every count is a facts.py fact, never typed.

Usage: python scripts/exec_brief/build_2026_09_14_sample_fork_and_needs.py <facts.json> <previous.html> <out.html>
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


# ---- the queue: one fork, and a gate the lens does not list -------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
if len(members) != pending or v("pendingForks") != 1:
    sys.exit(f"refusing: facts disagree - {len(members)} members, {pending} pending, {v('pendingForks')} forks")
if [p["slug"] for p in members] != ["d0466"] or not members[0]["fork"]:
    sys.exit(f"refusing: this page is written for the sample fork alone in the queue; the queue is {[p['slug'] for p in members]}")
AQ = v("authorityQueueKinds")
if AQ["listsConfirmationGates"]:
    sys.exit("refusing: the page says the lens omits confirmation gates and carries the Needs ask by hand; the lens now lists them")
MM = v("modularMembersNeeds")
NEEDS = {n["name"]: n for n in MM["needs"]}
STORIES = {s["name"]: s for s in MM["stories"]}
GATE = MM["gate"]
if GATE["method"] != "confirmation" or GATE["hasResult"]:
    sys.exit(f"refusing: the Business gate must be a confirmation with no result; it reads {GATE}")
TABS_NEEDS = {
    "members": ["nMemberRebuildIsLocal", "nMemberDependsDownOnly", "nEveryModuleHasAMemberHome"],
    "issues": ["nIssuesSubsetIsItsOwnMember", "nStateIsComputedAndExposed", "nMemberIsAnAnchor"],
    "process": ["nProcessIsReadNotHardcoded", "nCustomProcessRefusesEverywhere"],
}
if sorted(sum(TABS_NEEDS.values(), [])) != sorted(NEEDS):
    sys.exit(f"refusing: the three tabs must carry every Need once; the record holds {sorted(NEEDS)}")
N_NEEDS = len(NEEDS)
for n in NEEDS.values():
    if not (n["title"] and n["statement"] and n["priority"] and n["derivedFrom"]):
        sys.exit(f"refusing: Need {n['name']} lacks a title, statement, priority or story edge")
if sorted(STORIES) != ["us111", "us112", "us113", "us114"]:
    sys.exit(f"refusing: the four stories of the statement are expected; the record holds {sorted(STORIES)}")
ST = MM["statement"]
if ST["saidBy"] != "wweatherholtz" or "keel-issues" not in ST["text"] or "absolute discipline" not in ST["text"]:
    sys.exit("refusing: the page quotes spans of the statement the record does not carry")

tests, failing = v("suiteTests"), v("suiteFailed")
dirty = v("treeUncommitted")["modified"]

# ---- the workspace the member Needs are read against -----------------------------------------------------
WG = v("workspaceGraph")
N_MEMBERS, N_LEAVES = len(WG["members"]), len(WG["leafMembers"])
N_EDGES, N_LEAF_EDGES = len(WG["memberEdges"]), len(WG["edgesAmongLeaves"])
N_CLI_DEPS = len(WG["keelCliDependsOn"])
if N_CLI_DEPS != N_MEMBERS - 1:
    sys.exit(f"refusing: the page says the binary depends on every other member; it depends on {N_CLI_DEPS} of {N_MEMBERS - 1}")
if WG["layersContractExists"] or WG["guardInNames"]:
    sys.exit("refusing: the page says the layering is not yet a guard; the facts say it is")
CF = v("cliFamilyCensus")
FAMILIES = [(f["family"], f["count"]) for f in CF["families"]]
TOP = CF["topLevel"]
if sum(c for _f, c in FAMILIES) != TOP:
    sys.exit(f"refusing: the family census does not sum to its top level: {FAMILIES} vs {TOP}")
FG = v("familyCensusGating")
HOOKS_CI = FG["sites"]["hooksAndCi"]
GATE_SITES = sum(HOOKS_CI.values())

# ---- the sample fork (D0466): two sweeps, the rule read at each, the hand set, the constant ---------------
SS = v("sampleSplit")
S50, S100, HAND = SS["at50"], SS["at100"], SS["handSet"]
OFF, LOW, MID = "0", "1.1", "1.25"
if f"{SS['constant']:g}" != MID:
    sys.exit(f"refusing: the page says {MID} is in force; the source reads {SS['constant']}")
W50, W100 = S50["rule"]["winner"], S100["rule"]["winner"]
if (W50, W100) != (LOW, MID):
    sys.exit(f"refusing: the Decision says the rule picks {LOW} at 50 and {MID} at 100; the facts pick {W50} and {W100}")
if LOW not in S100["rule"]["refused"] or LOW not in S50["rule"]["admissible"]:
    sys.exit("refusing: the Decision says 1.1 is admissible at 50 and refused at 100; the facts disagree")
R50, R100 = S50["rows"], S100["rows"]
N50, N100 = R50[OFF]["hop1"]["n"], R100[OFF]["hop1"]["n"]
if (N50, N100) != (50, 100):
    sys.exit(f"refusing: the two samples are not 50 and 100 cases: {N50}, {N100}")
if not (HAND[MID]["bar"] == "MET" and HAND[LOW]["bar"] == "MET" and HAND[OFF]["bar"] == "NOT MET"):
    sys.exit(f"refusing: the page says the bar is met at {LOW} and {MID} and not at off; the readings are {HAND}")
med50_off, med50_low = R50[OFF]["hop1"]["median"], R50[LOW]["hop1"]["median"]
med100_off, med100_low, med100_mid = R100[OFF]["hop1"]["median"], R100[LOW]["hop1"]["median"], R100[MID]["hop1"]["median"]
gain50_low = S50["rule"]["readings"][LOW]["twoHopGain"]
gain100_mid = S100["rule"]["readings"][MID]["twoHopGain"]
move100_low = S100["rule"]["readings"][LOW]["medianMove"]
top3_50_lost = R50[OFF]["hop1"]["top3"] - R50[LOW]["hop1"]["top3"]
top3_100_lost = R100[OFF]["hop1"]["top3"] - R100[LOW]["hop1"]["top3"]
SET50 = sorted(R50, key=float)
SET100 = sorted(R100, key=float)


def courses(rows):
    body = "".join(f"<tr><td>{a}</td><td>{b}</td><td>{c}</td></tr>" for a, b, c in rows)
    return (f'<div class="tbl-wrap"><table><thead><tr><th>course</th><th>what changes</th><th>what it leaves</th></tr></thead>'
            f'<tbody>{body}</tbody></table></div>')


# A record id inside a quoted statement is rendered as the thing it names (the contract keeps ids out of the
# reader's text); the record keeps the id, the digest keeps the Need's name, and the page says the words were glossed.
GLOSS = {"(keel-issues, st126's name)": "(keel-issues, your name for it)", " (keel show control-census)": "", "st126": "your statement", "(D0479 layering)": "(the declared layering)", "(cargo build -v)": "(the build's own report)",
         "(the existing D0018 invariant, held for the member)": "(the standing text-is-truth invariant, held for the member)",
         "(keel show control-census)": "(the control census)"}
ID_RE = re.compile(r"\b(?:[Dd]0\d{3}|issue\d{3}|st\d{3}|us\d{3})\b")


def glossed(text):
    for k, g in GLOSS.items():
        text = text.replace(k, g)
    if ID_RE.search(text):
        sys.exit(f"refusing: a record id survives the gloss in a quoted statement: {text[:80]}")
    return text


def needs_table(names):
    """The Needs quoted from the record: title, priority, and the statement with its threshold and whose judgment it is."""
    body = "".join(
        f'<tr><td><span class="m">{escape(NEEDS[n]["priority"])}</span></td><td><b>{escape(NEEDS[n]["title"])}</b><br>'
        f'<q>{escape(glossed(NEEDS[n]["statement"]))}</q></td></tr>' for n in names)
    return (f'<div class="tbl-wrap"><table data-members="{len(names)}"><thead><tr><th>priority</th><th>the Need, verbatim</th></tr></thead>'
            f'<tbody>{body}</tbody></table></div>')


def story_quote(name):
    s = STORIES[name]
    return f'<p><strong>Your story</strong> ({escape(s["asA"])}): <q>{escape(s["iWant"])}</q></p>'


def needs_opts(radio, names):
    k = len(names)
    return (f'<div class="opts" data-records="{GATE["name"]}">'
            f'<label><input type="radio" name="{radio}" value="Accept as written: the {k} Needs on this tab ({", ".join(names)}) say what I meant; record my confirmation of the Business gate for them">Accept as written</label>'
            f'<label><input type="radio" name="{radio}" value="Narrow: one or more of these {k} Needs ({", ".join(names)}) is wider than I meant - I say which and how in the note">Narrow</label>'
            f'<label><input type="radio" name="{radio}" value="Not measured right: a threshold marked as the AI\'s judgment on this tab is wrong - I give the number or the observable in the note">Fix a threshold</label>'
            f'<label><input type="radio" name="{radio}" value="Hold: these {k} Needs wait; no architecture Decision is derived from them yet">Hold</label></div>')


# ---- exhibits: two per ask, the fork carries three -----------------------------------------------------
fig5 = logic_lanes(
    f"The same rule lands on {W50} at {N50} cases and on {W100} at {N100}",
    (f"read at {N50}", [
        (f"{LOW} moves the median one", "admissible", "accent", ""),
        (f"rule: {W50}", "", "warn", ""),
    ], ["", ""]),
    (f"read at {N100}", [
        (f"{LOW} moves it {move100_low}", "refused", "accent", ""),
        (f"rule: {W100}", "", "ok", ""),
    ], ["", ""]),
)
fig6 = bars(
    f"At {N50} cases {LOW} has the most far hits inside the rule",
    [(s, R50[s]["hop2"]["hits"], "ok" if s in S50["rule"]["admissible"] else "warn", "") for s in SET50],
    unit="",
)
fig7 = bars(
    f"At {N100} cases {LOW} is refused and {MID} has the most",
    [(s, R100[s]["hop2"]["hits"], "ok" if s in S100["rule"]["admissible"] else "warn", "") for s in SET100],
    unit="",
)
figM1 = logic_lanes(
    f"Every edit relinks the binary today: it has all {N_CLI_DEPS} other members below it",
    ("today", [
        ("one-line edit in a leaf", "", "accent", ""),
        ("the leaf, then the binary", "everything above relinks", "bad", ""),
    ], ["", ""]),
    ("as the Needs read", [
        ("the same edit", "", "accent", ""),
        ("that member and those above it", "the rest reads Fresh", "ok", ""),
    ], ["", ""]),
)
figM2 = downstream(
    f"The Needs land on a workspace of {N_MEMBERS} members and {N_EDGES} declared dependencies",
    ("read the manifests", "as they are", ""),
    [
        ("leaf members", "each a rebuild unit", f"{N_LEAVES}", "ok"),
        ("dependencies among leaves", "must point down", f"{N_LEAF_EDGES}", "warn"),
        ("layering guard", "none yet", "0", "bad"),
    ],
)
figI1 = logic_lanes(
    f"A project that only tracks items gets all {TOP} verbs today",
    ("today", [
        ("a project tracks items", "", "accent", ""),
        ("installs the whole binary", "governance verbs included", "bad", ""),
    ], ["", ""]),
    ("as the Needs read", [
        ("the same project", "", "accent", ""),
        ("installs the items member", "state computed, exposed", "ok", ""),
    ], ["", ""]),
)
figI2 = bars(
    f"The {TOP} top-level verbs sit in {len(FAMILIES)} families; the items member takes a subset",
    [(f, c, "ok" if f in ("authoring", "orientation") else "muted", "") for f, c in FAMILIES],
    unit=("verb", "verbs"),
)
figP1 = logic_lanes(
    "A red gate in a custom process is refused the same in all three places",
    ("the Need's test", [
        ("a process the engine never saw", "plain SysML, in a fixture", "accent", ""),
        ("one gate red", "", "warn", ""),
        ("hook, commit gate, CI refuse", "one verdict text", "ok", ""),
    ], ["", "", ""]),
    ("what fails it", [
        ("the same process", "", "accent", ""),
        ("an engine edit is needed", "or one place lets it through", "bad", ""),
        ("the step advances", "", "bad", ""),
    ], ["", "", ""]),
)
figP2 = downstream(
    f"Hooks and CI run the gating family at {GATE_SITES} sites today",
    ("hooks and CI", "where a refusal fires", ""),
    [(verb, "", f"{n}", "ok" if n else "muted") for verb, n in sorted(HOOKS_CI.items(), key=lambda kv: -kv[1])],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0466", "ask-sample", "Sample fork", "sample"),
    (GATE["name"], "ask-members", "Members", "members"),
    (GATE["name"], "ask-issues", "Items", "issues"),
    (GATE["name"], "ask-process", "Process", "process"),
]
panel_bodies = {
    "sample": f"""<h2>The recall rule reads at which sample</h2>
<p>The item: <q>the weighted rule picks {LOW} on the bench's default {N50} cases and {MID} on {N100} - which sample does the rule read, or must a challenger win both.</q></p>
<p><strong>The fork:</strong> the rule holds near hits and rows and allows one median position where the far arm gains three. At {N50}, {LOW} moves off's median {med50_off} to {med50_low}, gains {gain50_low}, and wins. At {N100}, off's median is {med100_off}; {LOW} moves it {move100_low} and is refused; {MID} moves it one, gains {gain100_mid}, and wins. Hand set: {HAND[MID]['reachable']}/{HAND[MID]['of']} at either, {HAND[OFF]['reachable']}/{HAND[OFF]['of']} at off.</p>
{fig5}
{fig6}
{fig7}
{courses([
    (f"A: read at {N50}", f"constant {MID} to {LOW}", f"near arm gives up {top3_50_lost} of {R50[OFF]['hop1']['top3Of']} top-3 places at {N50}, {top3_100_lost} of {R100[OFF]['hop1']['top3Of']} at {N100}"),
    (f"B: read at {N100}", f"sample size becomes a clause; {MID} stands", "a clause written after the reading"),
    ("C: win at both", f"a challenger must win at {N50} and {N100}; {MID} stands", "a setting better at all but one size never lands"),
    ("Do nothing", "nothing", f"{MID} in force under a rule that picks {LOW} by default"),
])}
<p><strong>True in the model:</strong> both sweeps, quoted. <strong>Mine:</strong> B. Plainly, B and C are post hoc, and A chases a move that reverses between two samples. <strong>What decides it:</strong> is a one-position median move at the bench's sample size signal (A) or noise (B, C)? <strong>Wrong if</strong> a second {N100}-case sweep put off's median at {med50_off}: {LOW} is then admissible at both.</p>
<div class="opts" data-records="d0466"><label><input type="radio" name="ask-sample" value="Option B: the weighted rule reads at 100 cases - the sample size becomes a clause of the rule, sweeps for this constant run with --cases 100 on both arms, and 1.25 stands (recommended)">B: read at {N100} (recommended)</label><label><input type="radio" name="ask-sample" value="Option A: the rule reads at the bench default of 50 cases; DOMINANCE moves to 1.1">A: read at {N50}</label><label><input type="radio" name="ask-sample" value="Option C: a challenger must be admissible and highest at both 50 and 100 cases; 1.25 stands as the incumbent">C: win at both</label><label><input type="radio" name="ask-sample" value="Do nothing: 1.25 stays with a rule that, read at its default, picks 1.1; the Issue stays open">Do nothing</label></div>""",
    "members": f"""<h2>Small members: what a rebuild touches</h2>
{needs_table(TABS_NEEDS["members"])}
{figM1}
{figM2}
<p><strong>Yours:</strong> granularity, modular, easy to rebuild. <strong>Mine:</strong> Fresh as the criterion, the by-function count, a guard. <strong>Wider than you meant?</strong> The third Need schedules every module; say so if the dispatch shell may stay whole.</p>
{needs_opts("ask-members", TABS_NEEDS["members"])}""",
    "issues": f"""<h2>Items member: the tracker without the engine's governance</h2>
{needs_table(TABS_NEEDS["issues"])}
{figI1}
{figI2}
<p><strong>Yours:</strong> the name, the subset, engine Decisions kept out. <strong>Mine:</strong> which verbs are the subset, and that a member boundary is where a control binds. <strong>Wider than you meant?</strong> The third Need lists every control naming no member; say so if only this member needs an anchor.</p>
{needs_opts("ask-issues", TABS_NEEDS["issues"])}""",
    "process": f"""<h2>Custom process: read from plain SysML, refused in three places alike</h2>
{needs_table(TABS_NEEDS["process"])}
{figP1}
{figP2}
<p><strong>Yours:</strong> <q>absolute discipline</q>, a need that grew as the project went. <strong>Mine:</strong> parity across editor hook, commit gate and CI as its measure; a fixture project as the test. <strong>The pain I lack:</strong> the last concrete process you could not hook, and where it slipped. <strong>Wider than you meant?</strong> Say so if one of the three places may be advisory.</p>
{needs_opts("ask-process", TABS_NEEDS["process"])}""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Read the recall rule at {N100} cases</strong> so {MID} stands; <strong>read the {N_NEEDS} Needs back</strong> and say which is wider than you meant. <strong>Wrong if</strong> a one-position median move at {N50} cases is signal to you.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> 1</span><span class="chip"><b>Needs, outside the queue lens</b> {N_NEEDS}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} files carried uncommitted edits; {tests} tests, {failing} failing. Needs and sweeps quoted; record names glossed. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""

TITLE = f"Read the recall rule at {N100} cases, then read back the {N_NEEDS} Needs"
SUB = "One fork and one Business gate"


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
      f"; summed {words(page)}; {N_NEEDS} Needs over three tabs; the rule picks {W50} at {N50} and {W100} at {N100}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
