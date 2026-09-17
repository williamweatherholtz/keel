#!/usr/bin/env python3
# not-an-instrument: every number this writes is a fact read from facts.json (the declared sensor
# scripts/exec_brief/facts.py) or a count over that page's own members tables; it renders, it does not
# measure. Style, copy machinery and the tab strip are the previously published shell (D0404).
"""Build the standing decision brief for the 2026-09-17 (forty-fourth) queue change: the thirteen held records (eleven process
changes - stems row, CI-triple lint, probe runner, wait command, guard tests, pair file, guard names, guard source, locked
anchor, suite member, issues member - and the two safety changes from issue578) are joined by a fourteenth held process change
from sprint 738: the console (serve, deck, launcher, console_registry, reports, attestation and the page they embed) left keel-cli
for member keel-serve, which depends on every other member and is depended on by keel-cli alone; ci_runs joined keel-github,
verification joined keel-view, and the orient computation descended into keel_model::readiness, with re-exports at every old
path. Three locked files change a path string that follows the file it names (the hardening lens's console source and page, the
obligation-review process and skill registry citing deck.rs), so the lock fired for edits that change no rule; D0509 is the
marked record the lock asks for, standing on D0479 and reversing nothing. Fourteen tabs, fourteen asks (D0404); every count is
a facts.py fact, never typed; each metric says in plain words what it is doing (issue562).

Usage: python scripts/exec_brief/build_2026_09_17_serve_is_a_member.py <facts.json> <previous.html> <out.html>
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


# ---- the queue: eleven held process changes and two held safety changes -------------------------------
pending = v("pendingAcceptances")
members = v("pendingMembers")
QUEUE = ["d0494", "d0495", "d0496", "d0497", "d0499", "d0500", "d0501", "d0502", "d0503", "d0504", "d0505", "d0506", "d0508", "d0509", "d0510"]
if len(members) != pending or [p["slug"] for p in members] != QUEUE:
    sys.exit(f"refusing: this page is written for {QUEUE}; the queue is {[p['slug'] for p in members]} ({pending} pending)")
if v("pendingForks") != 0 or any(m["fork"] for m in members):
    sys.exit("refusing: the page says no held Decision is a fork; the lens disagrees")

# ---- the touched receipt is machine-local and one deep (D0421): every tab that quotes "the landing run" reads the same file.
# Read it once. Either it is a landing's green run, or a later commit's EMPTY run (nothing under test changed after the
# landing) and the landing's own run survives only as its log, written between the two commits; each tab says which it shows.
_SMV0 = v("serveSitsAboveEveryMemberAndBelowTheBinary")["live"]
LATEST, LLOG = _SMV0["landingReceipt"], _SMV0["landingLog"]
LATEST_IS_LATER_EMPTY = bool(LATEST["exists"] and LATEST["outcome"] == "empty" and LATEST["stems"] == [] and LLOG and LATEST["head"] == LLOG["nextCommit"])
if not LATEST["exists"] or (LATEST["outcome"] != "pass" and not LATEST_IS_LATER_EMPTY):
    sys.exit(f"refusing: the receipt is neither a green landing run nor the next commit's empty run: {LATEST} log={LLOG}")
if LATEST_IS_LATER_EMPTY and not (LLOG["logsInWindow"] >= 1 and LLOG["failed"] == 0 and LLOG["run"] == LLOG["passed"] and LLOG["skipped"] == 0):
    sys.exit(f"refusing: the page says the landing's run, read from its log, was green with nothing skipped: {LLOG}")
# a later sprint's green landing overwrote the earlier tabs' receipts too (one deep): each of those tabs then shows the
# latest green run and says whose it is (LAST_GREEN_SPRINT), never quoting it as its own
LATEST_IS_LATER_GREEN = bool(LATEST["exists"] and LATEST["outcome"] == "pass" and LATEST["failed"] == "0")
LATER_RUN_STANDS = LATEST_IS_LATER_EMPTY or LATEST_IS_LATER_GREEN
LAST_GREEN_TESTS = LLOG["passed"] if LATEST_IS_LATER_EMPTY else int(LATEST["passed"])
LAST_GREEN_FAILED = LLOG["failed"] if LATEST_IS_LATER_EMPTY else int(LATEST["failed"])
# the latest green run; a tab whose own landing receipt is gone says whose it shows: sprint 738's while its receipt stands,
# a later commit's by its sha once one has run over it (the receipt is one deep, D0421)
_LAND738_TO = v("serveSitsAboveEveryMemberAndBelowTheBinary")["landed"]["range"][1]
LAST_GREEN_SPRINT = "sprint 738's" if LATEST["head"] == _LAND738_TO else f"commit {LATEST['head'][:8]}'s (after sprint 738)"

# -- ask 9 (first tab): the guard-name list is a fact of the schema member, locked where it lands --
GN = v("guardNameListIsASchemaFact")
if GN["status"] != "proposed" or GN["marker"] != "#ProspectiveChange" or GN["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (guard names): {GN['status']}, {GN['marker']}, {GN['acceptance']}")
if not (GN["dependsOnD0479"] and GN["measuredToken"] and GN["notAFork"] is False):
    # the record weighs no alternative and says so by carrying none; it stands on D0479 and states its measurement
    sys.exit(f"refusing: the guard-names record must stand on D0479 and carry MEASURED:: {GN['dependsOnD0479']} {GN['measuredToken']}")
for _k in ("namesThirdExtraction", "namesTheOneReference", "namesLockCannotTell", "namesD0486Precedent", "namesNewHome", "namesReExport",
           "namesJoinsLock", "namesRendererHome", "namesNoGuardChanges", "namesOneHome", "namesDisarmClass", "namesTwoLockedFiles",
           "namesThirdMeeting", "namesReversesNothing"):
    if not GN[_k]:
        sys.exit(f"refusing: the guard-names record does not say what the page quotes it saying: {_k}")
GNS = GN["source"]
if not (GNS["listDeclaredInSchema"] and GNS["countMatchesList"] and GNS["listNoLongerDeclaredInGuards"] and GNS["guardsReExports"]
        and GNS["guardNamesOnTheLock"] and GNS["lockCommentNamesSprint"] and GNS["dispatchTest"] and GNS["surfaceTest"] and GNS["censusReadsSchema"]
        and GNS["viewReadsNoCrateAbove"] and GNS["moduleHomeKnowsGuardNames"]):
    sys.exit(f"refusing: the list, the lock, the re-export, the census read or the member's read set is not in source as the page says: {GNS}")
# sprint 737 made `view` a wrapper module in keel-cli (it globs keel_view::view and adds the three item views), so four
# modules are re-exported by `pub use` and the fifth is wrapped: the count the view tab stands on is four plus that wrapper.
_VIEW_WRAPPED = 1 if v("issuesReadsTheModelAndTheWriteApi")["source"]["cliViewWrapperAddsTheThree"] else 0
if len(GNS["cliReExports"]) + _VIEW_WRAPPED != 5 or len(GNS["viewTopModules"]) != 5 or GNS["viewSubmoduleCount"] != 10:
    sys.exit(f"refusing: the page says five re-exported modules (one wrapped since sprint 737) and ten view submodules: {GNS['cliReExports']} wrapped={_VIEW_WRAPPED} {GNS['viewTopModules']} {GNS['viewSubmodules']}")
N_GUARDS, N_LOCK_PATHS, N_MEMBERS = GNS["declaredCount"], len(GNS["lockPaths"]), GNS["memberCount"]
N_VIEW_FILES, N_VIEW_LINES, N_VIEW_SUB = GNS["viewRsFiles"], GNS["viewRsLines"], GNS["viewSubmoduleCount"]
VIEW_DEPS = GNS["viewDependsOn"]
GNL = GN["landed"]
if GNL is None or GNL["renames"] == 0 or GNL["deleted"] != 0:
    sys.exit(f"refusing: the page says the view moved by rename and nothing was deleted: {GNL}")
N_RENAMES, N_MODIFIED, N_ADDED, N_FILES_CHANGED, N_INS, N_DEL = GNL["renames"], GNL["modified"], GNL["added"], GNL["filesChanged"], GNL["insertions"], GNL["deletions"]
GND = GN["declared"]
if not (GND["guardsDocNamesTheFile"] and GND["processNamesRenderer"] and not GND["processNamesOldPath"] and GND["skillNamesRenderer"] and GND["claudeCopyAgrees"]):
    sys.exit(f"refusing: guards.md, the stpa-diagram process or its skill does not name the new homes as the page says: {GND}")
if GND["registryViewCorePath"] != "members/keel-view/src/view/mod.rs" or GND["registryArchMemberPath"] != "members/keel-view/src/arch.rs" or not GND["registryArchSupersedes"]:
    sys.exit(f"refusing: the code registry does not follow the files as the page says: {GND}")
GNV = GN["live"]
if not (GNV["version"]["exit0"] and GNV["version"]["matchesTheList"] and GNV["version"]["guards"] == N_GUARDS):
    sys.exit(f"refusing: keel version's guards line does not equal the declared list: {GNV['version']}")
if not (GNV["processChange"]["exit0"] and GNV["processChange"]["verdict"] == "PASS"):
    sys.exit(f"refusing: the page says the lock is green on this tree: {GNV['processChange']}")
for _n, _i, _sev in (("579", GN["issue579"], "Medium"), ("580", GN["issue580"], "Medium"), ("581", GN["issue581"], "Medium")):
    if not (_i["exists"] and _i["resolverAsExpected"] and _i["severity"] == _sev):
        sys.exit(f"refusing: issue{_n}, its resolver edge or its severity is not as the page says: {_i}")
if not (GN["issue579"]["resolverDodNamesIssue"] and GN["issue580"]["resolverNamedByIssue"] and GN["issue581"]["resolverDodNamesIssue"]):
    sys.exit("refusing: each of the three issues must be named by its resolver or name it (the issues guard's rule)")
SP2 = GN["sprint732"]
if not (SP2["exists"] and SP2["chartersAsExpected"] and SP2["retroNamesIssue581"] and SP2["retroNamesModgraphVacuous"] and SP2["retroNamesAnchorControlFired"]
        and SP2["retroNamesD0486Pattern"] and SP2["dodSaysHelpByteIdentical"] and SP2["dodSaysGuardsUnchanged"]):
    sys.exit(f"refusing: sprint 732 must exist, be chartered by D0479 and carry the retro findings and DoD claims the page quotes: {SP2}")
N_RESULTS2, N_GATE_RESULTS2, N_RETRO2, PTS2 = SP2["results"], SP2["gateResults"], SP2["retroFindings"], SP2["estimatedPoints"]
GNP = GN["resolverPositions"]
if any(GNP[a] is None for a in GNP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {GNP}")
POS_SUITE2, POS_FAMILIES, POS_LAYERING, POS_QUOTED, POS_ROOT = (GNP[a]["place"] for a in
    ("dcSuiteIsAMember", "dcGuardsAreMembersPerFamily", "dcWorkspaceLayeringIsGuarded", "dcQuotedProbeLineIsRefused", "dcOneRepoRootHelper"))
N_BACKLOG_GN = GN["backlogItems"]

# -- ask 11 (first tab): a locked guard source's test anchor repoints through keel-fs, and the lock fired for it --
LA = v("lockedTestAnchorRepointsThroughKeelFs")
if LA["status"] != "proposed" or LA["marker"] != "#ProspectiveChange" or LA["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (locked anchor): {LA['status']}, {LA['marker']}, {LA['acceptance']}")
if not (LA["dependsOnD0479"] and LA["dependsOnD0504"] and LA["notAFork"] is False):
    # the record weighs no alternative and says so by carrying none; it stands on D0479 and on the lock's current shape, D0504
    sys.exit(f"refusing: the locked-anchor record must stand on D0479 and D0504: {LA['dependsOnD0479']} {LA['dependsOnD0504']}")
for _k in ("namesBreaksTwoLevelsDown", "namesSecondDoorClosed", "namesOneLineOfTestCode", "namesLogicUnchanged", "namesSevenOtherAnchors", "namesScanWidens",
           "namesLockOnTheFile", "namesNoException", "namesD0388Class", "namesFirstTestOnlyFire", "namesWorkingAsSpecified", "namesCfgTestForkIsSeparate"):
    if not LA[_k]:
        sys.exit(f"refusing: the locked-anchor record does not say what the page quotes it saying: {_k}")
LAS = LA["source"]
if not (LAS["helperFileExists"] and LAS["helperIsPub"] and LAS["helperDocHidden"] and LAS["helperWalksToGit"] and LAS["helperHasOwnTest"] and LAS["libExportsTestSupport"]
        and LAS["oneDefinitionInKeelFs"] and LAS["adherenceOnTheLockList"] and LAS["adherenceTestFound"] and LAS["adherenceTestCallsHelper"] and LAS["adherenceTestHoldsEmptyPrefix"]
        and LAS["scanTestFound"] and LAS["scanReadsTestBearingSources"] and LAS["scanExcludesAlwaysTest"] and LAS["scanAssertsCliInPopulation"] and LAS["scanAssertsPopulationSize"]
        and LAS["scanNamesHelperInMessage"] and LAS["collapseScriptDeclaresItself"] and LAS["collapseScriptIdempotent"]):
    sys.exit(f"refusing: the helper, its one definition, the locked test's call, the widened scan or the codemod's declaration is not in source as the page says: {LAS}")
if LAS["anchorsInCliSrc"] or LAS["anchorsInMemberSrc"]:
    sys.exit(f"refusing: the page says no cwd-relative anchor is left in keel-cli/src or any member's src: {LAS['anchorsInCliSrc']} {LAS['anchorsInMemberSrc']}")
if not all(LAS["devDepKeelFs"].values()):
    sys.exit(f"refusing: the page says keel-github, keel-schema and keel-actor each carry keel-fs as a dev-dependency: {LAS['devDepKeelFs']}")
N_USE_FILES, N_CLI_CALLERS, N_TEST_ANCHORS = LAS["useLineCount"], LAS["cliCallerCount"], len(LAS["anchorsInCliTests"])
N_WS_MEMBERS_LA = LAS["workspaceMemberCount"]
LAL = LA["landed"]
if LAL is None or LAL["adherenceInsertions"] != 1 or LAL["adherenceDeletions"] != 1 or not LAL["adherenceDiffIsTheRepoint"]:
    sys.exit(f"refusing: the page says the locked file's whole diff is one line out, one line in, and that line is the repoint: {LAL}")
if LAL["lockedFilesTouched"] != ["keel-cli/src/adherence.rs"] or not LAL["newDecisionInRange"] or not LAL["sprintInRange"]:
    sys.exit(f"refusing: the page says adherence.rs is the only locked file in the range and the Decision and sprint landed with it: {LAL}")
N_FILES4, N_MODIFIED4, N_ADDED4, N_INS4, N_DEL4 = (LAL[k] for k in ("filesChanged", "modified", "added", "insertions", "deletions"))
LAV = LA["live"]
if not (LAV["processChange"]["exit0"] and LAV["processChange"]["verdict"] == "PASS" and LAV["processChange"]["violations"] == 0):
    sys.exit(f"refusing: the page says the lock is green on this tree: {LAV['processChange']}")
LR4 = LAV["landingReceipt"]
LANDING4_IS_OWN = LR4["exists"] and LR4["head"] == LAL["range"][1] and LR4["outcome"] == "pass"
if not LANDING4_IS_OWN and not LATER_RUN_STANDS:
    sys.exit(f"refusing: the receipt is neither sprint 734's landing nor a later empty run beside 735's log: {LR4}")
N_LANDED_TESTS4, N_LANDED_FAILED4 = (int(LR4["passed"]), int(LR4["failed"])) if LANDING4_IS_OWN else (LAST_GREEN_TESTS, LAST_GREEN_FAILED)
LANDING4_ROW = f"{N_LANDED_TESTS4} tests, {N_LANDED_FAILED4} failing" if LANDING4_IS_OWN else f"{LAST_GREEN_SPRINT}: {N_LANDED_TESTS4} tests, {N_LANDED_FAILED4} failing"
LANDING4_TRUE = f"{N_LANDED_TESTS4} tests green" if LANDING4_IS_OWN else f"the receipt is now {LAST_GREEN_SPRINT}, {N_LANDED_TESTS4} green"
I557 = LA["issue557"]
# the resolver is the #Resolves edge (resolution is computed from it, D0047); the DoD's prose names the recurrence, not the number
if not (I557["exists"] and I557["resolverAsExpected"]):
    sys.exit(f"refusing: issue557 or its resolver edge is not as the page says: {I557}")
SP4 = LA["sprint734"]
if not (SP4["exists"] and SP4["chartersAsExpected"] and SP4["retroScansAvoidable"] and SP4["retroNamesGluedDocs"] and SP4["retroNamesStaleCounts"]
        and SP4["retroNamesCliTestsOutOfScope"] and SP4["retroNamesProbeFalseStart"] and SP4["retroNamesScanNameKept"] and SP4["retroNoNewItemCount"] == SP4["retroFindings"]):
    sys.exit(f"refusing: sprint 734 must exist, be chartered by D0479 and carry the retro findings the page quotes, each justified: {SP4}")
if not (SP4["dodResults"] and SP4["dodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the story's DoD result passed: {SP4['dodResults']}")
N_RESULTS4, N_GATE_RESULTS4, N_RETRO4, PTS4 = SP4["results"], SP4["gateResults"], SP4["retroFindings"], SP4["estimatedPoints"]
LAP = LA["resolverPositions"]
if any(LAP[a] is None for a in LAP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {LAP}")
POS_ROOT4, POS_SUITE4, POS_PROCESS4 = (LAP[a]["place"] for a in ("dcOneRepoRootHelper", "dcSuiteIsAMember", "dcProcessIsAMember"))
N_BACKLOG_LA = LA["backlogItems"]

# -- ask 12 (first tab): the build-and-test tooling is a member that depends on no view and no guard; two descents out of the lock --
SM = v("suiteDependsOnNoViewOrGuard")
if SM["status"] != "proposed" or SM["marker"] != "#ProspectiveChange" or SM["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (suite member): {SM['status']}, {SM['marker']}, {SM['acceptance']}")
if not (SM["dependsOnD0479"] and SM["dependsOnD0504"] and SM["notAForkInConsequences"]):
    # the record weighs no alternative and says so in its consequences; it stands on D0479 and on the lock's current shape, D0504
    sys.exit(f"refusing: the suite-member record must stand on D0479 and D0504 and say it is not a fork: {SM['dependsOnD0479']} {SM['dependsOnD0504']} {SM['notAForkInConsequences']}")
for _k in ("namesCannotBothHold", "namesFirstDoor", "namesProbeIsTheContract", "namesOneLineUnchanged", "namesNoViewNoGuard", "namesNoGuardLogicChanges",
           "namesHeldToTheProbe", "namesMisfiledNotShared", "namesLockOnThePath", "namesNeverAGuard", "namesByteForByte", "namesWrongIf"):
    if not SM[_k]:
        sys.exit(f"refusing: the suite-member record does not say what the page quotes it saying: {_k}")
SMS = SM["source"]
if not (SMS["memberExists"] and SMS["memberDeclaresAllFive"] and SMS["memberDependsOnNoViewGuardOrWrite"] and SMS["cliReexportsAllFive"] and SMS["cliManifestDependsOnSuite"]
        and SMS["suiteListedBeforeCli"] and SMS["guardReceiptReexportsForced"] and SMS["fsxDefinesPredicate"] and SMS["fsxPredicateReadsFlagAndEnv"] and SMS["fsxPredicateHasOwnTests"]
        and SMS["codemodDeclaresItself"] and SMS["codemodIdempotent"] and SMS["codemodHasApplyFlag"] and SMS["censusFound"] and SMS["censusWalksCliSrc"]
        and SMS["censusWalksEveryMember"] and SMS["censusAssertsPopulation"] and SMS["scanAssertsOwnSrcInPopulation"] and SMS["suiteRunsCliManifestOnly"]):
    sys.exit(f"refusing: the member, its five modules, its read set, keel-cli's re-exports, the predicate's new home, the codemod or the census is not in source as the page says: {SMS}")
if SMS["cliStillHoldsModules"] or SMS["guardLibDeclaresContentkey"] or SMS["guardReceiptDefinesForced"] or SMS["deliverablePathsNameMembers"]:
    sys.exit(f"refusing: the page says nothing of the five is left in keel-cli/src, the guard member neither declares contentkey nor defines forced, and the suite's paths name no member: "
             f"{SMS['cliStillHoldsModules']} {SMS['guardLibDeclaresContentkey']} {SMS['guardReceiptDefinesForced']} {SMS['deliverablePaths']}")
if len(SMS["memberPathDeps"]) != 5 or len(SMS["memberModules"]) != 6 or len(SMS["memberCrateDeps"]) != 2:
    sys.exit(f"refusing: the page says five path dependencies, six modules (five plus lib) and two crates: {SMS['memberPathDeps']} {SMS['memberModules']} {SMS['memberCrateDeps']}")
N_PATH_DEPS, N_CRATE_DEPS, N_WS_MEMBERS_SM = len(SMS["memberPathDeps"]), len(SMS["memberCrateDeps"]), SMS["workspaceMemberCount"]
SML = SM["landed"]
if SML is None or not SML["lockedDiffIsTheTwoDescents"] or SML["deleted"] != 0 or len(SML["movedWhole"]) != 2:
    sys.exit(f"refusing: the page says the locked files' whole diff is the two descents, nothing was deleted and two files moved whole: {SML}")
if sorted(SML["lockedFilesTouched"]) != sorted(["members/keel-guards/src/contentkey.rs", "members/keel-guards/src/lib.rs", "members/keel-guards/src/receipt.rs"]):
    sys.exit(f"refusing: the page says three locked files are in the range - the moved key, lib.rs and receipt.rs: {SML['lockedFilesTouched']}")
if not (SML["newDecisionInRange"] and SML["sprintInRange"] and SML["codemodInRange"]):
    sys.exit(f"refusing: the page says the Decision, the sprint and the codemod landed in the range: {SML}")
_LIB06, _REC06 = SML["lockedFileNumstat"]["members/keel-guards/src/lib.rs"], SML["lockedFileNumstat"]["members/keel-guards/src/receipt.rs"]
if _LIB06["insertions"] != 0 or _LIB06["deletions"] != 1 or _REC06["insertions"] != 3 or _REC06["deletions"] != 6:
    sys.exit(f"refusing: the page says lib.rs lost one line and receipt.rs lost six and gained three: {SML['lockedFileNumstat']}")
N_FILES5, N_RENAMES5, N_MODIFIED5, N_ADDED5, N_INS5, N_DEL5 = (SML[k] for k in ("filesChanged", "renames", "modified", "added", "insertions", "deletions"))
N_LOCKED_OUT, N_LOCKED_IN = len(SML["lockedRemovedLines"]), len(SML["lockedAddedLines"])   # the lines themselves are the fact; the page shows their count
SMV = SM["live"]
if not (SMV["processChange"]["exit0"] and SMV["processChange"]["verdict"] == "PASS" and SMV["processChange"]["violations"] == 0):
    sys.exit(f"refusing: the page says the lock is green on this tree: {SMV['processChange']}")
if SMV["guards"]["total"] != N_GUARDS:
    sys.exit(f"refusing: the page says the guard count is unchanged by the two descents: {SMV['guards']} vs {N_GUARDS}")
LR5, LLOG5 = SMV["landingReceipt"], SMV["landingLog"]
LANDING5_IS_OWN = LR5["exists"] and LR5["head"] == SML["range"][1] and LR5["outcome"] == "pass"
if LANDING5_IS_OWN:
    N_LANDED_TESTS5, N_LANDED_FAILED5, N_STEMS5 = int(LR5["passed"]), int(LR5["failed"]), len(LR5["stems"])
    LANDED_ROW5 = f"{N_LANDED_TESTS5} tests, {N_LANDED_FAILED5} failing, {N_STEMS5} stems"
elif LLOG5 and LLOG5["logsInWindow"] >= 1 and LLOG5["failed"] == 0 and LLOG5["run"] == LLOG5["passed"] and LLOG5["skipped"] == 0:
    # the landing's receipt was overwritten by a later run; the run itself is read from its log in the landing's window (landingLog)
    N_LANDED_TESTS5, N_LANDED_FAILED5, N_STEMS5 = LLOG5["passed"], LLOG5["failed"], LLOG5["binaries"]
    LANDED_ROW5 = f"{N_LANDED_TESTS5} green from its log, {N_LANDED_FAILED5} failing"
else:
    sys.exit(f"refusing: the receipt is not sprint 735's landing and its log is not a green run in its window: {LR5} log={LLOG5}")
for _n, _i in (("588", SM["issue588"]), ("589", SM["issue589"])):
    # the resolver is the #Resolves edge (resolution is computed from it, D0047)
    if not (_i["exists"] and _i["resolverAsExpected"]):
        sys.exit(f"refusing: issue{_n} or its resolver edge is not as the page says: {_i}")
SP5 = SM["sprint735"]
if not (SP5["exists"] and SP5["chartersAsExpected"] and SP5["retroScansAvoidable"] and SP5["retroNamesSuiteMeasuresCliAlone"] and SP5["retroNamesProseVsProbe"]
        and SP5["retroNamesSecondFiring"] and SP5["retroNamesThirdFiringTrigger"] and SP5["retroNamesCodemodCheck"] and SP5["retroNamesProbeSettles"] and SP5["retroNamesStemsRow"]):
    sys.exit(f"refusing: sprint 735 must exist, be chartered by D0479 and carry the retro findings the page quotes: {SP5}")
if not (SP5["storyDodResults"] and SP5["storyDodResults"][-1]["outcome"] == "pass" and SP5["itemDodResults"] and SP5["itemDodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the story's and the item's DoD results both passed: {SP5['storyDodResults']} {SP5['itemDodResults']}")
N_RESULTS5, N_GATE_RESULTS5, N_RETRO5, N_RETRO_NOT_TRACKED5, PTS5 = SP5["results"], SP5["gateResults"], SP5["retroFindings"], SP5["retroNotTrackedCount"], SP5["estimatedPoints"]
SMP = SM["resolverPositions"]
if any(SMP[a] is None for a in SMP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {SMP}")
POS_PROCESS5, POS_ISSUES5, POS_MEASURES5, POS_DELIVERS5, POS_THIN5 = (SMP[a]["place"] for a in
    ("dcProcessIsAMember", "dcKeelIssuesIsAMember", "dcSuiteMeasuresTheWorkspace", "dcSprintNamesTheItemItDelivers", "dcKeelCliIsThinDispatch"))
N_BACKLOG_SM = SM["backlogItems"]

# -- ask 13 (first tab): the item member reads the model and the write API and nothing above; the predicate descends out of the lock; one D0480 clause reversed --
IM = v("issuesReadsTheModelAndTheWriteApi")
if IM["status"] != "proposed" or IM["marker"] != "#ProspectiveChange" or IM["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (issues member): {IM['status']}, {IM['marker']}, {IM['acceptance']}")
if not (IM["supersedesClauseOfD0480"] and not IM["supersedesWholeD0480"] and IM["notAForkInConsequences"]):
    # one clause reversed, the target in force (D0398: never both edges); the record weighs no alternative and says so in its consequences
    sys.exit(f"refusing: the issues-member record must reverse one clause of the charter and say it is not a fork: {IM['supersedesClauseOfD0480']} {IM['supersedesWholeD0480']} {IM['notAForkInConsequences']}")
for _k in ("namesThreeThingsUnsaid", "namesLockedPathMarker", "namesNoCrateAbove", "namesCannotDisagree", "namesPathNotHome", "namesNeverRestatedInSynopsis",
           "namesRestOfD0480Stands", "namesReadersRule", "namesTwoVocabulariesClass", "namesOneHomePerFact", "namesHumanJudgesNotTree", "namesDescentNotDependency",
           "namesByteIdentical", "namesGuardLineUnchanged"):
    if not IM[_k]:
        sys.exit(f"refusing: the issues-member record does not say what the page quotes it saying: {_k}")
IM480 = IM["d0480"]
if not (IM480 and IM480["status"] == "accepted" and IM480["acceptedUnderStandingConsent"] and IM480["clauseText"] and IM480["clauseIsInConsequences"]):
    sys.exit(f"refusing: the page says the charter is accepted under standing consent and carries the reversed clause in its consequences: {IM480}")
IMS = IM["source"]
if not (IMS["memberExists"] and IMS["memberDeclaresAllFour"] and IMS["memberReexportsAddTask"] and IMS["memberDependsOnNoGovernance"] and IMS["memberHasNoBuildRs"]
        and IMS["memberDefinesTriage"] and IMS["memberDefinesRecordIssue"] and IMS["cliReexportsGithubIngest"] and IMS["cliReexportsIntakeWrite"] and IMS["cliReexportsIssues"]
        and IMS["cliViewWrapperAddsTheThree"] and IMS["cliWriteWrapperAddsTheIssueWrite"] and IMS["mainCallsTriageHolds"] and IMS["cliManifestDependsOnIssues"]
        and IMS["issuesListedBeforeCli"] and IMS["resolversDefinesBoth"] and IMS["modelDeclaresResolvers"] and IMS["queriesDefinesAtLeastMedium"]
        and IMS["guardLibReexportsDeclared"] and IMS["guardIssuesReexportsKindHolds"] and IMS["guardIssuesStillAppliesIt"]
        and IMS["codemodDeclaresItself"] and IMS["codemodIdempotent"] and IMS["codemodHasApplyFlag"]):
    sys.exit(f"refusing: the member, its four modules, its read set, keel-cli's wrappers, the predicate's new home, the guard's re-exports or the codemod is not in source as the page says: {IMS}")
if IMS["cliStillHoldsGithubIngest"] or IMS["writeStillHoldsIntakeWrite"] or IMS["writeStillDefinesRecordIssue"] or IMS["viewStillDefinesOpenIssues"] or IMS["guardLibDefinesDeclared"] or IMS["guardIssuesDefinesKindHolds"]:
    sys.exit(f"refusing: the page says nothing moved is left at its old home and the guard member defines neither predicate: {IMS}")
if len(IMS["memberPathDeps"]) != 4 or len(IMS["memberModules"]) != 5 or len(IMS["memberCrateDeps"]) != 1:
    sys.exit(f"refusing: the page says four path dependencies, five modules (four plus lib) and one crate: {IMS['memberPathDeps']} {IMS['memberModules']} {IMS['memberCrateDeps']}")
N_PATH_DEPS8, N_CRATE_DEPS8, N_WS_MEMBERS_IM = len(IMS["memberPathDeps"]), len(IMS["memberCrateDeps"]), IMS["workspaceMemberCount"]
IML = IM["landed"]
if IML is None or not IML["lockedDiffIsTheTwoDescents"] or IML["deleted"] != 0 or len(IML["renamedFrom"]) != 2:
    sys.exit(f"refusing: the page says the locked files' whole diff is the two descents, nothing was deleted and two files moved: {IML}")
if sorted(IML["lockedFilesTouched"]) != sorted(["members/keel-guards/src/issues.rs", "members/keel-guards/src/lib.rs"]):
    sys.exit(f"refusing: the page says two locked files are in the range - lib.rs and issues.rs: {IML['lockedFilesTouched']}")
if not (IML["newDecisionInRange"] and IML["sprintInRange"] and IML["codemodInRange"] and IML["resolversInRange"]):
    sys.exit(f"refusing: the page says the Decision, the sprint, the codemod and the predicate's new file landed in the range: {IML}")
_LIB08, _ISS08 = IML["lockedFileNumstat"]["members/keel-guards/src/lib.rs"], IML["lockedFileNumstat"]["members/keel-guards/src/issues.rs"]
N_FILES8, N_RENAMES8, N_MODIFIED8, N_ADDED8, N_INS8, N_DEL8 = (IML[k] for k in ("filesChanged", "renames", "modified", "added", "insertions", "deletions"))
N_LOCKED_OUT8, N_LOCKED_IN8 = len(IML["lockedRemovedLines"]), len(IML["lockedAddedLines"])   # the lines themselves are the fact; the page shows their count
IMV = IM["live"]
if not (IMV["processChange"]["exit0"] and IMV["processChange"]["verdict"] == "PASS" and IMV["processChange"]["violations"] == 0):
    sys.exit(f"refusing: the page says the lock is green on this tree: {IMV['processChange']}")
if IMV["guards"]["total"] != N_GUARDS:
    sys.exit(f"refusing: the page says the guard count is unchanged by the descent: {IMV['guards']} vs {N_GUARDS}")
LR8, LLOG8 = IMV["landingReceipt"], IMV["landingLog"]
LANDING8_IS_OWN = LR8["exists"] and LR8["head"] == IML["range"][1] and LR8["outcome"] == "pass"
if LANDING8_IS_OWN:
    N_LANDED_TESTS8, N_LANDED_FAILED8, N_STEMS8 = int(LR8["passed"]), int(LR8["failed"]), len(LR8["stems"])
    LANDED_ROW8 = f"{N_LANDED_TESTS8} tests, {N_LANDED_FAILED8} failing, {N_STEMS8} stems"
elif LLOG8 and LLOG8["logsInWindow"] >= 1 and LLOG8["failed"] == 0 and LLOG8["run"] == LLOG8["passed"] and LLOG8["skipped"] == 0:
    # sprint 738's landing overwrote 737's receipt (one deep); 737's run is read from its log in its landing's window
    N_LANDED_TESTS8, N_LANDED_FAILED8, N_STEMS8 = LLOG8["passed"], LLOG8["failed"], LLOG8["binaries"]
    LANDED_ROW8 = f"{N_LANDED_TESTS8} green from its log, {N_LANDED_FAILED8} failing"
else:
    sys.exit(f"refusing: the receipt is not sprint 737's landing and its log is not a green run in its window: {LR8} log={LLOG8}")
SP8 = IM["sprint737"]
if not (SP8["exists"] and SP8["chartersAsExpected"] and SP8["retroScansAvoidable"] and SP8["retroNamesFourthSprintRunning"] and SP8["retroNamesThirdHeld"]
        and SP8["retroNamesControlFiredAsDesigned"] and SP8["retroNamesProbeNarrowed"] and SP8["retroNamesThreeFlagHelpers"] and SP8["retroNamesHomeIsComputed"]):
    sys.exit(f"refusing: sprint 737 must exist, be chartered by the item member's Decision and carry the retro findings the page quotes: {SP8}")
if not (SP8["storyDodResults"] and SP8["storyDodResults"][-1]["outcome"] == "pass" and SP8["itemDodResults"] and SP8["itemDodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the story's and the item's DoD results both passed: {SP8['storyDodResults']} {SP8['itemDodResults']}")
N_RESULTS8, N_GATE_RESULTS8, N_RETRO8, N_RETRO_NO_ITEM8, N_RETRO_NOT_TRACKED8, PTS8 = (SP8["results"], SP8["gateResults"], SP8["retroFindings"], SP8["retroNoNewItemCount"],
                                                                                        SP8["retroNotTrackedCount"], SP8["estimatedPoints"])
IMP = IM["resolverPositions"]
if any(IMP[a] is None for a in IMP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {IMP}")
POS_ISSUES8, POS_TOUCHED8, POS_MEASURES8, POS_CITATIONS8, POS_DELIVERS8, POS_SERVE8, POS_THIN8 = (IMP[a]["place"] for a in
    ("dcKeelIssuesIsAMember", "dcTouchedSetDescendsTheWorkspace", "dcSuiteMeasuresTheWorkspace", "dcSourceCitationsOnTheLivingDocsResolve",
     "dcSprintNamesTheItemItDelivers", "dcServeAndGithubAreMembers", "dcKeelCliIsThinDispatch"))
N_BACKLOG_IM = IM["backlogItems"]

# -- ask 14 (first tab): the console is member keel-serve, above every member and below the binary; three locked path strings follow their files --
CN = v("serveSitsAboveEveryMemberAndBelowTheBinary")
if CN["status"] != "proposed" or CN["marker"] != "#ProspectiveChange" or CN["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (serve member): {CN['status']}, {CN['marker']}, {CN['acceptance']}")
if not (CN["dependsOn"] == ["d0479"] and not CN["supersedesAnything"] and CN["notAForkInConsequences"]):
    # the record stands on the extraction charter alone, reverses nothing, and says in its consequences that it weighs no alternative
    sys.exit(f"refusing: the serve-member record must stand on D0479 alone, reverse nothing and say it is not a fork: {CN['dependsOn']} {CN['supersedesAnything']} {CN['notAForkInConsequences']}")
for _k in ("namesFourThingsUnsaid", "namesThreeLockedPathStrings", "namesNoMemberBelowCanHold", "namesCliAlone", "namesNoMemberNamesIt", "namesGuardReadsWhereManifestsPut",
           "namesOrientIsReadiness", "namesReadersRule", "namesTopOfGraph", "namesTwoVocabulariesClass", "namesHumanJudges", "namesFailClosed",
           "namesByteIdentical", "namesGuardLineUnchanged", "namesLastExtraction", "namesIssue593"):
    if not CN[_k]:
        sys.exit(f"refusing: the serve-member record does not say what the page quotes it saying: {_k}")
CNS = CN["source"]
if not (CNS["memberExists"] and CNS["memberDeclaresAllSix"] and CNS["memberHoldsConsoleHtml"] and CNS["memberEmbedsConsoleHtml"] and CNS["memberEmbedsMainRs"]
        and CNS["memberHasLintPreamble"] and CNS["memberDependsOnEveryMember"] and CNS["memberHasNoBuildRs"] and CNS["noMemberNamesServe"] == []
        and len(CNS["cliReexportsTheSix"]) == 6 and CNS["cliReexportsCiRuns"] and CNS["cliReexportsVerification"] and CNS["cliReexportsOrientNames"]
        and CNS["cliManifestDependsOnServe"] and CNS["serveListedLastBeforeCli"] and CNS["githubDependsOnTheThree"] and CNS["githubDeclaresCiRuns"]
        and CNS["githubHoldsCiRuns"] and CNS["viewDeclaresVerification"] and CNS["viewHoldsVerification"] and CNS["modelDeclaresReadiness"]
        and CNS["readinessDefinesTheFour"] and CNS["hardeningReadsNewPaths"] >= 2 and CNS["processCitesNewDeck"] and CNS["registryCitesNewDeck"]
        and CNS["codemodDeclaresItself"] and CNS["codemodIdempotent"] and CNS["codemodHasApplyFlag"]):
    sys.exit(f"refusing: the member, its six modules and page, its read set, keel-cli's re-exports and manifest, the three new homes, the lens's paths, the two citations or the codemod is not in source as the page says: {CNS}")
if CNS["cliStillHoldsAny"] or CNS["cliStillHoldsConsoleHtml"] or CNS["cliLibStillDefinesOrient"] or CNS["cliNormalDepsStillName"] or CNS["cliDevDepsStillNameTower"] or CNS["hardeningReadsOldPaths"]:
    sys.exit(f"refusing: the page says nothing moved is left at its old home, keel-cli's manifest dropped the five and the lens reads no old path: {CNS}")
if len(CNS["memberPathDeps"]) != 11 or len(CNS["memberModules"]) != 7 or len(CNS["memberCrateDeps"]) != 7 or len(CNS["memberDevDeps"]) != 2:
    sys.exit(f"refusing: the page says eleven path dependencies, seven modules (six plus lib), seven crates and two dev-deps: {CNS['memberPathDeps']} {CNS['memberModules']} {CNS['memberCrateDeps']} {CNS['memberDevDeps']}")
N_PATH_DEPS38, N_CRATE_DEPS38, N_WS_MEMBERS_CN = len(CNS["memberPathDeps"]), len(CNS["memberCrateDeps"]), CNS["workspaceMemberCount"]
CNL = CN["landed"]
if CNL is None or not CNL["lockedDiffIsPathStringsOnly"] or CNL["deleted"] != 0 or len(CNL["renamedFrom"]) != 9:
    sys.exit(f"refusing: the page says the locked files' whole diff is path strings following their files, nothing was deleted and nine files moved: {CNL}")
if sorted(CNL["lockedFilesTouched"]) != sorted(["members/keel-guards/src/hardening.rs", ".engine/processes/obligation-review.sysml", ".engine/skills/obligation-review/registry.sysml"]):
    sys.exit(f"refusing: the page says three locked files are in the range - the lens, the process, the registry: {CNL['lockedFilesTouched']}")
if not (CNL["newDecisionInRange"] and CNL["sprintInRange"] and CNL["codemodInRange"] and CNL["readinessInRange"] and CNL["memberManifestInRange"]):
    sys.exit(f"refusing: the page says the Decision, the sprint, the codemod, the orient module and the member manifest landed in the range: {CNL}")
N_FILES38, N_RENAMES38, N_MODIFIED38, N_ADDED38, N_INS38, N_DEL38 = (CNL[k] for k in ("filesChanged", "renames", "modified", "added", "insertions", "deletions"))
N_LOCKED_OUT38, N_LOCKED_IN38 = len(CNL["lockedRemovedLines"]), len(CNL["lockedAddedLines"])   # the lines themselves are the fact; the page shows their count
N_MOVED_WHOLE38 = len(CNL["movedWhole"])
CNV = CN["live"]
if not (CNV["processChange"]["exit0"] and CNV["processChange"]["verdict"] == "PASS" and CNV["processChange"]["violations"] == 0):
    sys.exit(f"refusing: the page says the lock is green on this tree: {CNV['processChange']}")
if CNV["guards"]["total"] != N_GUARDS:
    sys.exit(f"refusing: the page says the guard count is unchanged by the extraction: {CNV['guards']} vs {N_GUARDS}")
LR38, LLOG38 = CNV["landingReceipt"], CNV["landingLog"]
LANDING38_IS_OWN = LR38["exists"] and LR38["head"] == CNL["range"][1] and LR38["outcome"] == "pass"
if LANDING38_IS_OWN:
    N_LANDED_TESTS38, N_LANDED_FAILED38, N_STEMS38 = int(LR38["passed"]), int(LR38["failed"]), len(LR38["stems"])
    LANDED_ROW38 = f"{N_LANDED_TESTS38} tests, {N_LANDED_FAILED38} failing, {N_STEMS38} stems"
elif LATER_RUN_STANDS and LLOG38 and LLOG38["logsInWindow"] >= 1 and LLOG38["failed"] == 0 and LLOG38["run"] == LLOG38["passed"] and LLOG38["skipped"] == 0:
    # a later commit's run (empty, or green over its own stems) overwrote 738's receipt; 738's run survives as its log
    N_LANDED_TESTS38, N_LANDED_FAILED38, N_STEMS38 = LLOG38["passed"], LLOG38["failed"], LLOG38["binaries"]
    LANDED_ROW38 = f"{N_LANDED_TESTS38} green from its log, {N_LANDED_FAILED38} failing"
else:
    sys.exit(f"refusing: the receipt is neither sprint 738's landing nor a later commit's run beside 738's green log: {LR38} {LLOG38}")
SP38 = CN["sprint738"]
if not (SP38["exists"] and SP38["chartersAsExpected"] and SP38["retroScansAvoidable"] and SP38["retroNamesFifthSprintRunning"] and SP38["retroNamesFourthHeld"]
        and SP38["retroNamesControlFiredAsDesigned"] and SP38["retroNamesCleanFirstBuild"] and SP38["retroNamesDualTruthEdge"] and SP38["retroNamesStandby"]):
    sys.exit(f"refusing: sprint 738 must exist, be chartered by the extraction charter and carry the retro findings the page quotes: {SP38}")
if not (SP38["storyDodResults"] and SP38["storyDodResults"][-1]["outcome"] == "pass" and SP38["itemDodResults"] and SP38["itemDodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the story's and the item's DoD results both passed: {SP38['storyDodResults']} {SP38['itemDodResults']}")
if SP38["retroTrackedCount"] != 3:
    sys.exit(f"refusing: the page says three retro findings became tracked items: {SP38['retroTrackedCount']}")
N_RESULTS38, N_GATE_RESULTS38, N_RETRO38, N_RETRO_TRACKED38, N_RETRO_NOT_TRACKED38, PTS38 = (SP38["results"], SP38["gateResults"], SP38["retroFindings"], SP38["retroTrackedCount"],
                                                                                        SP38["retroNotTrackedCount"], SP38["estimatedPoints"])
for _n, _i, _sev in (("593", CN["issue593"], "Medium"), ("594", CN["issue594"], "Medium"), ("595", CN["issue595"], "Medium")):
    if not (_i["exists"] and _i["resolverAsExpected"] and _i["severity"] == _sev):
        sys.exit(f"refusing: issue{_n}, its resolver edge or its severity is not as the page says: {_i}")
CNP = CN["resolverPositions"]
if any(CNP[a] is None for a in CNP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {CNP}")
POS_SERVE38, POS_TOUCHED38, POS_MEASURES38, POS_CITATIONS38, POS_DELIVERS38, POS_THIN38, POS_REGISTRY38, POS_LINT38, POS_STANDBY38 = (CNP[a]["place"] for a in
    ("dcServeAndGithubAreMembers", "dcTouchedSetDescendsTheWorkspace", "dcSuiteMeasuresTheWorkspace", "dcSourceCitationsOnTheLivingDocsResolve",
     "dcSprintNamesTheItemItDelivers", "dcKeelCliIsThinDispatch", "dcCodeRegistryPathsResolve", "dcMembersCarryTheLintPreamble", "dcSuiteReceiptNamesTheStandbyItSpanned"))
N_BACKLOG_CN = CN["backlogItems"]

# -- ask 15 (first tab): a sitting review is finished by analysis; the human's word stays on the three per-item verbs --
SR = v("sittingReviewIsFinishedByAnalysisNotConfirmation")
SRD = SR["decision"]
if SRD["status"] != "proposed" or SRD["marker"] != "#ProspectiveChange" or SRD["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (sitting review): {SRD['status']}, {SRD['marker']}, {SRD['acceptance']}")
if not (SRD["supersedesClauseD0049"] and SRD["supersedesClauseD0051"] and SRD["dependsOnD0204"] and SRD["dependsOnD0312"] and SRD["notAForkInConsequences"]):
    sys.exit(f"refusing: the sitting-review record must reverse one clause each of the ceremony and confirm-only charters, stand on pull-oversight and proposed-results, and say it is not a fork: {SRD}")
for _k in ("namesTheHumansWords", "namesD0049Clause3", "namesD0204RetiredTheFraming", "namesTheCoverageNumbers", "namesEightyOldMethod", "namesMethodAnalysis",
           "namesTheThreeVerbs", "namesNoConfirmationFromHere", "namesAssuranceSurface", "namesReceiptNotTestimony", "namesTheRetroPrecedent", "namesD0337Held",
           "namesHistoryNotRewritten", "namesTheGuardToFollow"):
    if not SRD[_k]:
        sys.exit(f"refusing: the sitting-review record does not say what the page quotes it saying: {_k}")
if not (SR["reversedClauses"]["d0049Clause3OnTree"] and SR["reversedClauses"]["d0051ClauseOnTree"]):
    sys.exit(f"refusing: the two clauses the page quotes as reversed must stand verbatim in their records: {SR['reversedClauses']}")
SRT = SR["today"]
if not (SRT["viewpointSurface"] == "act" and SRT["skillSaysTheOneGate"] and SRT["skillRecordShapeIsConfirmation"]):
    sys.exit(f"refusing: the page says the OLD shape stands today - viewpoint on act, the skill asking for the one gate with a confirmation record: {SRT}")
SRC = SR["coverage"]
if SRC is None or not SRC["dueEqualsUncoveredMinusGrandfathered"] or SRC["sprintsCovered"] != SRC["covered"]:
    sys.exit(f"refusing: the coverage numbers must re-derive (due = uncovered - grandfathered; covered = distinct covered stories): {SRC}")
if not (SRC["sprints"] == 738 and SRC["covered"] == 116 and SRC["due"] == 309 and SRC["grandfathered_unreviewed"] == 313 and SRC["sittingReviews"] == 80):
    sys.exit(f"refusing: the record cites 738/116/309/313 and eighty reviews; the lens reads otherwise: {SRC}")
N_SPRINTS_SR, N_COVERED_SR, N_READ_SR, N_BATCH_SR, N_UNCOV_SR, N_DUE_SR, N_GRAND_SR, N_REVIEWS_SR = (SRC[k] for k in
    ("sprints", "covered", "readReviewed", "batchAcknowledgedOnly", "uncovered", "due", "grandfathered_unreviewed", "sittingReviews"))
N_SKILL_CONF_SR = SRT["skillConfirmationMentions"]
_i597 = SR["issue597"]
if not (_i597 and _i597["exists"] and _i597["resolverAsExpected"] and _i597["severity"] == "Medium" and _i597["resolverDodNamesIssue"]):
    sys.exit(f"refusing: the finding, its resolver edge or its severity is not as the page says: {_i597}")
if SR["resolverPosition"] is None or SR["resolverReadyRank"] is None:
    sys.exit(f"refusing: the resolver must be on the backlog and on the ready frontier: {SR['resolverPosition']} {SR['resolverReadyRank']}")
if SR["resolverDodResults"] != [] or not SR["resolverDodNamesTheGuard"]:
    sys.exit(f"refusing: the page says nothing is applied yet and the resolver's DoD names the guard: {SR['resolverDodResults']} {SR['resolverDodNamesTheGuard']}")
POS_SR, RANK_SR, N_READY_SR, N_BACKLOG_SR = SR["resolverPosition"]["place"], SR["resolverReadyRank"], SR["readyItems"], SR["backlogItems"]

# -- ask 10: the guard-source lock names the member directory by prefix --
GP = v("guardSourceLockIsADirectoryPrefix")
if GP["status"] != "proposed" or GP["marker"] != "#ProspectiveChange" or GP["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change (guard source): {GP['status']}, {GP['marker']}, {GP['acceptance']}")
if not (GP["dependsOnD0479"] and GP["dependsOnD0503"] and GP["notAFork"] is False):
    # the record weighs no alternative and says so by carrying none; it stands on D0479 and on the lock entry D0503 made
    sys.exit(f"refusing: the guard-source record must stand on D0479 and D0503: {GP['dependsOnD0479']} {GP['dependsOnD0503']}")
for _k in ("namesFourthExtraction", "namesOldSize", "namesSilentScan", "namesD0388Class", "namesThirdMeeting", "namesFileListLoses", "namesDirsConstant",
           "namesPrefixRule", "namesScansEveryMember", "namesReachedOwnFiles", "namesDispatchReadsTables", "namesUnionTest", "namesWidening",
           "namesLockedByConstruction", "namesCheckFollowsSubject", "namesNextExtractionCovered", "namesEveryFileLocked", "namesCargoNotLocked",
           "namesTwelveMembers", "namesRecordedThreeTimes"):
    if not GP[_k]:
        sys.exit(f"refusing: the guard-source record does not say what the page quotes it saying: {_k}")
GPS = GP["source"]
if not (GPS["oldPathOffTheList"] and GPS["adherenceOnTheList"] and GPS["guardNamesOnTheList"] and GPS["guardsDirIsTheLock"] and GPS["prefixRuleInSource"]
        and GPS["coverageTestFound"] and GPS["coverageReadsTheManifest"] and GPS["coverageScansEveryMember"] and GPS["coverageAssertsTwelve"]
        and GPS["coverageAssertsReachedOwn"] and GPS["coverageNamesBothConstants"] and GPS["lockTestFound"] and GPS["dispatchReadsTables"]
        and GPS["dispatchScansNoText"] and GPS["unionTestModule"] and GPS["unionTestUsesRunnableOnly"] and GPS["oldFileGone"]
        and GPS["moduleHomeHasCrateHelpers"] and GPS["splitScriptDeclaresItself"] and GPS["splitScriptRecordsWrongFix"]):
    sys.exit(f"refusing: the two constants, the prefix rule, the coverage / lock / dispatch tests or the old file's absence is not in source as the page says: {GPS}")
if GPS["uncoveredGuardDefiningFiles"]:
    sys.exit(f"refusing: the page says every guard-defining file is under the lock; these are not: {GPS['uncoveredGuardDefiningFiles']}")
if len(GPS["lockTestLocks"]) != 5 or len(GPS["lockTestFrees"]) != 4 or "keel-cli/src/guards.rs" not in GPS["lockTestFrees"]:
    sys.exit(f"refusing: the page says the lock test holds five paths locked and four free, the old path among the free: {GPS['lockTestLocks']} {GPS['lockTestFrees']}")
if GPS["familyCount"] != 9 or GPS["workspaceMemberCount"] < 12 or GPS["oldFileLinesAtBase"] is None:
    sys.exit(f"refusing: the page says nine families, at least twelve members and a measured old file: {GPS['familyCount']} {GPS['workspaceMemberCount']} {GPS['oldFileLinesAtBase']}")
N_LOCK_FILES, N_LOCK_DIRS, N_FAMILIES = GPS["lockFileCount"], len(GPS["lockDirs"]), GPS["familyCount"]
N_MEMBER_FILES, N_MEMBER_LINES, N_DEFINING = GPS["memberFileCount"], GPS["memberLines"], len(GPS["guardDefiningFilesInMember"])
N_WS_MEMBERS, OLD_LINES = GPS["workspaceMemberCount"], GPS["oldFileLinesAtBase"]
GPL = GP["landed"]
if GPL is None or GPL["renames"] == 0 or GPL["deleted"] != 1:
    sys.exit(f"refusing: the page says the moved modules landed by rename and exactly one file was deleted (the old guards.rs): {GPL}")
N_RENAMES3, N_MODIFIED3, N_ADDED3, N_DELETED3, N_FILES3, N_INS3, N_DEL3 = (GPL[k] for k in ("renames", "modified", "added", "deleted", "filesChanged", "insertions", "deletions"))
GPD = GP["declared"]
if not (GPD["guardsDocGroupsByTier"] and GPD["guardsDocNamesTheMember"] and not GPD["guardsDocNamesOldPath"]):
    sys.exit(f"refusing: guards.md must group by tier (issue584), name the member path and not the old one: {GPD}")
N_DOC_TIERS = 2
GPV = GP["live"]
if not (GPV["version"]["exit0"] and GPV["version"]["guards"] == N_GUARDS):
    sys.exit(f"refusing: keel version's guards line must equal the declared list after the move: {GPV['version']} vs {N_GUARDS}")
if not (GPV["processChange"]["exit0"] and GPV["processChange"]["verdict"] == "PASS" and GPV["processChange"]["violations"] == 0):
    sys.exit(f"refusing: the page says the lock is green on this tree: {GPV['processChange']}")
LR3 = GPV["landingReceipt"]
# the receipt is machine-local and one deep: once a later sprint landed it is that sprint's, and this tab says so rather than quoting it as 733's
LANDING3_IS_OWN = LR3["exists"] and LR3["head"] == GPL["range"][1] and LR3["outcome"] == "pass" and LR3["stemsIncludeInit"]
LANDING3_IS_734 = LR3["exists"] and LR3["head"] == LAL["range"][1] and LR3["outcome"] == "pass"
if not (LANDING3_IS_OWN or LANDING3_IS_734 or LATER_RUN_STANDS):
    sys.exit(f"refusing: the receipt is neither sprint 733's landing, nor 734's, nor a later empty run beside 735's log: {LR3}")
N_LANDED_TESTS, N_LANDED_FAILED = (int(LR3["passed"]), int(LR3["failed"])) if (LANDING3_IS_OWN or LANDING3_IS_734) else (LAST_GREEN_TESTS, LAST_GREEN_FAILED)
_WHOSE3 = "sprint 734's" if LANDING3_IS_734 else LAST_GREEN_SPRINT
LANDING3_ROW = f"{N_LANDED_TESTS} tests, {N_LANDED_FAILED} failing" if LANDING3_IS_OWN else f"{_WHOSE3}: {N_LANDED_TESTS} tests, {N_LANDED_FAILED} failing"
LANDING3_TRUE = f"{N_LANDED_TESTS} tests green at the landing" if LANDING3_IS_OWN else f"now {_WHOSE3}, {N_LANDED_TESTS} green"
for _n, _i, _sev in (("583", GP["issue583"], "Medium"), ("584", GP["issue584"], "Low"), ("585", GP["issue585"], "Low"), ("586", GP["issue586"], "Low"), ("587", GP["issue587"], "Low")):
    if not (_i["exists"] and _i["resolverAsExpected"] and _i["severity"] == _sev):
        sys.exit(f"refusing: issue{_n}, its resolver edge or its severity is not as the page says: {_i}")
if not (GP["issue583"]["resolverNamedByIssue"] and GP["issue584"]["resolverDodNamesIssue"] and GP["issue585"]["resolverDodNamesIssue"]
        and GP["issue586"]["resolverDodNamesIssue"] and GP["issue587"]["resolverNamedByIssue"]):
    sys.exit("refusing: each of the five issues must be named by its resolver or name it (the issues guard's rule)")
SP3 = GP["sprint733"]
if not (SP3["exists"] and SP3["chartersAsExpected"] and SP3["retroScansAvoidable"] and SP3["retroNamesAnchorControlFired"] and SP3["retroNamesRootHelperMoved"]
        and SP3["retroNamesProbeRunnerFired"] and SP3["retroNamesIssues"] == ["583", "584", "585", "586", "587"]):
    sys.exit(f"refusing: sprint 733 must exist, be chartered by D0479 and carry the retro findings the page quotes: {SP3}")
if not (SP3["dodResults"] and SP3["dodResults"][-1]["outcome"] == "pass"):
    sys.exit(f"refusing: the page says the story's DoD result passed: {SP3['dodResults']}")
N_RESULTS3, N_GATE_RESULTS3, N_RETRO3, PTS3 = SP3["results"], SP3["gateResults"], SP3["retroFindings"], SP3["estimatedPoints"]
GPP = GP["resolverPositions"]
if any(GPP[a] is None for a in GPP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {GPP}")
POS_ROOT3, POS_SUITE3, POS_CATALOGUE, POS_BUILDSCRIPT, POS_RANGE = (GPP[a]["place"] for a in
    ("dcOneRepoRootHelper", "dcSuiteIsAMember", "dcGuardsCatalogueNamesTheFamily", "dcBuildScriptHasOneHomeBelowItsUsers", "dcFactsAboutARangeReadTheRange"))
if POS_ROOT3 >= POS_SUITE3:
    sys.exit(f"refusing: the page says the root helper now sits ahead of the suite extraction: {POS_ROOT3} vs {POS_SUITE3}")
N_BACKLOG_GS = GP["backlogItems"]

# -- asks 7 and 8: a verifier's stop is never a block; a subagent is measured from its own start --
SV = v("subagentOwnStartAndVerifierStop")
OS_, VS_ = SV["d0501"], SV["d0502"]
for _n, _d in (("own-start", OS_), ("verifier-stop", VS_)):
    if _d["status"] != "proposed" or _d["marker"] != "#SafetyChange" or _d["acceptance"] is not None:
        sys.exit(f"refusing: the page says a held, unaccepted SAFETY change ({_n}): {_d['status']}, {_d['marker']}, {_d['acceptance']}")
    if not (_d["namesIssue578"] and _d["namesSixBlocks"] and _d["saysSafetyChange"] and _d["saysNotAFork"] and _d["namesWrongIf"]):
        sys.exit(f"refusing: the {_n} record must name the Issue, the six blocks, that it is a safety change, that it is not a fork, and its falsifier")
if not all(OS_[k] for k in ("namesSessionBaseline", "namesAgentFile", "namesNeverOverwrites", "namesFallback", "namesSubagentStartHome")):
    sys.exit("refusing: the own-start record must name the session file, the agent file, never overwriting it, the fallback, and the one registration home")
if not all(VS_[k] for k in ("namesTheInventedFinding", "namesNeverABlock", "namesTheControl", "namesRecorderKeepsBlock", "namesD0424", "dependsOnD0501")):
    sys.exit("refusing: the verifier-stop record must name the invented finding, never a block, the control, the recorder's kept block, the actor rule, and stand on the own-start record")
SVS = SV["source"]
if not all(SVS[k] for k in ("baselinePathFolds", "dispatcherWritesOnFirstFire", "baselineOwnThenSession", "routeIsPure", "verifierArmIsTheType", "verifierArmLedgersRefused",
                            "verifierArmExitsZero", "routeBeforeGate", "recorderRelabelKept", "startEventIsCounted", "guardsLedger", "censusHookEvents")) \
        or len(SVS["routeEnumArms"]) != 4 or len(SVS["tests"]) != 3 or not all(SVS["claudeSurface"].values()):
    sys.exit(f"refusing: the two controls are not in source as the page describes them: {SVS}")
N_ROUTE_ARMS, N_ROUTE_TESTS = len(SVS["routeEnumArms"]), len(SVS["tests"])
SVD = SV["declared"]
if not (SVD["controlEvent"]["found"] and SVD["controlEvent"]["control"] == "ctlTurnBoundaryGate" and SVD["controlEvent"]["record"] == "ledger" and SVD["controlEvent"]["namesD0501"]):
    sys.exit(f"refusing: the subagent-start event is not declared as the page describes: {SVD['controlEvent']}")
if not all(SVD["cliFact"].values()):
    sys.exit(f"refusing: the hook's CLI fact, its supersession edge or its mirror does not name the event: {SVD['cliFact']}")
if not (SVD["controlMap"]["found"] and SVD["controlMap"]["title"] == "hook-rule: verifier:tree-written" and SVD["controlMap"]["dischargesEhz2"]):
    sys.exit(f"refusing: the control map does not declare the verifier's control as the page describes: {SVD['controlMap']}")
N_HOOK_RULES = SVD["controlMap"]["hookRuleTitles"]
if not (all(SVD["process"].values()) and all(SVD["skill"].values()) and all(SVD["claudeMd"].values())):
    sys.exit(f"refusing: the process step, the skill (or its .claude copy) or the working rules do not name both controls: {SVD['process']} {SVD['skill']} {SVD['claudeMd']}")
SVL = SV["live"]
if not (SVL["start"]["exit"] == 0 and SVL["start"]["silent"] and SVL["start"]["wroteOwnFile"] and SVL["start"]["fileNameFoldsTheSlash"].startswith("agent-facts_") and SVL["start"]["fingerprintLength"] > 0):
    sys.exit(f"refusing: the page says a start fire writes the agent's own file, silently, with the separator folded: {SVL['start']}")
if not (SVL["secondFireLeavesIt"]["exit"] == 0 and SVL["secondFireLeavesIt"]["unchanged"]):
    sys.exit(f"refusing: the page says a later fire never overwrites the start file: {SVL['secondFireLeavesIt']}")
if not (SVL["stopUnmoved"]["exit"] == 0 and SVL["stopUnmoved"]["silent"]):
    sys.exit(f"refusing: the page says a verifier over its unmoved start pays nothing: {SVL['stopUnmoved']}")
if not (SVL["stopMoved"]["exit"] == 0 and SVL["stopMoved"]["namesControl"] and SVL["stopMoved"]["isSystemMessage"] and SVL["stopMoved"]["saysNotABlock"]):
    sys.exit(f"refusing: the page says a verifier over a moved start is exit 0 with one message naming the control and not a block: {SVL['stopMoved']}")
if not (SVL["stopNoBaseline"]["exit"] == 0 and SVL["stopNoBaseline"]["namesNotGated"]):
    sys.exit(f"refusing: the page says an agent no fire named is told it is not gated: {SVL['stopNoBaseline']}")
if not (SVL["ledger"]["startLines"] == 1 and SVL["ledger"]["refusedLines"] == 1 and SVL["ledger"]["blockLines"] == 0 and SVL["cleanedUp"]):
    sys.exit(f"refusing: the probe must leave one start line, one refused line, no block, and no file: {SVL['ledger']} cleaned {SVL['cleanedUp']}")
N_PROBE_FIRES = SVL["ledger"]["probeLines"] + 1          # the fifth fire (no baseline, its own session) is not under the probe session's id
FP_LEN = SVL["start"]["fingerprintLength"]
SVG = SV["ledger"]
N_START_REAL, N_VER_WRITTEN, N_REC_RED = SVG["subagentStartFires"], SVG["verifierTreeWritten"], SVG["recorderTreeRed"]
N_BLOCKS_EVER, N_BLOCKS_DAY, N_FP_FILES = SVG["subagentStopBlocksEver"], SVG["subagentStopBlocksOn0916"], SVG["agentFpFiles"]
if N_BLOCKS_DAY < 6:
    sys.exit(f"refusing: the record says six blocks on the day; the ledger holds fewer stop-gate blocks that day: {N_BLOCKS_DAY}")
I578b = SV["issue578"]
if not (I578b["exists"] and I578b["resolverAsExpected"] and I578b["resolverDodNamesIssue"] and I578b["severity"] == "High"):
    sys.exit(f"refusing: issue578, its resolver edge, the resolver's DoD or its severity is not as the page says: {I578b}")
SP1 = SV["sprint731"]
if not (SP1["exists"] and SP1["chartersAsExpected"] and SP1["retroNamesIssue578"] and SP1["standupNamesOrder"]):
    sys.exit(f"refusing: sprint 731 must exist, be chartered by the verifier-stop record, carry the retro's finding and the standup's dispatch order: {SP1}")
N_RESULTS1, N_GATE_RESULTS1 = SP1["results"], SP1["gateResults"]
SVP = SV["resolverPositions"]
if any(SVP[a] is None for a in SVP):
    sys.exit(f"refusing: an item the page places on the backlog is not there: {SVP}")
POS_VIEW, POS_SUITE, N_BACKLOG_SV = SVP["dcViewIsAMember"]["place"], SVP["dcSuiteIsAMember"]["place"], SV["backlogItems"]

# -- ask 6: the probe pair travels in a file --
PF = v("probePairTravelsInAFile")
if PF["status"] != "proposed" or PF["marker"] != "#ProspectiveChange" or PF["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {PF['status']}, {PF['marker']}, {PF['acceptance']}")
if not all(PF[k] for k in ("namesIssue571", "namesThreeDispatches", "namesEightySeconds", "namesD0224", "namesExactlyTwoLines", "namesVerifierNeverTranscribes",
                           "namesBothFlagsRefused", "namesTypedPairStays", "namesD0047", "namesTwoSecondRefusal", "saysProcessChange", "namesResolvesIssue571")):
    sys.exit("refusing: the pair-file Decision must name the Issue, the third dispatch, the eighty-second climb, the file rule, the two-line shape, that the verifier never transcribes, the both-flags refusal, the typed pair staying, the reminder rule, the two-second refusal, that it is a process change, and the Issue it resolves")
PFS = PF["source"]
if not all(PFS[k] for k in ("probeCarriesSource", "textNamesTheFile", "bothFlagsRefused", "exactlyTwoLines", "blankLineRefused", "ownArgsSkipsThePath", "waitRefusesIt", "parseBeforeAnyRung", "notRunRowNamesThePair", "notRunRowTest")) or len(PFS["probeTests"]) != 2:
    sys.exit(f"refusing: the control is not in source as the page describes it: {PFS}")
PFC = PF["cliFact"]
if not all(PFC.values()):
    sys.exit(f"refusing: the CLI fact or its mirror does not name the flag and the Decision: {PFC}")
PFK = PF["skill"]
if not all(PFK[k] for k in ("step1LaunchesFromFile", "transcribesNothing", "namesIssue571", "receiptShapeNamesFlag", "notRunRowForm", "claudeCopyAgrees", "briefSlotIsTheFile", "briefClaudeCopyAgrees", "processStepWritesTheFile")):
    sys.exit(f"refusing: the two skills or the process step are not as the page describes, or a .claude copy differs: {PFK}")
if PF["claudeMdLines"] < 2:
    sys.exit(f"refusing: CLAUDE.md must name the flag on both ladder lines: {PF['claudeMdLines']}")
PFL = PF["live"]
REFUSALS = [PFL["oneLine"], PFL["blankSecond"], PFL["threeLines"], PFL["bothFlags"], PFL["waitBeside"]]
if any(r["exit"] != 2 for r in REFUSALS) or not (PFL["oneLine"]["namesTheShape"] and PFL["blankSecond"]["namesTheLine"] and PFL["threeLines"]["namesTheShape"] and PFL["bothFlags"]["namesTwice"] and PFL["waitBeside"]["namesTheLaunch"]):
    sys.exit(f"refusing: every wrong shape must be refused at parse, exit 2, naming its shape: {REFUSALS}")
if not PFL["receiptUntouchedByRefusals"]:
    sys.exit("refusing: the page says a refusal writes no receipt; the receipt changed")
if not (PFL["help"]["exit"] == 0 and PFL["help"]["namesFlag"]):
    sys.exit(f"refusing: --help must name the flag: {PFL['help']}")
N_REFUSALS = len(REFUSALS)
PFV = PFL["verifierReceipt"]
if not (PFV["exists"] and PFV["outcome"] == "pass" and PFV["stoppedAt"] == "none"):
    sys.exit(f"refusing: the page says the sprint's own ladder is green end to end: {PFV}")
N_RUNGS_PF = PFV["rungsGreen"]
PFR = PFL["probeRung"]
# the receipt on disk is the LATEST ladder's - sprint 733's, whose pair is the module-home probe and the sprint's own
# negative (the fourth sprint to carry its pair in a file); the page says which file and whose sides, it does not require
# sprint 730's own pair to still be there
if not (PFR and PFR["verdict"] == "pass" and PFR["namesAFile"] and len(PFR["sides"]) == 2):
    sys.exit(f"refusing: the page says the latest probe rung ran from a named file of two lines: {PFR}")
if not PFR.get("fileStemIsAStem"):
    sys.exit(f"refusing: the pair file's stem must be one token - the sensor misread the command line: {PFR['fileStem']!r}")
# the receipt is one deep and machine-local: after sprint 734 lands it is 734's (pos734/neg734, the fifth sprint to carry
# its pair in a file); before, 733's (the module-home probe and neg733, the fourth). The page names the ordinal it read.
_PF_733 = "module_home.py" in PFR["sides"][0] and "--probe" in PFR["sides"][0] and "neg733" in PFR["sides"][1]
_PF_734 = "pos734" in PFR["sides"][0] and "neg734" in PFR["sides"][1]
# and after sprint 735 lands it is 735's (pos735/neg735, the sixth)
_PF_735 = "pos735" in PFR["sides"][0] and "neg735" in PFR["sides"][1]
# and each later extraction's pair is posNNN/negNNN (736 the seventh, 737 the eighth); the page names the ordinal it read
_PF_LATER = {n: (f"pos{n}" in PFR["sides"][0] and f"neg{n}" in PFR["sides"][1]) for n in (736, 737, 738)}
if not (_PF_733 or _PF_734 or _PF_735 or any(_PF_LATER.values())):
    sys.exit(f"refusing: the page says the latest ladder's pair is sprint 733's to 738's, positive then negative: {PFR['sides']}")
PAIR_ORDINAL = "ninth" if _PF_LATER[738] else "eighth" if _PF_LATER[737] else "seventh" if _PF_LATER[736] else "sixth" if _PF_735 else "fifth" if _PF_734 else "fourth"
# the ordinal as a count of ladders in a row (the checker reads "eighth" as "eight h", a duration)
N_PAIR_LADDERS = {"fourth": 4, "fifth": 5, "sixth": 6, "seventh": 7, "eighth": 8, "ninth": 9}[PAIR_ORDINAL]
PROBE_S_PF, LADDER_S_PF = PFR["seconds"], sum(PFL["rungSeconds"].values())
PAIR_FILE = PFR["fileStem"]
I571 = PF["issue571"]
if not (I571["exists"] and I571["resolverAsExpected"] and I571["resolverDodNamesIssue"]):
    sys.exit(f"refusing: issue571, its resolver edge or the resolver's DoD is missing: {I571}")
I578 = PF["issue578"]
if not (I578["exists"] and I578["resolverAsExpected"] and I578["resolverDodNamesIssue"]):
    sys.exit(f"refusing: issue578, its resolver edge or the resolver's DoD is missing: {I578}")
SP0 = PF["sprint730"]
if not (SP0["exists"] and SP0["chartersAsExpected"] and SP0["retroNamesIssue578"]):
    sys.exit(f"refusing: sprint 730 must exist, be chartered by the pair-file Decision and carry the retro's finding: {SP0}")
N_RESULTS0, N_GATE_RESULTS0 = SP0["results"], SP0["gateResults"]
PFP = PF["resolverPositions"]
if any(PFP[a] is None for a in PFP):
    sys.exit(f"refusing: a resolver the page places on the backlog is not there: {PFP}")
POS_STOP, N_BACKLOG_PF = PFP["dcVerifierStopIsNeverABlock"]["place"], PF["backlogItems"]

# -- ask 5 (first tab): the guard source's own tests follow the scratch rule, under the lock --
GS = v("guardTestsNameScratchPerProcess")
if GS["status"] != "proposed" or GS["marker"] != "#ProspectiveChange" or GS["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {GS['status']}, {GS['marker']}, {GS['acceptance']}")
if not (GS["dependsOnD0498"] and GS["d0498Status"] == "accepted"):
    sys.exit(f"refusing: the page says this record applies an accepted rule: dependsOn {GS['dependsOnD0498']}, rule {GS['d0498Status']}")
if not (GS["namesSevenSites"] and GS["namesTheLockReadsThePath"] and GS["namesNoPredicateChange"] and GS["namesCarveOutRejected"] and GS["namesLaterDecisionMayNarrow"] and GS["namesObligation"]):
    sys.exit("refusing: the record must name the seven sites, that the lock reads the path, no predicate change, the rejected carve-out, that a later Decision may narrow the lock, and the obligation")
GSG = GS["guardsRs"]
if not (GSG["lockedByName"] and GSG["lockMessageFound"] and GSG["firstCfgTestLine"]):
    sys.exit(f"refusing: guards.rs is not the locked file the page describes: {GSG}")
if GSG["scratchCallsInTestRegion"] != 7 or GSG["scratchCallsAboveIt"] != 0 or GSG["fixedJoinsInTestRegion"] != 0:
    sys.exit(f"refusing: the page says seven scratch calls, all in the test region, and no fixed join left: {GSG}")
GSD = GSG["diff"]
if GSD["insertions"] != 7 or GSD["deletions"] != 7 or len(GSD["hunkStartLines"]) != 7 or not GSD["allHunksBelowFirstCfgTest"]:
    sys.exit(f"refusing: the page says a 7/7 diff of seven one-line hunks, every one below the first #[cfg(test)]: {GSD}")
N_SITES_G, FIRST_CFG, N_HUNKS = GSG["scratchCallsInTestRegion"], GSG["firstCfgTestLine"], len(GSD["hunkStartLines"])
GSC = GS["control"]
if not (GSC["helperFound"] and GSC["helperCarriesPid"] and GSC["censusFound"] and len(GSC["probePair"]) == 2):
    sys.exit(f"refusing: the helper, the census or its probe pair is not in source as the page describes: {GSC}")
N_SCRATCH_SITES = GSC["scratchSitesInTree"]
if N_SCRATCH_SITES < N_SITES_G:
    sys.exit(f"refusing: the tree's scratch sites cannot be fewer than guards.rs's: {N_SCRATCH_SITES} < {N_SITES_G}")
GSO = GS["obligation"]
if not (GSO["exists"] and GSO["resolvedByD0499"] and GSO["firstRedWasTheLock"]):
    sys.exit(f"refusing: the yielded-red obligation must exist, open on the lock and be triaged to this record: {GSO}")
I577 = GS["issue577"]
if not (I577["exists"] and I577["resolverAsExpected"] and I577["resolverDodNamesIssue"]):
    sys.exit(f"refusing: issue577, its resolver edge or the resolver's DoD is missing: {I577}")
SP9 = GS["sprint729"]
if not (SP9["exists"] and SP9["chartersAsExpected"]):
    sys.exit(f"refusing: sprint 729 must exist and be chartered by the scratch rule: {SP9}")
N_RESULTS9, N_GATE_RESULTS9 = SP9["results"], SP9["gateResults"]
GSP = GS["resolverPositions"]
if GSP["dcWorktreeScratchIsPerProcess"] is None:
    sys.exit(f"refusing: the retro's resolver is not on the backlog: {GSP}")
POS_WORKTREE = GSP["dcWorktreeScratchIsPerProcess"]["place"]

# -- ask 4 (first tab): the wait for a launched ladder is a command --
WF = v("verifyWaitsForItsPid")
if WF["status"] != "proposed" or WF["marker"] != "#ProspectiveChange" or WF["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {WF['status']}, {WF['marker']}, {WF['acceptance']}")
if not (WF["namesIssue575"] and WF["namesSeventeenSeconds"] and WF["namesD0047"] and WF["namesLaunchesNothing"] and WF["namesKilledNotAVerdict"] and WF["saysProcessChange"]):
    sys.exit("refusing: the wait Decision must name the Issue, the early read, the sentence-is-not-a-control rule, that it launches nothing, the KILLED answer, and that it is a process change")
WFS = WF["source"]
if not all(WFS[k] for k in ("waitStateFound", "threeStatesPlusFinished", "pureControlFound", "livenessDecides", "deadWriterIsKilled", "oneReportForBothSurfaces", "launchFlagsRefused", "killedLine")) or len(WFS["probeTests"]) != 2:
    sys.exit(f"refusing: the control is not in source as the page describes it: {WFS}")
WFC = WF["cliFact"]
if not all(WFC.values()):
    sys.exit(f"refusing: the CLI fact or its mirror does not name the flag and the Issue: {WFC}")
WFK = WF["skill"]
if not (WFK["step4OpensWithWait"] and WFK["step5OpensWithWait"] and WFK["step5ProseWaitGone"] and WFK["claudeCopyAgrees"]):
    sys.exit(f"refusing: the skill's steps 4 and 5 do not open with the command, or the .claude copy differs: {WFK}")
if not WF["claudeMdLine"]:
    sys.exit("refusing: CLAUDE.md carries no `keel verify --wait [ROOT]` line")
WFL = WF["live"]
if WFL["refusal"]["exit0"] or not WFL["refusal"]["namesTheLaunch"]:
    sys.exit(f"refusing: `--wait --probe` must be refused naming the launch: {WFL['refusal']}")
if not (WFL["help"]["exit0"] and WFL["help"]["namesWait"] and WFL["help"]["namesIssue575"]):
    sys.exit(f"refusing: --help must name the flag and the Issue: {WFL['help']}")
I575 = WF["issue575"]
if not (I575["exists"] and I575["resolverAsExpected"] and I575["resolverDodNamesIssue"]):
    sys.exit(f"refusing: issue575, its resolver edge or the resolver's DoD is missing: {I575}")
SP8 = WF["sprint728"]
if not (SP8["exists"] and SP8["chartersAsExpected"]):
    sys.exit(f"refusing: sprint 728 must exist and be chartered by the wait Decision: {SP8}")
N_RESULTS8, N_GATE_RESULTS8 = SP8["results"], SP8["gateResults"]
WFP = WF["resolverPositions"]
if any(WFP[a] is None for a in WFP):
    sys.exit(f"refusing: a resolver the page places on the backlog is not there: {WFP}")
POS_EPOCH, POS_GITREAD = WFP["dcWaitChecksTheLaunchEpoch"]["place"], WFP["dcGuardReadFailureIsNotAnEmptyDiff"]["place"]
N_WAIT_STEPS = sum(1 for k in ("step4OpensWithWait", "step5OpensWithWait") if WFK[k])
N_PROBE_TESTS = len(WFS["probeTests"])

# -- ask 3: one script-probe runner for both surfaces --
PR = v("scriptProbesOneRunner")
if PR["status"] != "proposed" or PR["marker"] != "#ProspectiveChange" or PR["acceptance"] is not None:
    sys.exit(f"refusing: the page says a held, unaccepted process change: {PR['status']}, {PR['marker']}, {PR['acceptance']}")
if not (PR["namesIssue574"] and PR["namesBothRedCommits"] and PR["namesThirdMember"] and PR["namesD0098"] and PR["namesNoHeredoc"] and PR["saysProcessChange"]):
    sys.exit("refusing: the probes Decision must name the Issue, both red commits, that it is the third member, the never-skip rule, no heredoc, and that it is a process change")
PRR = PR["runner"]
if not all(PRR[k] for k in ("exists", "carriesOwnMarker", "discoverFound", "runFound", "reportFound", "pairFound", "noScriptsDirExits2")):
    sys.exit(f"refusing: the runner is not in source as the page describes it: {PRR}")
PRC = PR["ci"]
if not (PRC["stepFound"] and PRC["stepCallsRunner"] and PRC["stepNamesD0496"]) or PRC["stepHasHeredoc"] or PRC["fileHasScriptProbeDiscovery"]:
    sys.exit(f"refusing: the page says ci.yml's step is one line calling the runner and the file carries no second discovery: {PRC}")
PRH = PR["hook"]
if not (PRH["blockFound"] and PRH["callsRunner"] and PRH["abortsNamingIssue"] and PRH["cannotRunBranch"] and PRH["stagedPattern"]):
    sys.exit(f"refusing: the hook block, its staged-path test, its abort or its refusal branch is missing: {PRH}")
PRL = PR["live"]
if not (PRL["pair"]["exit0"] and PRL["pair"]["line"] and PRL["pair"]["line"].startswith("PROBE PASS")):
    sys.exit(f"refusing: the runner's own pair did not pass live: {PRL['pair']}")
if not (PRL["realTree"]["exit0"] and PRL["realTree"]["verdict"] and PRL["realTree"]["verdict"].endswith("script probe(s) passed")):
    sys.exit(f"refusing: the real tree's probes did not pass live: {PRL['realTree']}")
N_PROBES, PROBES_S, PAIR_S = PRL["realTree"]["count"], PRL["realTree"]["seconds"], PRL["pair"]["seconds"]
if N_PROBES < 2:
    sys.exit(f"refusing: the page says the runner discovered the tree's marked scripts; it found {N_PROBES}")
I574 = PR["issue574"]
if not (I574["exists"] and I574["resolverAsExpected"] and I574["resolverDodNamesIssue"]):
    sys.exit(f"refusing: issue574, its resolver edge or the resolver's DoD is missing: {I574}")
RED_RUNS = PR["ciRunsThatFailedOnTheHeredocStep"]
N_RED_RUNS = len([r for r in RED_RUNS if r["conclusion"] == "failure"])
SP7 = PR["sprint727"]
if not (SP7["exists"] and SP7["chartersAsExpected"]):
    sys.exit(f"refusing: sprint 727 must exist and be chartered by the probes Decision: {SP7}")
N_RESULTS7, N_GATE_RESULTS7 = SP7["results"], SP7["gateResults"]
PRP = PR["classResolverPositions"]
if PRP["dcVerifyWaitsForItsPid"] is None:
    sys.exit(f"refusing: the class's next resolver is not on the backlog: {PRP}")
POS_WAIT_NOW = PRP["dcVerifyWaitsForItsPid"]["place"]
N_PY_BLOCKS = PRH["pythonGatedBlocks"]

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
ENGINE_PATHS = L["landingCommitEnginePaths"]
STEMS_RUN_ATTRIBUTED = bool(LR["exists"] and LR["outcome"] == "pass" and LR["stemsIncludeInit"] and LR["failed"] == "0")
if STEMS_RUN_ATTRIBUTED and not ENGINE_PATHS:
    sys.exit("refusing: the page says the landed commit changed paths under .engine; git shows none")
if not STEMS_RUN_ATTRIBUTED and not (LATEST_IS_LATER_EMPTY and not ENGINE_PATHS):
    sys.exit(f"refusing: the latest run neither attributed init and passed nor is a later commit's empty run over no engine change: {LR} {ENGINE_PATHS}")
# the exhibit shows the run the receipt holds: one that attributed init, or the later empty one (its commit changed nothing under
# test and no engine file), with the last green landing's count beside it from its log
N_TESTS_LANDED = int(LR["passed"]) if STEMS_RUN_ATTRIBUTED else LAST_GREEN_TESTS
N_STEMS = len(LR["stems"])
N_SOURCE_STEMS = len([s for s in LR["stems"] if s != "init"])
STEMS_TITLE = (f"The latest landing run attributed {N_STEMS} stems, one from the embedded tree" if STEMS_RUN_ATTRIBUTED
               else f"The latest run at {LR['head'][:7]} attributed no stem: its commit changed nothing under test; the landing before it ran {N_TESTS_LANDED} green")
STEMS_TRUE = (f"the latest landing took <q>init</q> from {len(ENGINE_PATHS)} engine files, {N_TESTS_LANDED} tests green" if STEMS_RUN_ATTRIBUTED
              else f"the latest run at {LR['head'][:7]} attributed no stem and changed {len(ENGINE_PATHS)} engine files; the landing before it, {LAST_GREEN_SPRINT}, {N_TESTS_LANDED} tests green from its log")
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
    STEMS_TITLE,
    [
        ("from changed source modules", N_SOURCE_STEMS, "ok", ""),
        ("from the embedded tree", N_STEMS - N_SOURCE_STEMS, "accent", ""),
        ("engine files behind that stem", len(ENGINE_PATHS), "accent", ""),
        ("tests green under it" if STEMS_RUN_ATTRIBUTED else "tests green in the landing before it, from its log", N_TESTS_LANDED, "ok", ""),
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


figP1 = logic_lanes(
    "Two judges do not read one list",
    ("what happened", [
        ("probe list", "in CI's yaml only", "accent", ""),
        ("local gates", "green", "ok", ""),
        ("CI, after the push", "red", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("probe list", "one runner, both call it", "accent", ""),
        ("script staged", f"{N_PROBES} probes in {PROBES_S} s", "ok", ""),
        ("python missing", "refuses, remedy named", "bad", ""),
    ], ["", "", ""]),
)
figP2 = bars(
    f"The runner finds {N_PROBES} marked scripts and runs them in {PROBES_S} s",
    [
        ("marked scripts it found", N_PROBES, "ok", ""),
        ("seconds, the whole set", PROBES_S, "accent", ""),
        ("seconds, its own pair", PAIR_S, "accent", ""),
        ("CI runs red on the old step", N_RED_RUNS, "bad", ""),
    ],
    unit=("item", "items"),
)
figP3 = downstream(
    "One runner lands on three surfaces",
    ("script probes", "one executable", ""),
    [
        ("ci.yml's step", "one line, no heredoc", "1", "ok"),
        ("the pre-commit hook", "when a script is staged", "1", "ok"),
        ("the runner itself", "a member of its own set", "1", "ok"),
        ("Issues in the class", "each with a resolver", f"{N_CLASS}", "accent"),
    ],
)


figW1 = logic_lanes(
    "A sentence does not stop an early read; a command does",
    ("what happened", [
        ("ladder launched", "stub says running", "accent", ""),
        ("read too soon", "stub taken as the verdict", "bad", ""),
        ("green ladder", "reported killed", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("ladder launched", "stub names its pid", "accent", ""),
        ("--wait", "blocks while the pid lives", "ok", ""),
        ("pid gone, still running", "KILLED, exit 2", "bad", ""),
    ], ["", "", ""]),
)
figW2 = bars(
    f"The stub says running for all {LADDER_S} s of a ladder",
    [(name, secs, "accent" if name == "touched" else "ok", "") for name, secs in RUNG_S.items()],
    unit=("second", "seconds"),
)
figW3 = downstream(
    "One command lands on four surfaces",
    ("the wait", "verify --wait", ""),
    [
        ("the verify binary", "in flight, finished, or killed", "1", "ok"),
        ("the CLI fact and its mirror", "name the flag", "2", "ok"),
        ("the verifier's steps", "open with it", f"{N_WAIT_STEPS}", "ok"),
        ("the retro's next item", f"sits {POS_EPOCH} of {N_BACKLOG}", "1", "accent"),
    ],
)


figG1 = logic_lanes(
    "The lock is on the file, not the region",
    ("what happened", [
        ("test lines edited", f"{N_HUNKS} one-line hunks", "accent", ""),
        ("no predicate touched", f"all below line {FIRST_CFG}", "ok", ""),
        ("lock fires", "tree red, census cannot land", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("test lines edited", f"{N_HUNKS} one-line hunks", "accent", ""),
        ("held record", "names the lines, nothing else", "ok", ""),
        ("lock satisfied", "signed when you sign", "ok", ""),
    ], ["", "", ""]),
)
figG2 = bars(
    f"Seven of {N_SCRATCH_SITES} per-process scratch sites sit in the locked file",
    [
        ("scratch sites across the tree", N_SCRATCH_SITES, "ok", ""),
        ("of them in the guard source", N_SITES_G, "accent", ""),
        ("lines changed in that file", N_HUNKS, "accent", ""),
        ("fixed-name joins left in its tests", GSG["fixedJoinsInTestRegion"], "ok", ""),
    ],
    unit=("site", "sites"),
)
figG3 = downstream(
    "One record lands on three surfaces",
    ("the held record", "guards.rs test module", ""),
    [
        ("the process-change lock", "satisfied at sign-off", "1", "ok"),
        ("the yielded-red obligation", "triaged to it", "1", "ok"),
        ("the retro's worktree item", f"sits {POS_WORKTREE} of {N_BACKLOG}", "1", "accent"),
    ],
)


figF1 = logic_lanes(
    "A pair retyped by a second actor is the defect; a file is not retyped",
    ("what happened", [
        ("primary names the pair", "correct, in the dispatch", "ok", ""),
        ("verifier retypes it", "into --probe A,B", "bad", ""),
        ("probe rung", "red after an 80 s climb", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("primary writes the file", "two lines, before reading", "ok", ""),
        ("verifier names the path", "--probe-from, retypes nothing", "ok", ""),
        ("wrong shape", "refused at parse, exit 2", "accent", ""),
    ], ["", "", ""]),
)
figF2 = bars(
    f"The probe rung is {PROBE_S_PF} s of a {LADDER_S_PF} s ladder read from a file",
    [(name, secs, "accent" if name == "probe" else "ok", "") for name, secs in PFL["rungSeconds"].items()],
    unit=("second", "seconds"),
)
figF3 = downstream(
    "One file lands on four surfaces",
    ("the pair file", "verify --probe-from", ""),
    [
        ("the verify binary", f"{N_REFUSALS} wrong shapes refused live", f"{N_REFUSALS}", "ok"),
        ("the CLI fact and its mirror", "name the flag", "2", "ok"),
        ("the two skills and the process step", "name the path, not the pair", "3", "ok"),
        ("the retro's stop-hook item", f"sits {POS_STOP} of {N_BACKLOG_PF}", "1", "accent"),
    ],
)


figN1 = logic_lanes(
    "The list descends because a member cannot read the crate above it",
    ("before the move", [
        ("view layer in keel-cli", "reads the list by the arms", "accent", ""),
        ("one file, one lock", "list and arms together", "ok", ""),
        ("keel-view extracted", "cannot see guards.rs", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("list in keel-schema", "old path re-exported", "ok", ""),
        ("the list joins the lock", f"{N_LOCK_PATHS} locked sources", "accent", ""),
        ("delete a name", "a signed edit", "ok", ""),
    ], ["", "", ""]),
)
figN2 = bars(
    f"Moving {N_GUARDS} names changes no count: {N_GUARDS} before and after",
    [
        ("names in the schema file", N_GUARDS, "ok", ""),
        ("names keel version reports", GNV["version"]["guards"], "ok", ""),
        ("locked sources", N_LOCK_PATHS, "accent", ""),
        ("moved files", N_RENAMES, "ok", ""),
        ("files deleted", N_DEL if False else GNL["deleted"], "ok", ""),
    ],
    unit="",
)
figN3 = downstream(
    "One list lands on five surfaces",
    ("the guard-name list", "keel-schema/src/guard_names.rs", ""),
    [
        ("guards.rs", "re-exports it; tests hold names to arms", "1", "ok"),
        ("the lock and its catalogue", "both name the file", "2", "ok"),
        ("the view member's census", "reads the schema, not the arms", "1", "ok"),
        ("the renderer's process, skill", "name the member path", "2", "ok"),
        ("the next extraction", f"suite sits {POS_SUITE2} of {N_BACKLOG_GN}", "1", "accent"),
    ],
)

figV1 = logic_lanes(
    "A block on the verifier is an order to do what its role forbids",
    ("what happened", [
        ("primary leaves a red", "a placeholder", "accent", ""),
        ("verifier stops", "gate: make green", "bad", ""),
        ("verifier obeys", "edits a record", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("verifier stops", "own start compared", "ok", ""),
        ("tree unmoved", "silent", "ok", ""),
        ("tree moved", "one counted line", "accent", ""),
    ], ["", "", ""]),
)
figV2 = bars(
    f"{N_BLOCKS_DAY} stop-gate blocks fired that day; a verifier's write is now a counted line",
    [
        ("stop-gate blocks that day", N_BLOCKS_DAY, "bad", ""),
        ("recorder red-tree lines", N_REC_RED, "accent", ""),
        ("verifier tree-written lines", N_VER_WRITTEN, "ok", ""),
        ("live probe: blocks on a verifier", SVL["ledger"]["blockLines"], "ok", ""),
    ],
    unit=("line", "lines"),
)
figV3 = downstream(
    "One route lands on five surfaces",
    ("the stop route", "hook subagent-stop", ""),
    [
        ("the hook, live", "moved start: exit 0, one line", "1", "ok"),
        ("the control map", f"one hook-rule beside {N_HOOK_RULES - 1}", "1", "ok"),
        ("step, skill, working rules", "name both controls", "3", "ok"),
        ("the census", "counts the verifier's writes", f"{N_VER_WRITTEN}", "accent"),
        ("the next items", f"view member sits {POS_VIEW} of {N_BACKLOG_SV}", "1", "accent"),
    ],
)

figO1 = logic_lanes(
    "The interval measured was the session's, not the agent's",
    ("what happened", [
        ("session's first fire", "one baseline, hours old", "accent", ""),
        ("primary edits", "the tree moves", "accent", ""),
        ("agent stops", "gated over others' edits", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("agent starts", "own fingerprint written", "ok", ""),
        ("primary's earlier edits", "outside the interval", "ok", ""),
        ("agent stops", "gated only on its own", "ok", ""),
    ], ["", "", ""]),
)
figO2 = bars(
    f"All {N_BLOCKS_EVER} stop-gate blocks were session-measured; {N_START_REAL} agents since measured from their start",
    [
        ("blocks ever, session-measured", N_BLOCKS_EVER, "bad", ""),
        ("of them that day", N_BLOCKS_DAY, "bad", ""),
        ("agent starts since the change", N_START_REAL, "ok", ""),
        ("start files on disk", N_FP_FILES, "ok", ""),
    ],
    unit="",
)
figO3 = downstream(
    "One fingerprint lands on four surfaces",
    ("the agent's own start", "one file per agent", ""),
    [
        ("the hook registration", "seventh event, one home", "1", "ok"),
        ("control events, fire ledger", "the start is counted", "2", "ok"),
        ("the hook's CLI fact", "retired by an edge", "1", "ok"),
        ("the verifier's control", "stands on this record", "1", "accent"),
    ],
)


figE1 = logic_lanes(
    "A path lock stops a test-only edit like a logic edit",
    ("before this sprint", [
        ("ten local helpers", "one per test module", "bad", ""),
        ("eight cwd anchors", "break two levels down", "bad", ""),
        ("one of them", "inside a locked file", "accent", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("one helper", "keel-fs walks to .git", "ok", ""),
        ("the locked file", "1 line out, 1 line in", "ok", ""),
        ("the lock", "fires; this record answers", "ok", ""),
    ], ["", "", ""]),
)
figE2 = bars(
    f"One helper is left; {N_WS_MEMBERS_LA} members have no anchor",
    [
        ("fn repo_root definitions", LAS["repoRootDefinitionCount"], "ok", ""),
        ("modules using it", N_USE_FILES, "accent", ""),
        ("keel-cli callers", N_CLI_CALLERS, "accent", ""),
        ("anchors, keel-cli/src", len(LAS["anchorsInCliSrc"]), "ok", ""),
        ("anchors, members", len(LAS["anchorsInMemberSrc"]), "ok", ""),
        ("anchors, keel-cli/tests", N_TEST_ANCHORS, "accent", ""),
        ("locked lines changed", LAL["adherenceInsertions"] + LAL["adherenceDeletions"], "ok", ""),
    ],
    unit="",
)
figE3 = downstream(
    "One repoint lands on four surfaces",
    ("the locked test's root", "keel_fs::test_support::repo_root()", ""),
    [
        ("the lock", "green beside this record", "1", "ok"),
        ("the anchor scan", "now covers keel-cli/src", "2", "ok"),
        ("the landing run", LANDING4_ROW, "1", "ok"),
        ("next extraction", f"suite {POS_SUITE4}, process {POS_PROCESS4} of {N_BACKLOG_LA}", "2", "accent"),
    ],
)
figM1 = logic_lanes(
    "Tooling that reads a guard is rebuilt with every guard",
    ("before this sprint", [
        ("five tooling modules", "inside keel-cli", "bad", ""),
        ("their key + predicate", "inside the guard member", "bad", ""),
        ("a view or guard edit", "rebuilds the tooling", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("member keel-suite", f"{N_PATH_DEPS} leaf members below it", "ok", ""),
        ("key + predicate moved", f"{N_LOCKED_OUT} lines out, {N_LOCKED_IN} in", "ok", ""),
        ("a view or guard edit", "tooling stays Fresh", "ok", ""),
    ], ["", "", ""]),
)
figM2 = bars(
    f"The member reads {N_PATH_DEPS} leaves; {N_GUARDS} guards unchanged",
    [
        ("modules moved", len(SMS["memberDeclaresTheFive"]), "ok", ""),
        ("of them moved whole", len(SML["movedWhole"]), "accent", ""),
        ("path dependencies (leaves)", N_PATH_DEPS, "ok", ""),
        ("view/guard/write dependencies", 0, "ok", ""),
        ("guards, before and after", N_GUARDS, "ok", ""),
    ],
    unit="",
)
figM3 = downstream(
    "Two descents land on four surfaces",
    ("the tooling member", "members/keel-suite", ""),
    [
        ("the lock", "green beside this record", "1", "ok"),
        ("the receipt census", f"walks {N_WS_MEMBERS_SM} members' src", "1", "ok"),
        ("the landing run", LANDED_ROW5, "1", "ok"),
        ("next extraction", f"process {POS_PROCESS5}, issues {POS_ISSUES5} of {N_BACKLOG_SM}", "2", "accent"),
    ],
)
figR1 = logic_lanes(
    "A review the human has not given is counted as owed",
    ("today", [
        ("sitting review", "confirmation, human-judged", "bad", ""),
        ("act tab", f"{N_DUE_SR} reviews owed", "bad", ""),
        ("each judgment in it", "already has its own verb", "accent", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("sitting review", "analysis, AI-judged", "ok", ""),
        ("assurance surface", f"{N_COVERED_SR} of {N_SPRINTS_SR} presented", "ok", ""),
        ("the human's word", "accept, judge-set, disposition", "ok", ""),
    ], ["", "", ""]),
)
figR3 = downstream(
    "One method change lands on four surfaces and leaves history alone",
    ("the sitting review", "sprint-review skill, phase 4", ""),
    [
        ("the record", "analysis; asks listed by verb", "1", "ok"),
        ("the viewpoint", "act to assurance", "1", "ok"),
        ("the console's act tab", f"{N_DUE_SR} owed become 0", "1", "ok"),
        ("a guard", "no new confirmation review", "2", "accent"),
        ("history", f"{N_REVIEWS_SR} reviews stay as recorded", "0", "muted"),
    ],
)
figCN1 = logic_lanes(
    "The console reads every member, so it sits above all of them",
    ("before", [
        ("serve, deck, launcher", "in the binary's crate", "bad", ""),
        ("any member edit", "rebuilds the binary and the console", "bad", ""),
        ("the HTTP-surface guard", "reads keel-cli/src paths", "accent", ""),
    ], ["", "", ""]),
    ("after", [
        ("member keel-serve", f"{N_PATH_DEPS38} members below it", "ok", ""),
        ("keel-cli", "the one reader; help byte-identical", "ok", ""),
        ("the guard", "reads the file where the manifests put it", "ok", ""),
    ], ["", "", ""]),
)
figCN3 = downstream(
    "The last extraction lands on five surfaces",
    ("the console member", "members/keel-serve", ""),
    [
        ("the lock", f"{N_LOCKED_OUT38} out, {N_LOCKED_IN38} in, path strings only", "1", "ok"),
        ("the landing run", LANDED_ROW38, "1", "ok"),
        ("three new homes", "ci_runs, verification, readiness", "1", "ok"),
        ("three findings", f"registry {POS_REGISTRY38}, lint {POS_LINT38}, standby {POS_STANDBY38} of {N_BACKLOG_CN}", "2", "accent"),
        ("thin dispatch", f"item {POS_THIN38} of {N_BACKLOG_CN}", "2", "accent"),
    ],
)
figI1 = logic_lanes(
    "A write that reads a guard's predicate has every guard above it",
    ("before", [
        ("the resolver check", "reads the guard member", "bad", ""),
        ("issue write, views", "in two other members", "bad", ""),
        ("any guard edit", "rebuilds the item processes", "bad", ""),
    ], ["", "", ""]),
    ("after", [
        ("the predicate", "in keel_model::resolvers", "ok", ""),
        ("member keel-issues", f"{N_PATH_DEPS8} members below it", "ok", ""),
        ("any guard edit", "the member stays Fresh", "ok", ""),
    ], ["", "", ""]),
)
# no bars figure for this ask: every count it would carry is in the panel's True line already (one home, D0105)
figI3 = downstream(
    "A descent and a reversed clause land on four surfaces",
    ("the item member", "members/keel-issues", ""),
    [
        ("the lock", "green beside this record", "1", "ok"),
        ("the charter", "one clause reversed", "1", "ok"),
        ("the landing run", LANDED_ROW8, "1", "ok"),
        ("next extraction", f"touched set {POS_TOUCHED8}, serve {POS_SERVE8} of {N_BACKLOG_IM}", "2", "accent"),
    ],
)
figD1 = logic_lanes(
    "A lock naming one file goes quiet when it moves",
    ("before the move", [
        ("one file, one lock", f"{OLD_LINES:,} lines by path", "accent", ""),
        ("bodies move out", "named file gone", "bad", ""),
        ("scan of keel-cli/src", "green over nothing", "bad", ""),
    ], ["", "", ""]),
    ("as the clause reads", [
        ("lock = a prefix", f"{N_LOCK_DIRS} dir + {N_LOCK_FILES} files", "ok", ""),
        ("a new family file", "locked by construction", "ok", ""),
        ("coverage test", f"scans {N_WS_MEMBERS} members", "ok", ""),
    ], ["", "", ""]),
)
figD2 = bars(
    f"Moving {N_GUARDS} guards into {N_MEMBER_FILES} files changes no count",
    [
        ("guards, keel version", GPV["version"]["guards"], "ok", ""),
        ("families", N_FAMILIES, "ok", ""),
        ("files in the locked dir", N_MEMBER_FILES, "accent", ""),
        ("defining a guard", N_DEFINING, "accent", ""),
        ("outside the lock", len(GPS["uncoveredGuardDefiningFiles"]), "ok", ""),
        ("files moved", N_RENAMES3, "ok", ""),
        ("deleted", N_DELETED3, "ok", ""),
    ],
    unit="",
)
figD3 = downstream(
    "One prefix lands on five surfaces",
    ("the guard-source lock", "members/keel-guards/src/", ""),
    [
        ("the lock's tests", f"{len(GPS['lockTestLocks'])} locked, {len(GPS['lockTestFrees'])} free", "2", "ok"),
        ("dispatch, union tests", "read the family tables", "2", "ok"),
        ("the catalogue", f"names the member, {N_DOC_TIERS} tiers", "1", "ok"),
        ("the landing run", LANDING3_ROW, "1", "ok"),
        ("next extraction", f"root {POS_ROOT3}, suite {POS_SUITE3} of {N_BACKLOG_GS}", "2", "accent"),
    ],
)


def words(fragment):
    return len(field_text(re.sub(r"<[^>]+>", " ", markup_only(fragment))).split())


ASKS = [
    ("d0510", "ask-sittingreview", "Review", "sittingreview"),
    ("d0509", "ask-servemember", "Console", "servemember"),
    ("d0508", "ask-issuesmember", "Issues", "issuesmember"),
    ("d0506", "ask-suitemember", "Suite", "suitemember"),
    ("d0505", "ask-lockedanchor", "Anchor", "lockedanchor"),
    ("d0504", "ask-guardsource", "Guard source", "guardsource"),
    ("d0503", "ask-guardnames", "Guard names", "guardnames"),
    ("d0502", "ask-verifierstop", "Verifier stop", "verifierstop"),
    ("d0501", "ask-ownstart", "Own start", "ownstart"),
    ("d0500", "ask-pairfile", "Pair file", "pairfile"),
    ("d0499", "ask-guardtests", "Guard tests", "guardtests"),
    ("d0497", "ask-wait", "Wait", "wait"),
    ("d0496", "ask-probes", "Probes", "probes"),
    ("d0495", "ask-triple", "Triple", "triple"),
    ("d0494", "ask-stems", "Stems", "stems"),
]
panel_bodies = {
    "sittingreview": f"""<h2>A sitting review is finished by analysis, not confirmation</h2>
{clause(SRD)}
<p><strong>What happened:</strong> each part of a review has its judge: tested work self-evidences; a proposed result is yours per item (judge-set); a Decision per Decision (accept); a finding per finding (disposition). What remains, <q>the review is done</q>, is a receipt, not testimony. The skill still records it as your confirmation; the act tab counts {N_DUE_SR} owed.</p>
<p><strong>What changes:</strong> an analysis record judged by the AI that ran it, listing what waits on your word by verb; the viewpoint moves from act to assurance; a guard admits no new confirmation review. The {N_REVIEWS_SR} recorded stay your word.</p>
{figR1}
{figR3}
{courses([
    ("Accept (recommended)", "skill phase 4, viewpoint, act tab, one guard", f"{N_DUE_SR} owed leave the console"),
    ("Keep confirmation", "nothing", f"{N_DUE_SR} owed, growing"),
    ("Do nothing", "nothing", "proposed; the count grows"),
])}
<p><strong>True:</strong> {N_SPRINTS_SR} sittings, {N_COVERED_SR} covered by {N_REVIEWS_SR} reviews, {N_UNCOV_SR} uncovered: {N_GRAND_SR} grandfathered, {N_DUE_SR} owed; the skill says confirmation {N_SKILL_CONF_SR} times; nothing applied. <strong>Mine:</strong> nothing is left for a sitting-level word. <strong>What decides it:</strong> a judgment of yours about the whole sitting that no verb carries - name it and it becomes one. <strong>Wrong if</strong> a sitting's direction is yours to judge and is not already a Decision.</p>
{opts("ask-sittingreview", "d0510", [
    ("Accept (recommended)", "Accept: a sitting review is a Test with method = analysis judged by the AI actor, carrying #Covers edges and listing the outstanding per-item asks by verb (keel accept, keel judge-set, a disposition); no confirmation-method review is recorded for a sitting from here on; sittingReviewVP moves to the assurance surface; the existing reviews stay as recorded (recommended)"),
    ("Keep confirmation", "Reject as written: the sitting review stays method = confirmation, human-judged, and the act tab keeps counting sprints without one"),
    ("Do nothing", "Do nothing: the Decision stays proposed here; the skill, the viewpoint and the count stay as they are"),
])}""",
    "servemember": f"""<h2>The console is a member above every member and below the binary</h2>
{clause(CN)}
<p><strong>What happened:</strong> the console, deck, launcher and registry sat in the binary's crate: every member edit rebuilt them, and the HTTP-surface guard named their paths there. They moved to one member that reads every member and is read by the binary alone.</p>
<p><strong>What changes:</strong> no guard, no lock rule. Three locked files lose {N_LOCKED_OUT38} lines and gain {N_LOCKED_IN38}: path strings following their files.</p>
{figCN1}
{figCN3}
{courses([
    ("Accept (recommended)", "nothing; landed, lock green, help byte-identical", "one record"),
    ("Keep the console in the binary", "revert the move", "every member edit rebuilds the console"),
    ("Do nothing", "nothing", "landed, unsigned"),
])}
<p><strong>True:</strong> {N_PATH_DEPS38} path dependencies, {N_CRATE_DEPS38} crates; {N_WS_MEMBERS_CN} members, this one last before the binary; {N_FILES38} files in the range ({N_RENAMES38} moved, {N_MODIFIED38} modified, {N_ADDED38} added), +{N_INS38}/-{N_DEL38}; {LANDED_ROW38}; {N_RETRO38} findings, {N_RETRO_TRACKED38} tracked. <strong>Mine:</strong> a fact lives where its readers are; the console's are the browser and the binary. <strong>What decides it:</strong> may the lock pass path strings that follow a moved file when no rule changes. <strong>Wrong if</strong> a guard reading a file at its new path is a different guard.</p>
{opts("ask-servemember", "d0509", [
    ("Accept (recommended)", "Accept: the console is member keel-serve, reading every member and read by keel-cli alone; ci_runs joins keel-github, verification joins keel-view, the orient computation is keel_model::readiness; the hardening guard reads the console where it lives; keel --help stays byte-identical and every guard's dispatch, severity and gate set is unchanged (recommended)"),
    ("Keep the console in the binary", "Reject as written; ask for the fork: serve, deck, launcher and console_registry return to keel-cli/src and the hardening guard reads them there"),
    ("Do nothing", "Do nothing: the Decision stays proposed here; the member and the three new homes stay as landed"),
])}""",
    "issuesmember": f"""<h2>Item processes read model and write API only</h2>
{clause(IM)}
<p><strong>What happened:</strong> the issue write checked its resolver through the guard member's function, so the item processes carried every guard. The function moved into the model, below both readers. The synopsis clause is reversed: <q>keel --help</q> has stayed byte-identical through every extraction.</p>
<p><strong>What changes:</strong> no guard, no lock rule. Two locked files lose {N_LOCKED_OUT8} lines and gain {N_LOCKED_IN8} (re-exports); one charter clause is reversed.</p>
{figI1}
{figI3}
{courses([
    ("Accept (recommended)", "nothing; landed, lock green, probe Fresh", "one record"),
    ("Keep it on the guards", "keel-issues depends on keel-guards", "rebuilt per guard edit"),
    ("Do nothing", "nothing", "landed, unsigned"),
])}
<p><strong>True:</strong> {len(IMS["memberDeclaresTheFour"])} modules in the member ({len(IML["renamedFrom"])} moved as files); {N_PATH_DEPS8} path dependencies, none a guard, view, suite or process; locked diff {N_LOCKED_OUT8} out, {N_LOCKED_IN8} in; {N_GUARDS} guards unchanged; {N_LANDED_TESTS8} tests green; {N_RETRO8} findings, {N_RETRO_NOT_TRACKED8} untracked with reasons. <strong>Mine:</strong> a predicate two members apply sits below both; a synopsis restates a computed fact. <strong>What decides it:</strong> may the lock pass lines leaving a guard file when no rule changes; may a held record reverse one clause of an accepted charter. <strong>Wrong if</strong> the compiled function is a different guard, or help should name a verb's crate.</p>
{opts("ask-issuesmember", "d0508", [
    ("Accept (recommended)", "Accept: keel-issues depends on keel-model, keel-write, keel-schema and keel-json only; declared_task_names and resolver_kind_holds live in keel_model::resolvers, re-exported by keel-guards at their old paths; D0480's synopsis clause is reversed and keel --help stays byte-identical; every guard's dispatch, severity and gate set is unchanged (recommended)"),
    ("Keep it on the guards", "Reject as written; ask for the fork: keel-issues depends on keel-guards for the resolver predicate, the probe leaves the DoD, and D0480's synopsis clause stands"),
    ("Do nothing", "Do nothing: the Decision stays proposed here; the member, the descent and the re-exports stay as landed"),
])}""",
    "suitemember": f"""<h2>The tooling depends on nothing it measures</h2>
{clause(SM)}
<p><strong>What happened:</strong> the build-and-test tooling sat in keel-cli reading one key and one predicate from the guard member, so every guard edit rebuilt it. It moved to its own member; key with it, predicate down to the file member.</p>
<p><strong>What changes:</strong> no guard, no lock rule, no receipt. Two locked files lose {N_LOCKED_OUT} lines and gain {N_LOCKED_IN} (a re-export; every caller compiles unchanged); this record answers the lock.</p>
{figM1}
{figM2}
{figM3}
{courses([
    ("Accept (recommended)", "nothing; landed, lock green, probe Fresh", "one record for two descents"),
    ("Keep the tooling on the guards", "keel-suite keeps its keel-guards edge", "rebuilt on every guard edit"),
    ("Do nothing", "nothing", "landed, unsigned"),
])}
<p><strong>True:</strong> {len(SMS["memberDeclaresTheFive"])} modules moved ({len(SML["movedWhole"])} whole), {len(SMS["cliStillHoldsModules"])} left behind; {N_PATH_DEPS} path dependencies, all leaves; locked diff {N_LOCKED_OUT} out, {N_LOCKED_IN} in over {len(SML["lockedFilesTouched"])} files; {N_GUARDS} guards before and after; {N_LANDED_TESTS5} tests green; {N_RETRO5} findings, {N_RETRO_NOT_TRACKED5} untracked with reasons. <strong>Mine:</strong> a predicate two members read belongs below both. <strong>What decides it:</strong> should the lock fire for lines leaving a guard file when no guard's rule changes. <strong>Wrong if</strong> byte-identical guard text is a different guard once compiled from another member.</p>
{opts("ask-suitemember", "d0506", [
    ("Accept (recommended)", "Accept: keel-suite depends on keel-fs, keel-git, keel-actor, keel-model and keel-perf only; contentkey moves whole to the member and no_receipt_forced moves down to keel_fs::fsx with keel_guards::receipt::forced re-exporting it; every guard's dispatch, severity and gate set is unchanged (recommended)"),
    ("Keep the tooling on the guards", "Reject as written; ask for the fork: keel-suite depends on keel-guards for the content key and the no-receipt predicate, and the freshness probe is dropped from the sprint's DoD"),
    ("Do nothing", "Do nothing: the Decision stays proposed here; the member, the two descents and the re-export stay as landed"),
])}""",
    "lockedanchor": f"""<h2>The lock fired for a test-only edit</h2>
{clause(LA)}
<p><strong>What happened:</strong> every test found the repository root its own way: ten helper copies, eight anchors on the crate's parent, the shape that broke twice. One keel-fs helper replaces them; one anchor sat in a locked guard source, so the lock stopped the commit.</p>
<p><strong>What changes:</strong> no guard, no lock rule. One test line in the locked file calls the helper; this record answers the lock beside it, the first marked record governing no enforcement logic.</p>
{figE1}
{figE2}
{figE3}
{courses([
    ("Accept (recommended)", "nothing; landed, lock green", "one record for a two-line diff"),
    ("Exempt test modules", "the lock skips cfg(test) lines", "a second process change; a lock reading code shape"),
    ("Do nothing", "nothing", "landed, unsigned"),
])}
<p><strong>True:</strong> {LAS["repoRootDefinitionCount"]} helper definition, {N_USE_FILES} importing modules, {N_CLI_CALLERS} keel-cli callers; {len(LAS["anchorsInCliSrc"])} anchors in keel-cli/src, {len(LAS["anchorsInMemberSrc"])} in members, {N_TEST_ANCHORS} in keel-cli/tests (outside the scan); locked diff {LAL["adherenceInsertions"]} line in, {LAL["adherenceDeletions"]} out; {LANDING4_TRUE}; {PTS4} points, {N_RETRO4} findings. <strong>Mine:</strong> answering the lock beats an exception in the scan. <strong>What decides it:</strong> should a path lock fire for edits touching no logic. <strong>Wrong if</strong> the lock is meant to guard logic alone.</p>
{opts("ask-lockedanchor", "d0505", [
    ("Accept (recommended)", "Accept: the only edit in keel-cli/src/adherence.rs is one test line - this_repo_yields_the_empty_prefix takes its root from keel_fs::test_support::repo_root(); the audit, the lock and every guard's dispatch, severity and gate set are unchanged; the anchor scan widens to keel-cli/src (recommended)"),
    ("Exempt test modules", "Reject as written; ask for the fork: a separate process-change Decision exempting cfg(test) lines from the guard-source lock"),
    ("Do nothing", "Do nothing: the Decision stays proposed here; helper, repoint and scan stay as landed"),
])}""",
    "guardsource": f"""<h2>The lock names the member directory, not one file</h2>
{clause(GP)}
<p><strong>What happened:</strong> the fourth extraction moved the guard bodies ({OLD_LINES:,} lines, one file) into member keel-guards. The lock named that file; a scan aimed at it now passes over nothing.</p>
<p><strong>What changes:</strong> no guard. The lock names the directory by prefix beside two files; the coverage test scans every member for a guard file outside it; dispatch reads the family tables.</p>
{figD1}
{figD2}
{figD3}
{courses([
    ("Accept (recommended)", "a prefix; the next file is locked unasked", "the member's src is signed"),
    ("Keep a file list", f"{N_MEMBER_FILES} paths, one by one", "the next file unlocked until added"),
    ("Do nothing", "nothing", "landed, unsigned"),
])}
<p><strong>True:</strong> {N_GUARDS} guards before and after; {N_FAMILIES} families, {N_MEMBER_FILES} files, {len(GPS["uncoveredGuardDefiningFiles"])} guard files outside the lock; {N_RENAMES3} renames, {N_DELETED3} deletion; {LANDING3_TRUE}; {PTS3} points, {N_RETRO3} findings, five Issues. <strong>Mine:</strong> a directory is the right grain. <strong>What decides it:</strong> may the lock widen from named files to every file under one directory. <strong>Wrong if</strong> a guard file lands in the member unlocked, or the coverage test passes over no member.</p>
{opts("ask-guardsource", "d0504", [
    ("Accept (recommended)", "Accept: GUARD_SOURCE_FILES loses keel-cli/src/guards.rs and a second constant, GUARD_SOURCE_DIRS, names members/keel-guards/src/; is_enforcement_surface holds a path locked when it is in the file list or starts with a locked directory; the coverage test reads the workspace manifest and scans every member's src for a guard-defining file outside the lock, asserting the scan reached the guards member's own files; every_enforced_guard_dispatches reads the family tables (recommended)"),
    ("Keep a file list", "Reject the prefix: keep GUARD_SOURCE_FILES as the only lock constant and name each of the member's guard files in it one by one, adding each new family file by hand"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the member, the prefix and the coverage test stay as landed"),
])}""",
    "guardnames": f"""<h2>The guard-name list lives in the schema member, under the lock</h2>
{clause(GN)}
<p><strong>What happened:</strong> the third extraction moved the view layer ({N_VIEW_FILES} files) into member keel-view; one line kept it above the guards - the census reading the list from guards.rs, a locked file.</p>
<p><strong>What changes:</strong> no guard. The list is declared in keel-schema, re-exported at its old path; its file joins the lock; the renderer's process and skill name the member path.</p>
{figN1}
{figN2}
{figN3}
{courses([
    ("Accept (recommended)", "one home; the lock follows", "a new guard edits two locked files"),
    ("Leave it in guards.rs", "keel-view reads the crate above it", "the forbidden cycle"),
    ("Do nothing", "nothing", "landed, unsigned"),
])}
<p><strong>True:</strong> {N_GUARDS} names in the new file and in <q>keel version</q>; {N_RENAMES} renames, {GNL["deleted"]} deletions; keel-view depends on {len(VIEW_DEPS)} members, nothing above; {PTS2} points, {N_RETRO2} findings, three Issues. <strong>Mine:</strong> a name is a fact, an arm its enforcement. <strong>What decides it:</strong> may a locked declaration move to a file the lock did not name until this commit. <strong>Wrong if</strong> a name can leave the new file without the lock firing.</p>
{opts("ask-guardnames", "d0503", [
    ("Accept (recommended)", "Accept: GUARD_NAMES is declared in members/keel-schema/src/guard_names.rs and re-exported by keel-cli/src/guards.rs under its old path; guard_names.rs joins GUARD_SOURCE_FILES so removing a name is the same locked edit as deleting its arm; the stpa-diagram process and skill name members/keel-view/src/view/stpa_diagram.rs; no guard's name, dispatch, severity or gate set changes (recommended)"),
    ("Leave the list in guards.rs", "Reject the descent: keep GUARD_NAMES declared in keel-cli/src/guards.rs and have the view member's proof census take the list as an argument from keel-cli instead"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the move and the lock entry stay as landed"),
])}""",
    "verifierstop": f"""<h2>A verifier's stop is never a block; a verifier that wrote is a counted line</h2>
{clause(VS_)}
<p><strong>What happened:</strong> blocked over the primary's red and ordered green, the verifier edited a record with an invented finding.</p>
<p><strong>What changes:</strong> the verifier's write becomes a counted line beside the recorder's red-tree line; the recorder keeps its block.</p>
{figV1}
{figV2}
{figV3}
{courses([
    ("Accept (recommended)", "never blocked; a write is counted", "the census row must be read"),
    ("Keep the block, reword it", "the block says stop, not fix", "asks an agent to disobey"),
    ("Do nothing", "nothing", "the next red tree repeats this"),
])}
<p><strong>True:</strong> live, a moved verifier start gave exit 0, one refused line, no block; an unmoved start, silence; {N_BLOCKS_DAY} blocks that day; {N_ROUTE_ARMS} route arms, {N_ROUTE_TESTS} tests. <strong>Mine:</strong> that six of those blocks were the verifier's - the ledger names sessions, not agents. <strong>What decides it:</strong> may a hazard control drop its block for one actor. <strong>Wrong if</strong> a verifier's write should ever be forced green.</p>
{opts("ask-verifierstop", "d0502", [
    ("Accept (recommended)", "Accept: for a stop payload whose agent_type is verifier the subagent-stop hook never emits a block - unmoved own start is silent, a moved one is exit 0 with one systemMessage and a refused-class ledger line under verifier:tree-written declared in the control map; the recorder keeps its block under recorder:tree-red (recommended)"),
    ("Keep the block, reword it", "Reject the route: keep blocking the verifier and change the block's text to tell it to stop instead of fixing the tree"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the route and its declarations stay as committed"),
])}""",
    "ownstart": f"""<h2>A subagent is measured from its own start</h2>
{clause(OS_)}
<p><strong>What happened:</strong> every subagent was measured from the session's first fire, hours old; the primary's edits since were charged to it.</p>
<p><strong>What changes:</strong> only its own writes reach an agent's stop gate; the start is the seventh registered hook event.</p>
{figO1}
{figO2}
{figO3}
{courses([
    ("Accept (recommended)", "each agent measured from its own start", "one file per agent"),
    ("Reset the baseline per dispatch", "the primary rewrites it first", "manual, where the harness already fires"),
    ("Do nothing", "nothing", "agents stay gated over the primary's edits"),
])}
<p><strong>True:</strong> live, a start fire wrote the agent's file ({FP_LEN} characters); a second left it; the unmoved stop was silent; {N_START_REAL} real starts ledgered since; all {N_BLOCKS_EVER} blocks ever were session-measured. <strong>Mine:</strong> that every fire inside a subagent carries its id - the harness's note says so. <strong>What decides it:</strong> may a hazard control's measured interval move without your word. <strong>Wrong if</strong> fires arrive without an agent id - the start count then stalls at {N_START_REAL}.</p>
{opts("ask-ownstart", "d0501", [
    ("Accept (recommended)", "Accept: the dispatcher writes agent-{{agent_id}}.fp on the first fire naming an agent that is not its stop, never overwrites it, and the subagent-stop hook measures that agent against its own start, falling back to the session baseline only for an agent no fire named; SubagentStart is registered as the seventh hook event and counted as a control event (recommended)"),
    ("Reset the baseline per dispatch", "Reject the per-agent file: keep one session baseline and have the primary rewrite it before each dispatch"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the dispatcher write and the registration stay as committed"),
])}""",
    "pairfile": f"""<h2>The verifier names the pair file; it never retypes the pair</h2>
{clause(PF)}
<p><strong>What happened:</strong> three verifiers retyped the primary's two command lines wrong into a comma-separated argument; each red cost an eighty-second climb and a re-dispatch.</p>
<p><strong>What changes:</strong> the primary writes the pair as a two-line file; the verifier passes its path. Any other shape is refused at parse.</p>
{figF1}
{figF2}
{figF3}
{courses([
    ("Accept (recommended)", "the pair is carried, never retyped", "one file per dispatch"),
    ("Tighten the brief instead", "the brief says retype exactly", "the fourth wrong copy"),
    ("Do nothing", "nothing", "flag stays; brief and skill unsigned"),
])}
<p><strong>True:</strong> {N_REFUSALS} wrong shapes refused live, exit 2, receipt untouched; the latest ladder - {N_PAIR_LADDERS} in a row have used it - ran its pair from <q>{PAIR_FILE}</q>, {N_RUNGS_PF} rungs green. <strong>Mine:</strong> a path is the one carrier a verifier cannot mis-copy. <strong>What decides it:</strong> may a subagent retype an argument another actor chose. <strong>Wrong if</strong> a verifier ever needs a pair the primary did not write.</p>
{opts("ask-pairfile", "d0500", [
    ("Accept (recommended)", "Accept: keel verify --probe-from FILE reads the known-positive then the known-negative from two lines the primary wrote; any other shape or --probe beside it is refused at parse before a rung runs; the verifier brief's pair slot is the file's path and the test-verify skill launches with it (recommended)"),
    ("Tighten the brief instead", "Reject the file: keep --probe POSITIVE,NEGATIVE as the verifier's carrier and have the brief spell the transcription rule instead"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the flag, brief and skill stay as committed"),
])}""",
    "guardtests": f"""<h2>The guard source's own tests follow the scratch rule</h2>
{clause(GS)}
<p><strong>What happened:</strong> the scratch rule rewrote {N_SCRATCH_SITES} test sites; {N_SITES_G} sit in the guard source's test module, a file locked whole. The lock fired on {N_HUNKS} one-line edits below line {FIRST_CFG}.</p>
<p><strong>What changes:</strong> no guard; this record is the lock's required signature for those {N_HUNKS} lines only.</p>
{figG1}
{figG2}
{figG3}
{courses([
    ("Accept (recommended)", "one rule for the whole file", "a held record per test edit there"),
    ("Narrow the lock to the predicate region", "the lock stops at the first test marker", "a lock change; a second brief"),
    ("Do nothing", "nothing", "seven lines unsigned; obligation open"),
])}
<p><strong>True:</strong> {N_HUNKS} insertions, {N_HUNKS} deletions, all in the test region; {GSG['fixedJoinsInTestRegion']} fixed-name joins remain; census and pair in the tree; the yielded-red obligation opens on this lock. <strong>Mine:</strong> a test-region carve-out is itself a lock change needing this signature. <strong>What decides it:</strong> may enforcement-surface test code change under a lighter rule than its predicates. <strong>Wrong if</strong> any hunk sits above line {FIRST_CFG}.</p>
{opts("ask-guardtests", "d0499", [
    ("Accept (recommended)", "Accept: the seven test-code scratch sites in keel-cli/src/guards.rs are rewritten to keel_fs::scratch, nothing above the file's first #[cfg(test)] line changes, and this Decision is the co-committed process-change record the lock asks for, governing those seven lines only (recommended)"),
    ("Narrow the lock to the predicate region", "Amend: accept the seven lines and ask for a further Decision making the process-change lock read only the region above a locked file's first #[cfg(test)] line"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the seven lines stay as landed"),
])}""",
    "wait": f"""<h2>The wait for a launched ladder must be a command</h2>
{clause(WF)}
<p><strong>What happened:</strong> a verifier read the running stub seconds after launch and called a green ladder killed. The wait was a sentence.</p>
<p><strong>What changes:</strong> <q>verify --wait</q> blocks while the writer lives, prints the finished table, calls a dead writer KILLED, never a verdict. Both read steps open with it.</p>
{figW1}
{figW2}
{figW3}
{courses([
    ("Accept (recommended)", "an early read is impossible, not forbidden", "one command per read step"),
    ("Refuse at the recorder instead", "the wait stays a sentence; a running stub is refused at recording", "misread later"),
    ("Do nothing", "nothing", "the next early read is the next false KILLED"),
])}
<p><strong>True:</strong> the control is pure over receipt text and a liveness answer; {N_PROBE_TESTS} tests hold its answers; the refusal ran live; {N_WAIT_STEPS} steps open with it. <strong>Mine:</strong> liveness, not age, separates in flight from killed. <strong>What decides it:</strong> may a subagent's wait be a skippable sentence. <strong>Wrong if</strong> a live pid outlives a finished ladder.</p>
{opts("ask-wait", "d0497", [
    ("Accept (recommended)", "Accept: the wait for a launched ladder is a command, keel verify --wait - it blocks while the receipt's writer pid is alive, prints the finished table and exits as the ladder did, reports a dead writer as KILLED during <rung> with exit 2 and never a verdict, and the test-verify skill's steps 4 and 5 open with it (recommended)"),
    ("Refuse at the recorder instead", "Reject the command: keep the wait as a skill sentence and have the recorder refuse a receipt whose outcome is running"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the command and the skill steps stay as committed"),
])}""",
    "probes": f"""<h2>The probes CI runs must run before the commit</h2>
{clause(PR)}
<p><strong>What happened:</strong> two commits landed green locally; CI failed both on a script probe whose list lived only in its yaml.</p>
<p><strong>What changes:</strong> one runner discovers every marked script; CI's step and the hook both call it. Python missing refuses, naming the remedy.</p>
{figP1}
{figP2}
{figP3}
{courses([
    ("Accept (recommended)", "one list, both surfaces; a red probe aborts the commit", f"{PROBES_S} s on commits that stage a script"),
    ("Run it on every commit", "the same runner, no staged-path test", f"{PROBES_S} s on every commit"),
    ("Do nothing", "nothing", "CI stays the first to run the probes"),
])}
<p><strong>True:</strong> the pair passes live; {N_PROBES} probes green in {PROBES_S} s; {N_RED_RUNS} CI runs red on the old step; {N_PY_BLOCKS} python-gated hook blocks. <strong>Mine:</strong> only a script edit can move the verdict. <strong>What decides it:</strong> may a gate exist on one surface only. <strong>Wrong if</strong> a probe fails on a change outside scripts.</p>
{opts("ask-probes", "d0496", [
    ("Accept (recommended)", "Accept: the script probes CI runs run before the commit - one runner, scripts/script_probes.py, discovers every marked script for ci.yml and for the pre-commit hook whenever a staged path matches scripts/**/*.py; a red probe aborts the commit; python missing is GATE CANNOT RUN, never a skip (recommended)"),
    ("Run it on every commit", "Amend: keep the one runner but drop the staged-path test so the hook runs the probes on every commit"),
    ("Do nothing", "Do nothing: the Decision stays proposed and on this page; the runner, step and hook stay as committed"),
])}""",
    "triple": f"""<h2>The lint before a push must see what CI compiles</h2>
{clause(CT)}
<p><strong>What happened:</strong> a delivery landed green through local clippy; CI's Linux clippy failed it on a body the Windows host never compiles.</p>
<p><strong>What changes:</strong> the clippy rung and the hook lint for the host and again for the Linux triple; a host without that std is red naming the remedy, never skipped.</p>
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
<p><strong>True:</strong> the binary's own test passes live; {STEMS_TRUE}. <strong>Mine:</strong> fix the procedure now; the receipt half sits {POS94} of {N_NEXT} in next-work. <strong>What decides it:</strong> may a verifier follow a row naming a function, or only a receipt stating its own attribution. <strong>Wrong if</strong> the next false MISMATCH is a stem the row omits.</p>
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

<div class="ask"><p class="verdict"><strong>Accept all fifteen.</strong></p></div>
<div class="chips"><span class="chip"><b>Waiting</b> {pending}</span><span class="chip"><b>Safety changes</b> 2</span><span class="chip"><b>Forks</b> 0</span><span class="chip"><b>Guard names</b> {N_GUARDS}</span></div>

<div class="tabs" role="tablist" aria-label="The asks">{tabs_html}</div>
{panels_html}
<label class="note-row">Notes<textarea data-d="note" rows="2" placeholder="optional"></textarea></label>
<div class="copy-bottom"><button class="copy" data-copy type="button">&#8681; Copy for AI</button></div>
<footer data-digest="provenance">Computed at {TREE}, {DATE}; {dirty} uncommitted files; {tests} tests, {failing} failing; CI red on {N_CI_RED} of {len(CI_PUSH)} pushes. <a href="https://github.com/williamweatherholtz/sysmlv2-ai-toolkit" target="_blank" rel="noopener">repository</a>.</footer>
</div>
"""


TITLE = "Accept fifteen: reviews finish by analysis"
SUB = "Fifteen held, no fork"


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
      f"; summed {words(page)}; {pending} held; probes {N_PROBES} in {PROBES_S} s (pair {PAIR_S} s), {N_RED_RUNS} CI reds on the old step; "
      f"clippy {CLIPPY_S} s of {LADDER_S} s; {N_RUNGS} rungs; class {N_CLASS}; wait {POS_WAIT_NOW} of {N_BACKLOG}; "
      f"{N_TESTS_LANDED} tests under the row; sprint results 726 {N_RESULTS}+{N_GATE_RESULTS}, 727 {N_RESULTS7}+{N_GATE_RESULTS7}, "
      f"728 {N_RESULTS8}+{N_GATE_RESULTS8}, 729 {N_RESULTS9}+{N_GATE_RESULTS9}; wait: {N_PROBE_TESTS} tests, {N_WAIT_STEPS} steps, epoch item {POS_EPOCH}, git-read item {POS_GITREAD}; "
      f"guard tests: {N_SITES_G} of {N_SCRATCH_SITES} sites, {N_HUNKS} hunks below line {FIRST_CFG}, worktree item {POS_WORKTREE}; "
      f"pair file: {N_REFUSALS} refusals, probe {PROBE_S_PF} s of {LADDER_S_PF} s from {PAIR_FILE}, {N_RUNGS_PF} rungs, sprint 730 {N_RESULTS0}+{N_GATE_RESULTS0}, stop-hook item {POS_STOP} of {N_BACKLOG_PF}; "
      f"own start / verifier stop: {N_PROBE_FIRES} live fires, fp {FP_LEN} chars, {N_ROUTE_ARMS} arms, {N_ROUTE_TESTS} tests, {N_HOOK_RULES} hook-rules, "
      f"{N_START_REAL} real starts, {N_VER_WRITTEN} verifier writes, {N_REC_RED} recorder reds, {N_BLOCKS_DAY} of {N_BLOCKS_EVER} blocks on the day, {N_FP_FILES} fp files, "
      f"sprint 731 {N_RESULTS1}+{N_GATE_RESULTS1}, view item {POS_VIEW} and suite item {POS_SUITE} of {N_BACKLOG_SV}; "
      f"guard names: {N_GUARDS} names, {N_LOCK_PATHS} locked sources, {N_MEMBERS} members, view {N_VIEW_FILES} files / {N_VIEW_LINES} lines / {N_VIEW_SUB} submodules, "
      f"landed {N_FILES_CHANGED} files ({N_RENAMES} R, {N_MODIFIED} M, {N_ADDED} A, +{N_INS}/-{N_DEL}), sprint 732 {N_RESULTS2}+{N_GATE_RESULTS2} ({PTS2} pts, {N_RETRO2} findings), "
      f"suite {POS_SUITE2}, families {POS_FAMILIES}, layering {POS_LAYERING}, quoted-line {POS_QUOTED}, root helper {POS_ROOT} of {N_BACKLOG_GN}; "
      f"guard source: {N_LOCK_FILES} locked files + {N_LOCK_DIRS} dir, {N_FAMILIES} families, {N_MEMBER_FILES} files / {N_MEMBER_LINES} lines ({N_DEFINING} defining), "
      f"old file {OLD_LINES} lines, {N_WS_MEMBERS} members, landed {N_FILES3} files ({N_RENAMES3} R, {N_MODIFIED3} M, {N_ADDED3} A, {N_DELETED3} D, +{N_INS3}/-{N_DEL3}), "
      f"{N_LANDED_TESTS}/{N_LANDED_FAILED} tests, sprint 733 {N_RESULTS3}+{N_GATE_RESULTS3} ({PTS3} pts, {N_RETRO3} findings), "
      f"root {POS_ROOT3}, suite {POS_SUITE3}, catalogue {POS_CATALOGUE}, build-script {POS_BUILDSCRIPT}, range {POS_RANGE} of {N_BACKLOG_GS}; "
      f"locked anchor: {LAS['repoRootDefinitionCount']} definition, {N_USE_FILES} use lines, {N_CLI_CALLERS} cli callers, {N_TEST_ANCHORS} test anchors, "
      f"adherence +{LAL['adherenceInsertions']}/-{LAL['adherenceDeletions']}, landed {N_FILES4} files ({N_MODIFIED4} M, {N_ADDED4} A, +{N_INS4}/-{N_DEL4}), "
      f"{N_LANDED_TESTS4}/{N_LANDED_FAILED4} tests, sprint 734 {N_RESULTS4}+{N_GATE_RESULTS4} ({PTS4} pts, {N_RETRO4} findings), "
      f"root {POS_ROOT4}, suite {POS_SUITE4}, process {POS_PROCESS4} of {N_BACKLOG_LA}; "
      f"suite member: {len(SMS['memberDeclaresTheFive'])} modules ({len(SML['movedWhole'])} whole), {N_PATH_DEPS} path deps + {N_CRATE_DEPS} crates, {N_WS_MEMBERS_SM} members, "
      f"locked -{N_LOCKED_OUT}/+{N_LOCKED_IN} over {len(SML['lockedFilesTouched'])} files, landed {N_FILES5} files ({N_RENAMES5} R, {N_MODIFIED5} M, {N_ADDED5} A, +{N_INS5}/-{N_DEL5}), "
      f"{N_LANDED_TESTS5}/{N_LANDED_FAILED5} tests ({LANDED_ROW5}), sprint 735 {N_RESULTS5}+{N_GATE_RESULTS5} ({PTS5} pts, {N_RETRO5} findings, {N_RETRO_NOT_TRACKED5} not tracked), "
      f"process {POS_PROCESS5}, issues {POS_ISSUES5}, measure {POS_MEASURES5}, delivers {POS_DELIVERS5}, thin {POS_THIN5} of {N_BACKLOG_SM}; "
      f"issues member: {len(IMS['memberDeclaresTheFour'])} modules ({len(IML['renamedFrom'])} moved as files), {N_PATH_DEPS8} path deps + {N_CRATE_DEPS8} crate, {N_WS_MEMBERS_IM} members, "
      f"locked -{N_LOCKED_OUT8}/+{N_LOCKED_IN8} ({_LIB08['deletions']}/{_LIB08['insertions']} lib.rs, {_ISS08['deletions']}/{_ISS08['insertions']} issues.rs), "
      f"landed {N_FILES8} files ({N_RENAMES8} R, {N_MODIFIED8} M, {N_ADDED8} A, +{N_INS8}/-{N_DEL8}), {N_LANDED_TESTS8}/{N_LANDED_FAILED8} tests ({LANDED_ROW8}), "
      f"sprint 737 {N_RESULTS8}+{N_GATE_RESULTS8} ({PTS8} pts, {N_RETRO8} findings, {N_RETRO_NO_ITEM8} already tracked, {N_RETRO_NOT_TRACKED8} not tracked), "
      f"issues {POS_ISSUES8}, touched {POS_TOUCHED8}, measure {POS_MEASURES8}, citations {POS_CITATIONS8}, delivers {POS_DELIVERS8}, serve {POS_SERVE8}, thin {POS_THIN8} of {N_BACKLOG_IM}")
assert_fits(out_path)   # probe first, then measure in a browser with and without web fonts; findings remove the page
