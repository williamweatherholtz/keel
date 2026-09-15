// =====================================================================
// ModularMembersRequirements - the eight accepted Needs of the st126 modularity program
// (.tracking/business/modular-members.sysml, Business gate a human pass at 2beaa66) decomposed into
// contracts a run meets or does not. Architecture phase, sprint 721, chartered by D0479.
//
// Every THRESHOLD is a count, a build-tool verdict or a byte comparison - never a review. Where the
// accepted Decisions D0479-D0486 already bind a threshold the requirement says so with a #DerivedFrom
// edge instead of restating the Decision. The three SubsystemRequirements are the technical choices
// (where the layer table lives, how a step binds to a project's Test, what one gate rung is); the
// process-binding ones are also the subject of the three held process-change Decisions recorded beside this file (D0488 -> D0489 -> D0490); the keel-issues item verbs also carry the held fork D0487.
//
// ORDER IS DELIBERATE: the layering pair first (the guards everything else is measured against), the
// module-home and anchor pair, then the keel-issues pair, then the process pair the hook Needs name.
// =====================================================================
package ModularMembersRequirements {
    private import EngineElement::*;
    private import EngineRequirements::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;
    private import ProjectBusinessModularMembers::*;

    // ---- dataArch: SystemRequirements ---------------------------------------------------------

    part srMemberEditRebuildsOnlyItsCone : SystemRequirement {
        :>> id = "56d2c259-67dd-4f57-95ee-7fa40754a996";
        :>> title = "A one-line edit inside a member recompiles that member and its dependents; every other member is Fresh";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "After a one-line edit inside workspace member M, `cargo build -v` reports Compiling for M and for every member whose Cargo.toml reaches M through [dependencies], and Fresh for every other member. THRESHOLD: measured on two edits, one to a leaf member (keel-json) and one to keel-model; the count of members reported Compiling that are outside M's reverse-dependency cone is 0 in both, and the cone is read from `cargo tree -i -p M`, not from a list a person keeps.";
        :>> rationale = "st126: 'make it easy to rebuild'. The Fresh criterion is the build tool's own word for 'not rebuilt', so the requirement is a verdict cargo emits, not a timing.";
        :>> kind = RequirementKind::nonFunctional;
        :>> priority = Priority::must;
    }

    part srMemberDependenciesAreDeclaredAndDownward : SystemRequirement {
        :>> id = "ee9e62a0-f2fa-41ea-a5a4-9b6287b6d9fe";
        :>> title = "Every member depends only on members in a lower declared layer, and a guard refuses the edge that does not";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "Each workspace member's Cargo.toml [dependencies] names every member crate it uses, and a forward guard (member-layering) fails any dependency from a member to a member in the same or a higher layer of the declared layer order. THRESHOLD: on this tree the guard reports 0 violations; on the D0388 known-positive (a fixture manifest that points one layer up) it reports 1 and exits red; `cargo test -p <member>` passes for every member with no other member's tests run.";
        :>> rationale = "The D0479 layers are the whole basis of local rebuilds; a layer order kept by review re-fuses the crate one convenience edge at a time (the pattern that put 53,598 lines back into keel-cli).";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    part srEveryCliModuleHasAHome : SystemRequirement {
        :>> id = "08f79c83-da0a-443e-9710-f0ba8dc37911";
        :>> title = "Every top-level module still in keel-cli/src is scheduled to a member or names its reason to stay";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "For every top-level module of keel-cli/src (each `<name>.rs` and each `<dir>/mod.rs`), either an open backlog item's DoD names the module and the member it moves to, or the module is on the thin-dispatch list the workspace manifest declares ([workspace.metadata.keel] stays-in-binary, with a one-line reason each). THRESHOLD: the count of modules with neither, computed by a check over `scripts/module_home.py`'s rust_sources() and the backlog, is 0; the check is a forward guard or a probe in the verify ladder, not a review.";
        :>> rationale = "st126: 'lots of granularity'. Without a by-module census the extraction ends where energy ran out; a stated reason for every module that stays makes the remainder a decision, not a residue.";
        :>> kind = RequirementKind::nonFunctional;
        :>> priority = Priority::should;
    }

    part srControlCensusNamesTheMember : SystemRequirement {
        :>> id = "4917af1a-33c4-4aa6-b9a9-18b9a7311f6d";
        :>> title = "Every control names the workspace member it binds to, and the census lists the ones that name none";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "`keel show control-census` carries a member field per control, computed from the crate that holds the control's code, its tests or the receipt it writes (a guard, a hook event, a sensor); a control whose crate cannot be computed appears under an unanchored list with the reason. A guard or sensor declaration may state the member it binds to (member = <crate>) and the guard scope then covers that crate's sources, tests and receipts only. THRESHOLD: the count of controls under unanchored on this tree is reported, never hidden; the census JSON schema carries member and unanchored as fields; a declared member that names no [workspace] member is refused at declaration.";
        :>> rationale = "st126: 'anchor points into our processes'. A member is the unit a person can point a control at; a census that cannot say which member a control watches cannot say what a member's removal would leave unwatched.";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::should;
    }

    part srItemVerbsBuildFromKeelIssues : SystemRequirement {
        :>> id = "fe6152c4-0c67-455d-a870-7ea0d7beb0ad";
        :>> title = "The item and issue verbs build and run from keel-issues, whose closure holds no engine-governance code";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "The verbs record issue, record task, record story, record statement, github ingest and plan cover, the resolution computation, and the lenses open-issues, intake, dispositions, whats-next, ready, blocked and done are dispatched from member keel-issues. THRESHOLD: `cargo tree -p keel-issues -e normal` names no crate that holds the Change Request process, the CR guards or the Decision-about-keel holds (keel-process and the guards member); `cargo build -p keel-issues` succeeds with the rest of the workspace absent; a fixture project tree holding only items and issues validates and orients with the keel-issues binary alone (exit 0, orient JSON with a ready list).";
        :>> rationale = "st126 named the member and the subset. The closure test is the D0480 boundary made a build fact rather than a diagram.";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    part srMemberStateIsComputedAndExposed : SystemRequirement {
        :>> id = "497c7a1a-c594-44ec-9369-07a5b18c66ad";
        :>> title = "Every state the member exposes is computed from the text and available as both CLI and JSON";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "For each of ready, blocked, done, open, resolved, owner and since-when: the value is computed from items, typed edges and results at read time; it is a field of one JSON lens and the CLI rendering of that lens; and no schema attribute stores it (the type-collision and attribute-vocabulary guards refuse a stored `status` on an item, as the engine's D0018 invariant already does for `superseded`). THRESHOLD: for each of the seven states one lens name is written down beside it; a fixture with an item whose stored text contradicts the computation is impossible to author (the attribute is refused), so the contradiction count is 0 by construction.";
        :>> rationale = "The tracker's whole claim is that its state cannot lie. Holding the engine's invariant for the member is a requirement, not an assumption, because the member will be built and used apart from the engine's guards.";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    part srProjectProcessIsReadFromItsTree : SystemRequirement {
        :>> id = "249ea143-739b-4e7e-9947-3021b238b1d3";
        :>> title = "A process declared in a project's own tree is enforced by keel advance and keel gate with no engine edit";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "`keel advance <process>` resolves a process definition declared under the project's `.tracking/processes/` exactly as it resolves one under `.engine/processes/`; a step's checkedBy may name a Test in the project's model (test:<name>), and the step is red when that Test's latest TestResult at HEAD is not a pass; advance refuses to move the cursor past a red step with the same refusal it gives an engine process. THRESHOLD: a fixture project holding one process the engine has never seen, two steps, the first bound to a failing Test - advance refuses (exit non-zero, the step named); after a passing result is recorded, advance moves; the count of edits under keel-cli/, members/ and .engine/ made to run the fixture is 0.";
        :>> rationale = "st126: 'how to hook into custom processes'. A process the engine only knows by reading it is the one property that lets a project's discipline be as hard as the engine's own.";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    part srCustomGateRefusesIdenticallyInThreeSurfaces : SystemRequirement {
        :>> id = "f2ea8c66-7e3f-4876-ab4b-5ecb5caf2372";
        :>> title = "A red step in a project process is refused by the hook, the commit gate and CI with byte-identical verdict text";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "One gate rung (process-gates) runs in the fast tier the editor hook calls, in the commit tier the pre-commit hook calls and in the CI gate job, and fails when any adopted process - engine or project - holds a step taken out of order: a later step's passing result recorded at a commit where an earlier step's bound check is red (the process cursor is computed, never stored, so the record of the later step IS the violation). THRESHOLD: on the fixture of srProjectProcessIsReadFromItsTree with a pass recorded on step two while step one's Test is red, the verdict line captured from `keel hook stop` (the hooks ledger), from `keel gate --workspace` and from the CI log is byte-identical after the path prefix is stripped; the rung is one function called from the three tiers, so parity is by construction and the comparison is its probe.";
        :>> rationale = "st126: 'absolute discipline'. Three surfaces that agree by construction is the measurable form; a rung any one tier omits is where the discipline leaks.";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    // ---- techArch: SubsystemRequirements (the technical choices) ------------------------------

    requirement ssrLayerTableLivesInTheWorkspaceManifest : SubsystemRequirement {
        :>> id = "bfa10677-c3ba-4492-a37e-6ab3bbd0f266";
        :>> title = "The layer order is one table in Cargo.toml [workspace.metadata.keel.layers], read by the guard and by scripts";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "The declared layer order of srMemberDependenciesAreDeclaredAndDownward is a single TOML table in the workspace Cargo.toml under [workspace.metadata.keel.layers], one key per member naming its layer number; the member-layering guard, scripts/module_home.py and the build skill read that table and hold no copy of it (D0486: the [workspace] members list is already the one list of crates; the layer table sits beside it). THRESHOLD: `grep` for a second layer list under .engine/, scripts/ and keel-cli/ finds 0; a member present in [workspace] members and absent from the table is a guard failure with the member named.";
        :>> rationale = "D0105, one home per fact: the layer order restated in a skill drifted from the tree once already (D0484, D0486 corrected the build skill twice in one day).";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    requirement ssrStepBindsToAProjectTest : SubsystemRequirement {
        :>> id = "94ae6760-89ec-4132-a6d6-38e5b3d513d1";
        :>> title = "A ProcessStep's checkedBy gains the binding kind test:<name>, resolved in the project's own model";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "Beside the existing bindings (a compiled guard name, a declared rule, gate:<phase>), a step's checkedBy may read `test:<name>` where <name> is a Test declared anywhere in the project's model; the step's verdict is the outcome of that Test's latest TestResult whose judgedAgainst is an ancestor of HEAD, and no result is red. The step-check-resolves guard extends to the new kind: an unresolvable test name is a hard failure with the name. THRESHOLD: the fixture of srProjectProcessIsReadFromItsTree exercises the binding both ways (refused, then passed); the unbound-name probe is red.";
        :>> rationale = "A project cannot compile a guard into keel; a Test it authors and records against is the one check it fully owns, and the engine already treats a phase gate as its verify-linked Tests passing (CLAUDE.md invariant 6).";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    requirement ssrProcessGatesIsOneRung : SubsystemRequirement {
        :>> id = "c22ea861-86d2-491c-80d8-f35b8c99ff7a";
        :>> title = "process-gates is one function, called by the fast tier, the commit tier and CI, that reports each adopted process's steps taken past a red earlier step";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> statement = "The rung of srCustomGateRefusesIdenticallyInThreeSurfaces is one function in the guards member that lists every process adopted in .engine/contracts/activation.toml (engine or project), computes each run's position with the same resolver keel advance uses, and emits one verdict line per step taken past a red earlier step in a fixed format `process-gates: <process> step <later> recorded while <earlier> is red (<check>)`. The fast tier, `keel gate --workspace` and the CI gate job call that function and print its lines unchanged. THRESHOLD: three captures of the same violation differ in 0 bytes after their path prefix; a search for a second evaluator of a step check finds only the cursor's and the rung's shared function.";
        :>> rationale = "Parity that is measured after the fact is a test that will be skipped; parity by a single implementation is what makes the byte comparison a probe and not a hope.";
        :>> kind = RequirementKind::functional;
        :>> priority = Priority::must;
    }

    // ---- satisfy: each Need is decomposed into at least one contract ---------------------------
    satisfy nMemberRebuildIsLocal by srMemberEditRebuildsOnlyItsCone;
    satisfy nMemberDependsDownOnly by srMemberDependenciesAreDeclaredAndDownward;
    satisfy nEveryModuleHasAMemberHome by srEveryCliModuleHasAHome;
    satisfy nMemberIsAnAnchor by srControlCensusNamesTheMember;
    satisfy nIssuesSubsetIsItsOwnMember by srItemVerbsBuildFromKeelIssues;
    satisfy nStateIsComputedAndExposed by srMemberStateIsComputedAndExposed;
    satisfy nProcessIsReadNotHardcoded by srProjectProcessIsReadFromItsTree;
    satisfy nCustomProcessRefusesEverywhere by srCustomGateRefusesIdenticallyInThreeSurfaces;

    // ---- the accepted Decisions that already bind a threshold ---------------------------------
    #DerivedFrom dependency from srMemberDependenciesAreDeclaredAndDownward to d0483;
    #DerivedFrom dependency from srMemberEditRebuildsOnlyItsCone to d0479;
    #DerivedFrom dependency from srItemVerbsBuildFromKeelIssues to d0480;
    #DerivedFrom dependency from ssrLayerTableLivesInTheWorkspaceManifest to d0486;
    // the three held process-hook Decisions (D0488 -> D0489 -> D0490) and the keel-issues Decision-scope fork (D0487)
    #DerivedFrom dependency from srProjectProcessIsReadFromItsTree to d0488;
    #DerivedFrom dependency from ssrStepBindsToAProjectTest to d0489;
    #DerivedFrom dependency from srCustomGateRefusesIdenticallyInThreeSurfaces to d0490;
    #DerivedFrom dependency from ssrProcessGatesIsOneRung to d0490;
    #DerivedFrom dependency from srItemVerbsBuildFromKeelIssues to d0487;

    // ---- techArch refines dataArch ------------------------------------------------------------
    #DependsOn dependency from ssrLayerTableLivesInTheWorkspaceManifest to srMemberDependenciesAreDeclaredAndDownward;
    #DependsOn dependency from ssrStepBindsToAProjectTest to srProjectProcessIsReadFromItsTree;
    #DependsOn dependency from ssrProcessGatesIsOneRung to srCustomGateRefusesIdenticallyInThreeSurfaces;

    // -- Architecture gate (allocIface): computed, method=test. The June gates of this project were the team lead's
    // confirmation; this one is the lens read back, because what it asserts (every Need decomposed, every
    // refinement typed, allocation only where a carrier exists) is a structural fact the engine computes. --
    verification modularMembersArchitectureGate : Test {
        :>> id = "f26fbdaa-de41-4f3b-8192-b8926978906d";
        :>> title = "modular-members Architecture gate: Needs decomposed, refinements typed, allocation honest";
        :>> createdAt = "2026-09-15"; :>> createdBy = "claudeFable5";
        :>> method = VerificationMethod::test;
        :>> procedureText = "keel show tier-satisfaction lists none of the eight modular-members Needs among the Need-tier gaps; each of the eight SystemRequirements appears in the SystemRequirement tiers; keel gate validate and every forward guard (edge-endpoints, resolver-kind, requirement-rootedness) are green on the tree holding this file, modular-members-components.sysml and the allocation edges; the allocation leg reports the requirements whose carrier is not yet registered as gaps rather than the tree carrying a wishful edge.";
    }
    // RAN: // RAN: KEEL verify . --probe 'python scripts/artefact.py --probe,python scripts/module_home.py --probe' -> outcome=pass stopped_at=none seconds=773 at=1789479137 (at>launch: ok) TOUCHED RECEIPT: outcome=pass passed=835 failed=0 seconds=750 at=1789479137 launch=1789478355 (at>launch: ok) stems=[init, shellcheck] changed=[init, shellcheck] (MATCH) lib=true head=9703282 log=./.keel/metrics/touched-1789478385.log runner=cargo-nextest 0.9.144 skipped=[] (no skipped binaries) test result line: 'lib::reports::tests::report_produces_cards_and_rejects_unknown (655648 ms)'
    part modularMembersArchitectureGateR1 : TestResult { :>> id = "23c84a36-15f0-4564-a0d3-88c3d45c0f5d"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "9703282"; :>> judgedAt = "2026-09-15"; :>> judgedBy = "claudeFable5"; }
    // RAN: // RAN: KEEL verify . --probe 'python scripts/artefact.py --probe,python scripts/module_home.py --probe' -> outcome=pass stopped_at=none seconds=773 at=1789479137 (at>launch: ok) TOUCHED RECEIPT: outcome=pass passed=835 failed=0 seconds=750 at=1789479137 launch=1789478355 (at>launch: ok) stems=[init, shellcheck] changed=[init, shellcheck] (MATCH) lib=true head=9703282 log=./.keel/metrics/touched-1789478385.log runner=cargo-nextest 0.9.144 skipped=[] (no skipped binaries) test result line: 'lib::reports::tests::report_produces_cards_and_rejects_unknown (655648 ms)'
    part modularMembersArchitectureGateR2 : TestResult { :>> id = "c836fa07-4083-46cb-8d95-8e5baff7c2bf"; :>> outcome = VerdictKind::pass; :>> judgedAgainst = "9703282"; :>> judgedAt = "2026-09-15"; :>> judgedBy = "claudeFable5"; }
}
