#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-13 (twenty-first) queue change: the twentieth publish's four asks
stand (the retro name D0461, the keystone's second source D0465, the sample fork D0466, the Issue-first retro D0467) and
one joined them. D0468 is a process-change ratification held under D0337: the stop-hook p90 over the last 25 fires and
the git-facts cache byte length are computed Indicators, each with a declared trigger - above 10 000 ms and above
8 000 000 bytes - that surfaces named work and gates nothing (D0088). The contract file is keystone-locked, so the edit
rode this marked Decision; the two numbers are the judgment the human can reject. The hook trigger fires on the tree that
declared it, and the guard on the critical path of nearly every fire past the receipt is tracked with a resolver.
One tab per ask (D0404); the four carried tabs are the twentieth builder's, every count a facts.py fact (sections 21
and 22), never typed. The ledger is RE-READ by facts.py, so the p90 on the page is today's, not the Decision's.

Usage: python scripts/exec_brief/build_2026_09_13_host_cost_thresholds.py <facts.json> <previous.html> <out.html>
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


def n(x):
    """A count or a measure, digit-grouped with a comma - the same form charts.bars prints at the bar end."""
    return f"{int(x):,}" if float(x) == int(x) else f"{x:,}"


# ---- the queue: four ratifications and one fork -------------------------------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
if len(members) != pending or v("pendingForks") != 1:
    sys.exit(f"refusing: facts disagree - {len(members)} members, {pending} pending, {v('pendingForks')} forks")
if [p["slug"] for p in members] != ["d0461", "d0465", "d0466", "d0467", "d0468"]:
    sys.exit(f"refusing: this page is written for the retro name, the keystone, the sample fork, the Issue-first retro and the host-cost thresholds; the queue is {[p['slug'] for p in members]}")
if [p["fork"] for p in members] != [False, False, True, False, False]:
    sys.exit("refusing: the fork must be the third member and the only one")

tests, failing = v("suiteTests"), v("suiteFailed")
dirty = v("treeUncommitted")["modified"]

# ---- the retro needle (D0461): how many retro gates write the capital form, and the note that bears on it ----
RT = v("retrosNamingDecisionUpper")
retros, retros_upper = RT["retros"], RT["namingD0NNN"]
NOTE = v("retroNoteStatement")
note_paras = [p.strip() for p in NOTE["text"].split("\n\n") if p.strip()]
NOTE_CORE = "I usually want issues coming out of retros, not decisions. Usually retro findings are based on empirical evidence. If they're proposing a non occasion issue, it could be a decision, I suppose"
if NOTE_CORE not in NOTE["text"]:
    sys.exit("refusing: the page quotes a span of the retro note that the record does not carry")

# ---- the keystone's second source (D0465): the code, the population it reads, the live pair --------------
KP = v("keystoneCharterPath")
if not (KP["acceptedChartersFn"] and KP["charterTargetsFn"] and KP["isAuthorisingCharterFn"] and KP["keystoneTakesCharters"]):
    sys.exit(f"refusing: the page says the second source is applied on this tree; the source reads {KP}")
MC = v("markedDecisionCensus")
if MC["humanAccepted"] + MC["autoAccepted"] + MC["noAcceptResult"] != MC["marked"]:
    sys.exit(f"refusing: the marked census does not sum: {MC}")
CE = v("charterEdgesToMarked")
LP = v("keystoneLivePair")
_pass = re.search(r"\[guard:process-change\] (PASS), (\d+) warning", LP)
_fail = re.search(r"\[guard:process-change\] (FAIL), (\d+) violation", LP)
if not (_pass and _fail):
    sys.exit(f"refusing: the live pair does not carry a PASS and a FAIL verdict: {LP}")
PASS_WORD, PASS_N, FAIL_WORD, FAIL_N = _pass.group(1), int(_pass.group(2)), _fail.group(1), int(_fail.group(2))

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

# ---- the Issue-first retro (D0467): what retros discharge themselves with today ---------------------------
RD = v("retroDischargeCensus")
if RD["retros"] != retros:
    sys.exit(f"refusing: the two retro censuses count different retro gates: {RD['retros']} and {retros}")

# ---- the host-cost thresholds (D0468): the lens, the contract, the ledger, the cache, the anchors, the Issue ----
HI = v("hostCostIndicators")
TH = v("hostCostThresholds")
SF = v("stopFireTail")
CB = v("gitFactsCacheBytes")
AN = v("hostCostAnchors")
CI = v("hookCriticalPathIssue")
HOOK, CACHE = "hookLatencyIndicator", "gitFactsSizeIndicator"
THR_MS, THR_B = TH[HOOK]["above"], TH[CACHE]["above"]
P90, MED = SF["p90"], SF["median"]
if HI[HOOK]["latest"] != P90:
    sys.exit(f"refusing: the lens reads a p90 of {HI[HOOK]['latest']} and the ledger {P90} - a fire landed between the two reads; re-run facts.py")
if HI[CACHE]["latest"] != CB:
    sys.exit(f"refusing: the lens reads {HI[CACHE]['latest']} cache bytes and the disk {CB}; re-run facts.py")
if (HI[HOOK]["threshold"], HI[CACHE]["threshold"]) != (f"above {n(THR_MS)}".replace(",", ""), f"above {n(THR_B)}".replace(",", "")):
    sys.exit(f"refusing: the lens and the contract declare different thresholds: {HI[HOOK]['threshold']}, {HI[CACHE]['threshold']} vs {THR_MS}, {THR_B}")
if HI[HOOK]["crossed"] != (P90 > THR_MS) or HI[HOOK]["triggered"] != (P90 > THR_MS):
    sys.exit(f"refusing: the hook trigger's state {HI[HOOK]} does not follow from {P90} against {THR_MS}")
if HI[CACHE]["crossed"] != (CB > THR_B) or HI[CACHE]["triggered"] != (CB > THR_B):
    sys.exit(f"refusing: the cache trigger's state {HI[CACHE]} does not follow from {CB} against {THR_B}")
if not (P90 > THR_MS and CB <= THR_B):
    sys.exit(f"refusing: this page says the hook trigger fires and the cache one is silent; the readings are {P90}/{THR_MS} and {CB}/{THR_B}")
if AN["slowFireMs"] != SF["slowFireMs"]:
    sys.exit(f"refusing: the Decision's slow-fire mark {AN['slowFireMs']} is not the one the ledger was read with ({SF['slowFireMs']})")
if AN["atDeclaration"]["threshold"] != THR_MS or AN["atDeclaration"]["p90"] <= THR_MS:
    sys.exit(f"refusing: the Decision says the trigger fired at declaration against {THR_MS}; it reads {AN['atDeclaration']}")
if not (AN["status"] == "proposed" and AN["marker"] and AN["notAFork"] and AN["gatesNothing"] and AN["keystonePath"]):
    sys.exit(f"refusing: the page says a proposed, marked, non-fork Decision that gates nothing on a locked path; the record reads {AN}")
if not (CI["namedBySurfaces"] and CI["resolver"]):
    sys.exit(f"refusing: the page says the trigger points at a tracked Issue with a resolver; the facts read {CI}")
if SF["fires"] != 25 or SF["pastReceipt"] + SF["fromReceipt"] != 25:
    sys.exit(f"refusing: the indicator binds the last 25 fires; the ledger read {SF['fires']}")
LED, SLOW = SF["decisionScaffoldingLed"], SF["pastReceipt"]
DS_MIN, DS_MAX = SF["decisionScaffoldingMs"]["min"], SF["decisionScaffoldingMs"]["max"]
if not (LED and DS_MIN and DS_MAX and SLOW >= 3):
    sys.exit("refusing: the page names the guard on the critical path; the ledger attributes none")
CACHE_MB = CB / 1_000_000
RATIO_CUT = THR_B / AN["cacheBytesAfterCut"]
if not (1.9 < RATIO_CUT < 2.1 and THR_B / (AN["issue440Mb"] * 1_000_000) < 1 / 3):
    sys.exit(f"refusing: the page says twice the settled size and under a third of the unnoticed one; {THR_B} against {AN['cacheBytesAfterCut']} and {AN['issue440Mb']} MB")
if not (AN["slowFireMs"] < AN["issue442MedianMs"] < AN["issue441SingleFireMs"] < THR_MS < P90):
    sys.exit("refusing: the page says the threshold sits above every anchoring figure and under today's p90; the facts do not order that way")


def courses(rows):
    body = "".join(f"<tr><td>{a}</td><td>{b}</td><td>{c}</td></tr>" for a, b, c in rows)
    return (f'<div class="tbl-wrap"><table><thead><tr><th>course</th><th>what changes</th><th>what it leaves</th></tr></thead>'
            f'<tbody>{body}</tbody></table></div>')


# ---- exhibits: two per ask, the fork carries three, the thresholds carry three -----------------------------
fig1 = logic_lanes(
    "A retro naming a Decision in capitals is read as naming that Decision",
    ("before", [
        ("retro: tracked by a Decision, in capitals", "", "accent", ""),
        ("guard: names no item", "refused", "bad", ""),
    ], ["", ""]),
    ("with the rule", [
        ("the same retro", "", "accent", ""),
        ("guard: the item exists", "form, not relevance", "ok", ""),
    ], ["", ""]),
)
fig2 = downstream(
    f"The needle lands on {retros_upper} of {retros} retro gates",
    ("read the capital form", "", ""),
    [
        ("retro gates", "", f"{retros}", "muted"),
        ("write a Decision in capitals", "read as an item", f"{retros_upper}", "ok"),
        ("Issue, DC as words", "unmatched", "0", "muted"),
    ],
)
fig3 = logic_lanes(
    "The lock is blind to acceptance today; the charter path makes it read yours",
    ("today", [
        ("Decision held, then you accept", "its file no longer changed", "accent", ""),
        ("lock: no marked file", "the decided edit refused", "bad", ""),
    ], ["", ""]),
    ("changed", [
        ("sprint names it as charter", "marked, accepted by you", "accent", ""),
        ("lock: charter read from disk", "pass; one line names it", "ok", ""),
    ], ["", ""]),
)
fig4 = downstream(
    f"Of {MC['marked']} marked Decisions, {MC['humanAccepted']} can authorise as a charter",
    ("read the charter edge", "of a changed sprint record", ""),
    [
        ("accepted by you", "authorise", f"{MC['humanAccepted']}", "ok"),
        ("standing consent", "authorise nothing", f"{MC['autoAccepted']}", "muted"),
        ("unaccepted", "authorise nothing", f"{MC['noAcceptResult']}", "muted"),
        ("charter edges to yours", f"of {CE['edges']}", f"{CE['toHumanAcceptedMarked']}", "warn"),
    ],
)
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
fig8 = logic_lanes(
    "A Decision-only retro is silent today; with the rule it gets one warning",
    ("today", [
        ("retro; the commit adds a Decision only", "", "accent", ""),
        ("guard: tracked, silent", "Issue and Decision read alike", "bad", ""),
    ], ["", ""]),
    ("changed", [
        ("the same retro", "no Issue or task added", "accent", ""),
        ("guard: one warning", "silent with 'standing rule:'", "ok", ""),
    ], ["", ""]),
)
fig9 = downstream(
    f"Of {RD['retros']} retro gates, {RD['decisionOnly']} have a Decision and no Issue or task",
    ("prefer an Issue", "warn on Decision-only", ""),
    [
        ("naming an Issue", "silent, the default", f"{RD['namingIssue']}", "ok"),
        ("naming a task", "silent", f"{RD['namingTask']}", "ok"),
        ("a Decision only", "the warning's class", f"{RD['decisionOnly']}", "warn"),
        ("none of the three", "", f"{RD['namingNone']}", "muted"),
    ],
)
fig10 = logic_lanes(
    "An unowned cost is read by nobody; a trigger says the work",
    ("today", [
        ("hook cost, cache size", "measured; owned by nothing", "accent", ""),
        ("no threshold", "growth read late", "bad", ""),
    ], ["", ""]),
    ("changed", [
        ("the same two numbers", f"p90 of {SF['fires']} fires; bytes", "accent", ""),
        (f"above {n(THR_MS)} ms / {n(THR_B)} B", "surfaces work; gates nothing", "ok", ""),
    ], ["", ""]),
)
fig11 = downstream(
    f"The hook trigger fires at {n(P90)} ms; the cache one is silent",
    ("declare the two triggers", "lens and burndown", ""),
    [
        (f"hook p90, last {SF['fires']} fires", f"above {n(THR_MS)}: fires", f"{n(P90)} ms", "bad"),
        ("cache on disk", f"under {n(THR_B)}: silent", f"{n(CB)} B", "ok"),
        ("slow fires led by one guard", f"{n(DS_MIN)}-{n(DS_MAX)} ms of each", f"{LED} of {SLOW}", "warn"),
    ],
)
_anchor_rows = sorted([
    ("one fire is slow", AN["slowFireMs"], "muted", ""),
    ("median that raised an Issue", AN["issue442MedianMs"], "muted", ""),
    ("fire that raised an Issue", AN["issue441SingleFireMs"], "muted", ""),
    ("the threshold", int(THR_MS), "warn", ""),
    ("p90 today", P90, "bad", ""),
], key=lambda r: r[1])
fig12 = bars(
    "The threshold sits above every figure that raised an Issue",
    _anchor_rows,
    unit=" ms",
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0461", "ask-retro", "Retro name", "retro"),
    ("d0465", "ask-keystone", "Keystone", "keystone"),
    ("d0466", "ask-sample", "Sample fork", "sample"),
    ("d0467", "ask-issue-first", "Retro Issue", "issuefirst"),
    ("d0468", "ask-host-cost", "Host costs", "hostcost"),
]
NOTE_SHORT = "I usually want issues coming out of retros, not decisions"
if NOTE_SHORT not in NOTE_CORE:
    sys.exit("refusing: the short span is not inside the note")
panel_bodies = {
    "retro": f"""<h2>A Decision named in capitals is a named item</h2>
<p>The item: <q>the retro-backlog guard reads a Decision written in capitals, the way every document writes one, as the item its file names in lower case; only the Decision form folds - Issue and DC in prose stay words.</q></p>
<p><strong>Accepting binds:</strong> a fourth needle, on trunk already; a retro may name an existing Decision in either case. The guard reads form, not relevance.</p>
<p><strong>Your note, verbatim:</strong> <q>{escape(note_paras[0])}</q></p>
{fig1}
{fig2}
{courses([
    ("Accept", "nothing more moves", f"{retros_upper} retro gates read as written"),
    ("Hold", "the code stands unratified", "the same"),
    ("Reject", "one revert", "a spelling rule no document follows"),
])}
<p><strong>My reading of your words:</strong> they lean against a Decision as a retro's product; this item makes a Decision in capitals count as one. <strong>True in the model:</strong> {retros_upper} of {retros} retro gates write the capital form; the item reads a name and picks no form - the form is the fourth tab. <strong>Wrong if</strong> you want retros discharged by an Issue or a task only.</p>
<p><strong>Accept:</strong> nothing. <strong>Hold:</strong> nothing. <strong>Reject:</strong> revert, recorded quoting you.</p>
<div class="opts" data-records="d0461"><label><input type="radio" name="ask-retro" value="Accept: the retro-backlog guard reads a Decision written in capitals as the item its file names; only the Decision form folds (recommended)">Accept (recommended)</label><label><input type="radio" name="ask-retro" value="Hold: the retro needle stays proposed; the code stands">Hold</label><label><input type="radio" name="ask-retro" value="Reject: the widened needle is reverted; a retro names a Decision in lower case or not at all">Reject</label></div>""",
    "keystone": f"""<h2>An accepted charter unlocks the edit it decided</h2>
<p>The item: <q>a locked-surface edit is authorised by a human-accepted marked Decision that the co-committed sprint names as its charter, not only by a marked Decision riding the commit.</q></p>
<p><strong>Today</strong> the lock reads the marker, never the acceptance: a proposed Decision in the commit unlocks an edit; one you accepted earlier does not, its file being unchanged. <strong>Accepting binds:</strong> a second source, applied on this tree: a changed sprint record whose charter is a marked Decision you accepted. Proposed, rejected, auto-accepted or unmarked authorises nothing. The first source stays.</p>
{fig3}
{fig4}
<p><strong>Live pair on this tree:</strong> the sprint chartered by a Decision you accepted, its file out of the commit: <b>{PASS_WORD}</b>, {PASS_N} warning. The same sprint chartered by a proposed one: <b>{FAIL_WORD}</b>, {FAIL_N} violation naming both paths.</p>
{courses([
    ("Accept", "nothing more moves", "accept first, edit after has a path"),
    ("Hold", "the code stands unratified", "the same"),
    ("Reject", "the second source reverted", "an accepted change lands by riding another marked Decision or touching its own file"),
])}
<p><strong>Residual:</strong> an accepted charter authorises every locked edit in its commit, as the first source did - the lock binds the commit, not the file. <strong>Wrong if</strong> you want the lock to read only the files a commit changes.</p>
<p><strong>Accept:</strong> nothing. <strong>Hold:</strong> nothing. <strong>Reject:</strong> revert, recorded quoting you.</p>
<div class="opts" data-records="d0465"><label><input type="radio" name="ask-keystone" value="Accept: a locked-surface edit is authorised by a human-accepted marked Decision that the co-committed sprint names as its charter, not only by a marked Decision riding the commit (recommended)">Accept (recommended)</label><label><input type="radio" name="ask-keystone" value="Hold: the charter source stays proposed; the code stands">Hold</label><label><input type="radio" name="ask-keystone" value="Reject: the second authorising source is reverted; a locked edit needs a marked Decision among the commit's changed files">Reject</label></div>""",
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
    "issuefirst": f"""<h2>A retro finding is an Issue first</h2>
<p>The item: <q>a retro finding is discharged by an Issue by default; a retro whose only new item is a Decision says why the finding is not a one-off, or the retro-backlog guard warns.</q></p>
<p><strong>From your note</strong> on the first tab: <q>{NOTE_SHORT}</q>. This item is that preference as a rule; the first tab is the name fix you said you did not care about.</p>
<p><strong>Accepting binds:</strong> the guard warns, never blocks, when a retro's commit adds Decisions only, unless the retro carries <code>standing rule:</code> and why. An Issue or a task stays silent, as today. Not applied yet; it lands by the second tab's path.</p>
{fig8}
{fig9}
{courses([
    ("Accept", "one warning, its test pair, a skill line, a doc row", "a Decision-only retro naming its rule stays silent"),
    ("Hold", "nothing moves", "the three forms stay equal"),
    ("Reject", "recorded quoting you", "the same"),
    ("Do nothing", "nothing", "the guard keeps treating the three forms as equal"),
])}
<p><strong>True in the model:</strong> the census reads retro text, not which item the commit added. <strong>My assumption:</strong> a warning in the burndown is the shape you asked for; a block would be a completeness gate. <strong>Wrong if</strong> you want a Decision-only retro refused outright.</p>
<p><strong>Accept:</strong> apply once, chartered by this item. <strong>Hold:</strong> nothing. <strong>Reject:</strong> recorded quoting you.</p>
<div class="opts" data-records="d0467"><label><input type="radio" name="ask-issue-first" value="Accept: a retro finding is discharged by an Issue by default; a retro whose only new item is a Decision says why the finding is not a one-off, or the retro-backlog guard warns (recommended)">Accept (recommended)</label><label><input type="radio" name="ask-issue-first" value="Hold: the Issue-first preference stays proposed; the guard treats the three forms as equal">Hold</label><label><input type="radio" name="ask-issue-first" value="Reject: the retro-backlog guard keeps treating an Issue, a task and a Decision as equal discharges; recorded rejected">Reject</label></div>""",
    "hostcost": f"""<h2>Two host costs carry a threshold each</h2>
<p>The item: <q>the stop-hook p90 over the last {SF['fires']} fires and the git-facts cache bytes are indicators, triggers declared at {n(THR_MS)} ms and {n(THR_B)} bytes; crossing one surfaces work and gates nothing.</q></p>
<p><strong>Why yours:</strong> the triggers file is locked; the edit rode this marked Decision, applied on this tree. <strong>Accepting binds:</strong> the two numbers, nothing procedural.</p>
{fig10}
{fig11}
{fig12}
{courses([
    ("Accept", "nothing", f"the hook trigger fires: {n(P90)} above {n(THR_MS)}"),
    ("Other number", "one clause, your number, its anchor", f"at {n(AN['slowFireMs'])} it fires for good; above {n(P90)} it is silent today"),
    ("Hold", "nothing", "triggers surface work unratified"),
])}
<p><strong>Bytes anchor:</strong> twice the {n(AN['cacheBytesAfterCut'])} the cache settled at once dead heads were cut; under a third of the {AN['issue440Mb']} MB it reached unread. <strong>True in the model:</strong> one guard leads {LED} of {SLOW} slow fires; its fix is tracked. <strong>Mine:</strong> a ten-second tail stall is where you feel the turn boundary. <strong>What decides it:</strong> the pause at the tail you tolerate before it is work. <strong>Wrong if</strong> the p90 stays above {n(THR_MS)} after that fix lands: the number reads a floor; it moves with its anchor.</p>
<div class="opts" data-records="d0468"><label><input type="radio" name="ask-host-cost" value="Accept: the stop-hook p90 over the last 25 fires triggers above 10000 ms and the git-facts cache above 8000000 bytes; each surfaces named work and gates nothing (recommended)">Accept (recommended)</label><label><input type="radio" name="ask-host-cost" value="Hold: the two thresholds stay proposed; the triggers keep surfacing work">Hold</label><label><input type="radio" name="ask-host-cost" value="Other number: one superseding clause carries a different threshold and its anchor - the number goes in the note">Other number</label></div>""",
}


def frame(panels_html, tabs_html, title, sub):
    return f"""<div class="page">
<p class="sub" data-digest="project">keel &middot; williamweatherholtz/sysmlv2-ai-toolkit</p>
<h1 data-digest="title">{title}</h1>
<div class="topbar"><p class="sub" data-digest="subtitle">{sub}</p><button class="copy" data-copy type="button" aria-label="Copy this brief for AI">&#8681; Copy for AI</button></div>

<div class="ask"><p class="verdict"><strong>Accept the four controls</strong> and <strong>read the recall rule at {N100} cases</strong> so {MID} stands. <strong>Wrong if</strong> a one-position median move at {N50} cases is signal to you.</p></div>
<div class="chips"><span class="chip"><b>Waiting on you</b> {pending}</span><span class="chip"><b>Forks</b> 1</span><span class="chip"><b>Hook p90</b> {n(P90)} ms</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Anything to add<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed from the repository at {TREE} on {DATE}; {dirty} files carried uncommitted edits; {tests} tests, {failing} failing. Sweeps, pair and note quoted from records; ledger and cache read live. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""

TITLE = f"Accept four controls, and read the recall rule at {N100} cases so its setting stands"
SUB = "Four ratifications and one fork"


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
      ", ".join(f"{k} {n_}" for k, n_ in sorted(panel_words.items(), key=lambda kv: -kv[1])) +
      f"; per tab " + ", ".join(f"{k} {frame_words + n_}" for k, n_ in panel_words.items()) +
      f"; summed {words(page)}; hook p90 {P90} ms against {THR_MS:g} (fires), cache {CB} B against {THR_B:g} (silent); "
      f"{LED} of {SLOW} slow fires led by decision-scaffolding")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
