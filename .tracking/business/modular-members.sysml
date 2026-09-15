// =====================================================================
// ProjectBusinessModularMembers - Business-workflow artifacts for the modularity program st126 named
// (2026-09-14): keel as granular workspace members, a keel-issues member for the item/issue process
// subset, and custom processes hooked with absolute discipline. Stories are in
// .tracking/intake/intake-2026-09-14.sysml (record story, #DerivedFrom st126). Every Need here has a
// #DerivedFrom edge to its story; a threshold st126 did not give is marked (my judgment) in the statement.
// D0479 (accepted) chartered the extraction already worked (sprints 714-719, D0064 decision-driven); these
// Needs are the contract the REMAINING extraction and the keel-issues member satisfy. No SystemRequirement
// or architecture Decision for keel-issues or the hook until the Business gate below is a human pass.
// =====================================================================
package ProjectBusinessModularMembers {
    private import EngineElement::*;
    private import EngineNeeds::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    part modularMembersBrief : Brief {
        :>> id = "e2ec5a22-ece2-4741-8f23-dc287bb55ecc";
        :>> title = "keel as granular members, keel-issues, disciplined custom processes - brief";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> problem = "keel-cli/src still holds 53,587 lines after eleven members took 13,958 (counted on this tree at 4d38dd7): an edit there relinks the whole binary, a running binary blocks its own rebuild (issue150), and the touched rung of one verify ran 562 s at d110d0a (834 tests). The item/issue processes a project wants from a tracker ship fused with the engine-governance surface (Decisions about keel, Change Requests, the CR guards), so a project that only tracks items carries all of it. A project that authors its own process in plain SysML has no stated way to make the engine enforce that process as hard as it enforces its own (st126: how to hook into custom processes that have absolute discipline?).";
        :>> opportunity = "Finish the D0479 layering as a contract, not a chore: each member is one function with its own manifest, rebuilds alone, and is a boundary a guard, hook or sensor can bind to (st126: anchor points into our processes). Carve keel-issues so the tracker subset stands on its own. Make process hooking a declared, guarded path - the same refusal in the editor hook, at the commit gate and in CI (gate-environment-parity) - so a downstream project gets the discipline keel gives itself.";
        :>> constraintsNote = "D0479 stands (accepted) and eleven members exist; nothing here reopens the layering. Deliverable vocabulary stays out of the engine. Frozen schema/core: any new process-hook construct is a Change Request. Authoring friction is the #1 risk (D0054): a member split that adds a step to record a fact is a regression, not modularity.";
    }

    part pMaintainer : Persona {
        :>> id = "19ea86ca-c6d6-4579-b462-bfef7c8a09b8";
        :>> title = "keel maintainer";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> description = "The person (and the AI actor working with them) who edits keel and lands it several times a day, and reads the engine's own processes to know what a change costs. Pays the rebuild on every edit and the touched-test set on every land.";
        :>> goals = "Rebuild only what an edit touched; know from the manifest, not from reading code, which member a module lives in and what it may depend on; put a control at a member boundary and have it hold.";
    }
    part pDownstreamProjectLead : Persona {
        :>> id = "e485b82e-9ff7-4891-b9e3-c9895838c658";
        :>> title = "downstream project lead";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> description = "Someone running a project on keel who tracks items and issues and authors that project's own process in plain SysML. Does not change keel and does not want the engine's self-governance (Decisions about keel, Change Requests) in their tree or their binary.";
        :>> goals = "Track items/issues with computed state from a small member; author a custom process and have its gates enforced absolutely without an engine code change.";
    }

    // -- Needs (solution-free, measurable; MoSCoW). Accept these BEFORE architecture. A threshold st126 did not give is marked (my judgment). --

    requirement nMemberRebuildIsLocal : Need {
        :>> id = "3bc7a73f-d228-4a90-a01b-524c866a8cd2";
        :>> title = "An edit rebuilds its member and the members above it, nothing else";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::operator;
        :>> statement = "After an edit inside one workspace member, a build compiles that member and the members that depend on it; every member beside or below it reads Fresh (cargo build -v). Measured on a one-line edit to a leaf and to keel-model. The Fresh criterion is my judgment of 'easy to rebuild'; st126 gave no number.";
        :>> priority = Priority::must;
    }
    requirement nMemberDependsDownOnly : Need {
        :>> id = "bb2bec5d-131e-4747-a2fc-0c690518dc6b";
        :>> title = "Every member declares its dependencies and depends only downward";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::internal;
        :>> statement = "Each member's Cargo.toml names every member it uses; no member depends on a member in the same layer or a higher one (D0479 layering), and the check is a guard, not a review. cargo test -p <member> passes with only its declared dependencies built.";
        :>> priority = Priority::must;
    }
    requirement nEveryModuleHasAMemberHome : Need {
        :>> id = "a0beab7e-5dfa-4326-910b-2ebca249abce";
        :>> title = "Every function of the engine has a member home or a stated reason to stay in the binary";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::internal;
        :>> statement = "Each top-level module still in keel-cli/src is either scheduled to a member by a backlog item or carries a stated reason it belongs to the binary (dispatch, main). Count of modules with neither: 0. 'Lots of granularity' is st126's; the by-function criterion is my judgment.";
        :>> priority = Priority::should;
    }
    requirement nMemberIsAnAnchor : Need {
        :>> id = "59ff739f-b4cf-46c8-8e90-5316e79d366a";
        :>> title = "A member boundary is a place a control binds";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::internal;
        :>> statement = "A guard, hook or sensor can be declared against one member (its crate, its tests, its receipts) and holds there; the control census (keel show control-census) names the member each control binds to, and a control that names no member is listed. My reading of st126's 'anchor points into our processes'.";
        :>> priority = Priority::should;
    }
    requirement nIssuesSubsetIsItsOwnMember : Need {
        :>> id = "d1ac30c7-4412-4b97-9ac6-003f4638648c";
        :>> title = "The item/issue processes run from a member without the engine-governance surface";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::customer;
        :>> statement = "The verbs about items and issues (record item/issue/task, resolve, ready, blocked, done, whats-next, open-issues) build and run from one member (keel-issues, st126's name) whose dependency closure contains no Decision-about-keel, Change Request or CR-guard code. A project tree holding only items and issues validates and orients with that member alone.";
        :>> priority = Priority::must;
    }
    requirement nStateIsComputedAndExposed : Need {
        :>> id = "3a524977-ffbb-4cd1-9c28-cb31af2397dd";
        :>> title = "The tracker exposes computed state, never stored";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::customer;
        :>> statement = "Every state the member exposes (ready, blocked, done, open, resolved, who owns, since when) is computed from the text and available as CLI and JSON; no item carries a stored status the computation could contradict (the existing D0018 invariant, held for the member).";
        :>> priority = Priority::must;
    }
    requirement nProcessIsReadNotHardcoded : Need {
        :>> id = "a0d5ecbd-0263-4016-b115-54eff7f532b4";
        :>> title = "A process is plain SysML the engine reads; adding one changes no engine code";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::customer;
        :>> statement = "A project authors a process (steps, gates, the Tests that verify each gate) in plain SysML and the engine enforces it - keel advance refuses a red step, keel gate reports it - with zero edits to keel. Measured on a fixture project with a process the engine has never seen.";
        :>> priority = Priority::must;
    }
    requirement nCustomProcessRefusesEverywhere : Need {
        :>> id = "72bb097a-cfda-4ceb-844b-f07c7e2a2208";
        :>> title = "A custom process is enforced the same in the hook, at the commit gate and in CI";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> source = NeedSource::customer;
        :>> statement = "A red gate in a project's custom process is refused in the editor hook, at the commit gate and in CI alike, with the same verdict text; a step cannot be advanced past a red gate in any of the three. 'Absolute discipline' is st126's; the three-environment parity is my measurable form of it.";
        :>> priority = Priority::must;
    }

    // -- Use cases --

    use case ucOneLineEditRebuildsOneMember : UseCase {
        :>> id = "8b7bfa67-8c40-4971-b817-b3a6f4957b58";
        :>> title = "A one-line edit rebuilds one member";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> description = "The maintainer fixes one line in a guard and rebuilds the release binary.";
        :>> mainSuccess = "cargo build --release compiles the guard member and the binary above it; every other member is Fresh; the land's touched set descends from the changed member, not the whole workspace.";
        :>> extensions = "The edit is in a leaf (keel-fs): the members above it rebuild, the count is read from cargo -v, and the receipt names them.";
    }
    use case ucProjectTracksItemsFromKeelIssues : UseCase {
        :>> id = "6250760b-1cad-4461-b030-9a8da47b700f";
        :>> title = "A project tracks items and issues from keel-issues alone";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> description = "A downstream project lead records items and issues, resolves them, and reads the frontier, using a tree that holds no Decision about keel and a binary built from keel-issues.";
        :>> mainSuccess = "record item/issue, resolve, whats-next and open-issues all answer from the member; the tree validates; no Change Request or CR guard is present in the binary.";
        :>> extensions = "The project records a Decision about ITS OWN work: where that falls (in keel-issues, or in a separate member) is the read-back question to the stakeholder, not decided here.";
    }
    use case ucCustomProcessHookedWithDiscipline : UseCase {
        :>> id = "e9691bb4-9399-4ad4-94a1-490459d5d2a9";
        :>> title = "A custom process is hooked and refused everywhere";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> description = "A downstream project lead authors a three-step process in plain SysML, binds a Test to each gate, and advances through it.";
        :>> mainSuccess = "With step 2's Test red, keel advance refuses in the editor hook, the commit gate refuses, CI refuses, all with one verdict text; with it green all three pass and the cursor moves.";
        :>> extensions = "The process names a gate no Test verifies: the engine reports the gate as unverifiable at author time, before any advance is attempted.";
    }

    // -- Lineage: each Need derives from the story that elicited it (business-elicitation beAuthorNeeds) --
    #DerivedFrom dependency from nMemberRebuildIsLocal to us111;
    #DerivedFrom dependency from nMemberDependsDownOnly to us111;
    #DerivedFrom dependency from nEveryModuleHasAMemberHome to us111;
    #DerivedFrom dependency from nMemberIsAnAnchor to us114;
    #DerivedFrom dependency from nIssuesSubsetIsItsOwnMember to us112;
    #DerivedFrom dependency from nStateIsComputedAndExposed to us112;
    #DerivedFrom dependency from nProcessIsReadNotHardcoded to us112;
    #DerivedFrom dependency from nProcessIsReadNotHardcoded to us113;
    #DerivedFrom dependency from nCustomProcessRefusesEverywhere to us113;

    // -- Business gate: human acceptance of the Needs (method=confirmation). No result until wweatherholtz's word on THESE eight; an AI cannot supply it (D0051). --
    verification modularMembersNeedsAccept : Test {
        :>> id = "a6ee32cc-a58d-4ae3-8f04-3aa101915dcf";
        :>> title = "modular-members Needs accepted (Business gate)";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> method = VerificationMethod::confirmation;
        :>> procedureText = "wweatherholtz accepts the eight modular-members Needs (nMemberRebuildIsLocal, nMemberDependsDownOnly, nIssuesSubsetIsItsOwnMember, nStateIsComputedAndExposed, nProcessIsReadNotHardcoded, nCustomProcessRefusesEverywhere [must]; nEveryModuleHasAMemberHome, nMemberIsAnAnchor [should]) in their own words, after the read-back asks what is WIDER than they meant (beConfirmNarrower). Recorded with keel accept / record result method=confirmation quoting those words. Only then may SystemRequirements and the keel-issues / process-hook architecture Decisions be derived.";
    }
    // RAN: Selected on the decision brief copy-out (twenty-ninth publish, built at cfb1cb8), pasted into chat 2026-09-14 by wweatherholtz; the read-back asked what was WIDER than meant and Accept as written was marked on all three tabs, Narrow / Fix a threshold / Hold on none. The reading of the marks as their acceptance was stated back in chat with the offer that one word stops it; their next message was 'Sure accept B then.' - their marked lines, verbatim: '[x] Accept as written: the 3 Needs on this tab (nMemberRebuildIsLocal, nMemberDependsDownOnly, nEveryModuleHasAMemberHome) say what I meant; record my confirmation of the Business gate for them' / '[x] Accept as written: the 3 Needs on this tab (nIssuesSubsetIsItsOwnMember, nStateIsComputedAndExposed, nMemberIsAnAnchor) say what I meant; record my confirmation of the Business gate for them' / '[x] Accept as written: the 2 Needs on this tab (nProcessIsReadNotHardcoded, nCustomProcessRefusesEverywhere) say what I meant; record my confirmation of the Business gate for them'.
    part modularMembersNeedsAcceptR1 : TestResult { :>> id = "06fa8f1d-28f2-49f3-b350-146c5e9b0185"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "2beaa66"; :>> judgedAt = "2026-09-14"; :>> judgedBy = "wweatherholtz"; }
}
