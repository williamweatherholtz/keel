#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-19 (fifty-second) queue change: the twenty-five held records of the
fifty-first page were all answered from chat ("Accept all decisions as recommended") and recorded; two new held records from
sprint 746 take their place. First: the downloaded binary, run over a tree an older `keel init` had scaffolded, rolled itself
back on sixteen retired-verb references in files the run never writes (the adopter's CLAUDE.md and its two project-owned
contracts); the record makes the spelling a retired verb has today ONE shipped fact with two readers - the guard's `today it
is` clause and a `verb-respell` step of migrate that rewrites the adopter's own living docs, CLAUDE.md joining the run's scope
and the process-change lock treating a locked file that is exactly the respell of its own HEAD text as the engine arriving.
Second: two sessions on two machines each minted the same two issue numbers into their own per-actor files, git saw no
conflict, and the merge landed green because guard duplicate-identity keyed a declared name on its package while every lens
reads the bare name; the record makes an allocated name (issue / st / us + digits) one namespace across every package (guard
class 5) and renumbers the later-landing pair. Neither carries an OPTION token; both applied under the held marker, so two
held acceptances, no fork. Two tabs, two asks (D0404); every count is a facts.py fact, never typed; each metric says what it does.

Usage: python scripts/exec_brief/build_2026_09_19_two_held.py <facts.json> <previous.html> <out.html>
"""
import re
import sys
from html import escape

sys.path.insert(0, __file__.rsplit("/", 1)[0] if "/" in __file__ else __file__.rsplit("\\", 1)[0])
# (charts.bars is not used: the two mandatory exhibits per ask are the lanes and the downstream landing)
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


# ---- the queue: two held process changes, no fork -----------------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
QUEUE = ["d0523", "d0524"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != 0 or any(m["fork"] for m in members):
    sys.exit(f"refusing: the page says neither held Decision is a fork; the lens says {[m['slug'] for m in members if m['fork']]}")
tests, failing = v("suiteTests"), v("suiteFailed")
dirty = v("treeUncommitted")["modified"]

# -- ask 1: a verb fold travels with migrate --
VF = v("aVerbFoldTravelsWithMigrate")
if VF["status"] != "proposed" or VF["marker"] != "#ProspectiveChange" or VF["acceptance"] is not None or VF["fork"]:
    sys.exit(f"refusing: the page says a held, unaccepted, unforked process change (verb fold): {VF['status']}, {VF['marker']}, {VF['acceptance']}, fork={VF['fork']}")
if not (VF["derivedFromSt167"] and VF["derivedFromSt168"] and VF["namesHeld"]):
    sys.exit("refusing: the verb-fold record must derive from the human's two statements and say it is held")
for _k in ("namesIssue622", "namesSixteenViolations", "namesTheScaffoldVerbs", "namesTheTwoContracts", "namesOwnershipPredicate", "namesTheFiveFolds",
           "namesNoActionButHandEdit", "namesOneFactTwoReaders", "namesTheContract", "namesEmbeddedCopy", "namesTodayItIs", "namesTheStep",
           "namesOneEditPerFile", "namesNoSpellingStays", "namesClaudeMdJoinsScope", "namesLockByContent", "namesRejectedDiscount", "namesCongruent",
           "namesIdempotent", "namesResolver", "namesFutureFoldAddsRow", "namesTheFollowUpRelease", "namesResidual"):
    if not VF[_k]:
        sys.exit(f"refusing: the verb-fold record does not say what the page quotes it saying: {_k}")
VT = VF["today"]
if not (VT["contractExists"] and VT["contractNamesD0523"] and VT["contractSaysEngineOwned"] and VT["rowAddTask"] == "record task"
        and VT["rowReport"] == "render report" and VT["rowGithubDecider"] == "github decider" and VT["noLensRow"]):
    sys.exit(f"refusing: the shipped contract is not as the page says: {VT}")
if not (VT["readerOneLine"] and VT["readerOneReadsEmbedded"] and VT["respellingLine"] and VT["todayItIsLine"] and VT["stepLine"] and VT["stepId"]
        and VT["stepPlannedAfterResync"] and VT["doorReadsClaudeMd"] and VT["restoreTakesClaudeMdWhenHeld"] and VT["pairTestLine"]
        and VT["pairAssertsOneEditPerFile"] and VT["pairAssertsIdempotent"] and VT["lockExemptionLine"] and VT["lockMessageNamesD0523"]
        and VT["lockPairLine"] and VT["lockPairHoldsOneMoreByte"]):
    sys.exit(f"refusing: the two readers, the step, the door, the restore, the lock exemption or a pair is not in source as the page says: {VT}")
N_ROWS = VT["contractRows"]
VL = VF["live"]["cliReference"]
if not (VL["exit0"] and VL["verdict"] == "PASS" and VL["violations"] == 0):
    sys.exit(f"refusing: the page says cli-reference is green on this tree: {VL}")
N_CLI_SCANNED = VL["scanned"]
for _n in ("st167", "st168"):
    if not (VF["intake"][_n]["present"] and VF["intake"][_n]["channel"] == "chat"):
        sys.exit(f"refusing: the human's direction {_n} is not a chat statement in the intake file: {VF['intake'][_n]}")
I622 = VF["issue622"]
if not (I622["exists"] and I622["resolverAsExpected"] and I622["resolverDodNamesIssue"]):
    sys.exit(f"refusing: the finding, its resolver edge or the resolver's DoD is not as the page says: {I622}")
R23 = VF["resolver"]
if not (R23["onBacklog"] and R23["dodResults"] and R23["dodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the resolver is on the backlog with a passing DoD result: {R23}")
R23_SHA = R23["dodResults"][-1]["judgedAgainst"][:8]
REL = VF["release051"]
if not (REL["present"] and REL["tag"] and REL["commit"] and REL["purposeNamesIssue622"] and not REL["purposeNamesIssue623"]):
    sys.exit(f"refusing: the page says the follow-up Release is recorded, names the fixed finding and not the open one: {REL}")
S746 = VF["sprint746"]
if not (S746["exists"] and S746["chartersAsExpected"] and S746["deliveredNamesBoth"] and S746["storyDod"] and S746["storyDod"][-1]["outcome"] == "pass"
        and S746["reviewSaysIssue623Open"]):
    sys.exit(f"refusing: sprint 746 must exist, be chartered, name both mid-sprint items, carry a passing story DoD and say the open finding is open: {S746}")
N_RESULTS46, N_GATE46, N_RETRO46, PTS46 = S746["results"], S746["gateResults"], S746["retroFindings"], S746["estimatedPoints"]
if not VT["landedCommit"]:
    sys.exit("refusing: the landing of the verb-respell step is not in git log")
VF_LANDED, VF_CI = VT["landedCommit"], VT["landedCi"]

# -- ask 2: an allocated name is one namespace across packages --
AN = v("anAllocatedNameIsOneNamespaceAcrossPackages")
if AN["status"] != "proposed" or AN["marker"] != "#ProspectiveChange" or AN["acceptance"] is not None or AN["fork"]:
    sys.exit(f"refusing: the page says a held, unaccepted, unforked process change (one namespace): {AN['status']}, {AN['marker']}, {AN['acceptance']}, fork={AN['fork']}")
# its NOT A FORK sits in the rationale (namesRejectedCentralAllocator below reads it), not the decision field the notAFork fact scans
for _k in ("namesTwoSessions", "namesTheBase", "namesTheTwoFiles", "namesTheMergeGreen", "namesTheAllocator", "namesClassTwoPerPackage", "namesTheLensSymptom",
           "namesTheLoss", "namesOneNamespace", "namesClassFive", "namesBothLocations", "namesShapeOnly", "namesClassTwoStands", "namesRenumberLaterSide",
           "namesMakeTheClaimTrue", "namesLandGatesMerged", "namesTreeWideWouldBeWrong", "namesRenumberNotQualify", "namesRejectedCentralAllocator",
           "namesFiveClasses", "namesKnownPositive", "namesResolver", "namesRenumberedMeaning"):
    if not AN[_k]:
        sys.exit(f"refusing: the one-namespace record does not say what the page quotes it saying: {_k}")
AT = AN["today"]
if not (AT["classFiveCommentLine"] and AT["allocatedMapLine"] and AT["classFiveCheckLine"] and AT["violationNamesBoth"] and AT["violationSaysRenumber"]
        and AT["predicateLine"] and AT["predicatePrefixes"] and AT["predicateDigitsOnly"] and AT["predicateDocNamesDisp"] and AT["pairTestLine"]
        and AT["classTwoUnchanged"] and AT["allocatorCommentNamesClassFive"] and AT["allocatorScansWholeTree"] and AT["guardsDocRowNamesClassFive"]):
    sys.exit(f"refusing: class 5, its predicate, the pair, the allocator's comment or the guards.md row is not in source as the page says: {AT}")
AL = AN["live"]["duplicateIdentity"]
if not (AL["exit0"] and AL["verdict"] == "PASS" and AL["violations"] == 0):
    sys.exit(f"refusing: the page says duplicate-identity is green on this tree: {AL}")
N_DI_SCANNED = AL["scanned"]
LV = AN["live"]
if not (LV["openIssuesExit0"] and not LV["issue622Open"] and LV["issue623Open"] and not LV["issue624Open"]):
    sys.exit(f"refusing: the page says the two fixed findings are computed resolved and the repeated-flag one open: {LV}")
N_OPEN = LV["openIssueCount"]
N_CLOSED = sum(1 for k in ("issue622Open", "issue624Open") if LV[k] is False)   # the two sprint findings the lens computes resolved
RN = AN["renumbered"]
if not (RN["issue622Declared"] and RN["issue623Declared"] and RN["fable5DeclaresNo620or621"] and RN["opus5Declares620and621"]):
    sys.exit(f"refusing: the page says the later-landing pair was renumbered and the other side's records untouched: {RN}")
I624 = AN["issue624"]
if not (I624["exists"] and I624["resolverAsExpected"] and I624["resolverDodNamesIssue"]):
    sys.exit(f"refusing: the collision finding, its resolver edge or the resolver's DoD is not as the page says: {I624}")
I623 = AN["issue623"]
if not (I623["exists"] and I623["resolverAsExpected"]):
    sys.exit(f"refusing: the open finding or its resolver edge is not as the page says: {I623}")
R24 = AN["resolver"]
if not (R24["onBacklog"] and R24["dodResults"] and R24["dodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the resolver is on the backlog with a passing DoD result: {R24}")
R24_SHA = R24["dodResults"][-1]["judgedAgainst"][:8]
if not AT["landedCommit"]:
    sys.exit("refusing: the landing of class 5 is not in git log")
AN_LANDED, AN_CI = AT["landedCommit"], AT["landedCi"]
N_BACKLOG = AN["backlogItems"]


def courses(rows):
    body = "".join(f"<tr><td>{a}</td><td>{b}</td><td>{c}</td></tr>" for a, b, c in rows)
    return (f'<div class="tbl-wrap"><table><thead><tr><th>course</th><th>what changes</th><th>what it costs</th></tr></thead>'
            f'<tbody>{body}</tbody></table></div>')


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


def ci_word(c):
    return "green" if c == "success" else (c or "not yet read")


# ---- exhibits: the two mandatory logic exhibits per ask; every title is a claim with a verb ----------------
figV1 = logic_lanes(
    "The guard fails the adopter on words the engine wrote, and the run reverts",
    ("what happened", [
        ("new binary migrates", "older scaffold", "accent", ""),
        ("guard reads their docs", "retired verbs, red", "bad", ""),
        ("run rolls back", "hand-edit, or stay old", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("one shipped table", "verb, spelling today", "accent", ""),
        ("respell step", "rewrites their docs", "ok", ""),
        ("guard reads same fact", "green, run lands", "ok", ""),
    ], ["read by", "and by"]),
)
figV3 = downstream(
    f"One fact with {N_ROWS} rows lands on the guard, the run, the lock and the release",
    ("a retired verb's spelling today", "one contract, two readers", ""),
    [
        ("guard's message", "names the spelling", "1", "ok"),
        ("migrate step", "their docs respelled", "1", "ok"),
        ("process-change lock", "exact respell of HEAD passes", "1", "ok"),
        ("follow-up release", f"{escape(REL['tag'])}, verified from the download", "1", "ok"),
    ],
)

figA1 = logic_lanes(
    "Two clones mint the same number, git sees no conflict, and the merge lands green",
    ("what happened", [
        ("two clones mint", "same pair, own files", "accent", ""),
        ("guard keys per package", "merge lands green", "bad", ""),
        ("lens reads bare name", "one item, one hidden", "bad", ""),
    ], ["then", "so"]),
    ("as the clause reads", [
        ("same two clones", "same collision", "muted", ""),
        ("class 5 reads the tree", "prefix + digits, once", "ok", ""),
        ("merge gate refuses", "names both, renumber", "ok", ""),
    ], ["then", "so"]),
)
figA3 = downstream(
    f"One class catches no collision in {N_DI_SCANNED:,} declarations and lands on four surfaces",
    ("an allocated name is one namespace", "guard class 5", ""),
    [
        ("merge-time gate", "refuses the next collision", "1", "ok"),
        ("allocator's comment", "its claimed backstop is true", "1", "ok"),
        ("later-landing records", "renumbered", "2", "ok"),
        ("other session's records", "untouched", "0", "muted"),
    ],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0523", "ask-fold", "Verb fold", "fold"),
    ("d0524", "ask-name", "One namespace", "name"),
]
# The do-nothing course and the alternative are the radios themselves; no courses table, the tab's budget is the terse ceiling.
panel_bodies = {
    "fold": f"""<h2>Migrate writes a retired verb's spelling today into the adopter's own docs</h2>
{clause(VF)}
<p><strong>What happened:</strong> the downloaded binary, run over an older scaffold's tree, rolled itself back on <q>16 violation(s)</q> from the retired-verb guard - all in the adopter's CLAUDE.md and project-owned contracts, files the run never writes.</p>
<p><strong>What changes:</strong> a renamed verb's spelling today is one shipped table. The guard's <q>today it is</q> clause reads it; migrate's <q>verb-respell</q> step rewrites exactly what the guard would fail in the adopter's docs. CLAUDE.md joins the run's scope; a locked contract that is exactly the respell of its HEAD text is the engine arriving.</p>
{figV1}
{figV3}
<p><strong>True:</strong> every clause is in source with probe pairs; the guard is green here over {N_CLI_SCANNED:,} references; the resolver passed at {R23_SHA}; the release at {escape(REL['commit'][:8])} was verified from the published asset. <strong>Mine:</strong> rewriting leaves the adopter whole; discounting the red does not. <strong>What decides it:</strong> whether migrate may write a project's CLAUDE.md at all. <strong>Wrong if</strong> a CLAUDE.md names a retired verb in prose the step must not touch.</p>
{opts("ask-fold", "d0523", [
    ("Accept (recommended)", "Accept: the spelling a retired verb has today is one shipped contract read from the binary's embedded copy; guard cli-reference names it and keel migrate's verb-respell step rewrites the adopter's own living docs, CLAUDE.md joining the run's scope and the lock treating an exact respell of HEAD as the engine arriving (recommended)"),
    ("Discount the red instead", "Reject the rewrite: the run discounts the retired-verb violations and lands; the adopter's own guard stays red until they hand-edit"),
    ("Do nothing", "Do nothing: withdraw the step and the table; a tree an older scaffold wrote cannot be migrated by the shipped binary"),
])}""",
    "name": f"""<h2>An allocated name is one namespace across every package</h2>
{clause(AN)}
<p><strong>What happened:</strong> two sessions minted the same two issue numbers into their own per-actor files. Git saw no conflict; every guard passed, the identity guard keying names per package. Every lens reads the bare name: one item, both resolvers, one side's open defect computed resolved.</p>
<p><strong>What changes:</strong> a name the write API mints by a whole-tree number scan - issue, st, us plus digits - is one namespace. A fifth guard class: such a name declared twice anywhere is a violation naming both; the per-package class stands for every other name. The later-landing pair was renumbered.</p>
{figA1}
{figA3}
<p><strong>True:</strong> class five, its predicate and probe pair are in source; the guard is green here over {N_DI_SCANNED:,} declarations; the resolver passed at {R24_SHA}; both renumbered findings are computed resolved. <strong>Mine:</strong> a merge-time refusal is enough; a central allocator would put a network on the offline write path. <strong>What decides it:</strong> whether catching a collision at the merge, not the write, is early enough. <strong>Wrong if</strong> two clones mint the same number and neither lands through the merge gate.</p>
{opts("ask-name", "d0524", [
    ("Accept (recommended)", "Accept: guard duplicate-identity gains class 5 - an allocated name (issue / st / us + digits) declared in two places anywhere under .tracking is a violation naming both, whatever package each sits in; the colliding pair is renumbered on the later-landing side; class 2 stands for every other name (recommended)"),
    ("A central allocator instead", "Reject the guard class in favour of a remote counter that hands out issue numbers"),
    ("Do nothing", "Do nothing: withdraw class 5; two clones minting the same number merge green and every lens shows one item"),
])}""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Accept both</strong>: migrate writes a retired verb's spelling today into the adopter's own docs, and an allocated issue number is one namespace across every package. Both shipped, probed and green here; held because each changes a process.</p></div>
<div class="chips"><span class="chip"><b>Decisions waiting</b> {pending}</span><span class="chip"><b>Forks</b> 0</span><span class="chip"><b>Process changes</b> {len(QUEUE)}</span><span class="chip"><b>Findings closed</b> {N_CLOSED}</span><span class="chip"><b>Results recorded</b> {N_RESULTS46}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing; guards run live; landed {VF_LANDED} (CI {ci_word(VF_CI)}), {AN_LANDED} (CI {ci_word(AN_CI)}). <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept the two held changes the release that lands downstream needed"
SUB = "Two held process changes from the release sprint"


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
      f"; summed {words(page)}; {pending} held; table rows {N_ROWS}; scanned {N_CLI_SCANNED}/{N_DI_SCANNED}; open issues {N_OPEN}; "
      f"sprint results {N_RESULTS46}+{N_GATE46}, retro findings {N_RETRO46}, {PTS46} points; backlog {N_BACKLOG}; landings {VF_LANDED}/{AN_LANDED}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
