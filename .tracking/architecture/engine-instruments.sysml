// THE MEASURES, as model items (D0363, Route A of issue395). Generated from what was previously
// .engine/contracts/instruments.toml by scratchpad/gen_instruments.py, then committed as the
// authored source - the contract file is GONE rather than kept as a second home for the same facts.
//
// WHY THEY ARE ITEMS AND NOT CONFIGURATION. The owner's correction, 2026-09-07: "metadata saying
// assessed=no is the wrong route. we verify through verification_type=analysis with a matching
// verification case. we don't embed verification in metadata." A verification case needs an
// endpoint, and a contract file holds none - so an instrument could be inventoried, declared and
// computed into the control structure, and still never be verified the way every other element is.
//
// Each measure is TWO items, which is the handbook's own shape (Leveson & Thomas 2018, pp.25-26):
// a Feedback path carrying the two ends of its loop, and a Sensor carrying the mechanism that
// produces it - with its DETERMINISM, because "one run of this is a measurement" and "one run of
// this is a sample" are different facts about a number and nothing else on the page says which.
package EngineInstruments {
    private import EngineElement::*;
    private import EngineSafety::*;

    // recallBench: .engine/tools/recall_bench.py
    // KNOWN FAILURE MODES: issue296 (a query built from the target's own body scores ~100% and discriminates nothing); issue391 (an element that answers the question is scored a miss because ground truth names one record); issue406 (a number read against a binary cargo was still relinking - the run now names its build line before the first case and re-reads it after the last, recall_binary.py)
    // PUBLISHED TO: a DoD criterion, and the standing brief when retrieval is the subject
    part fbRecallBench : Feedback {
        :>> id = "d7cccf55-0e25-5649-9a33-798ef195d4b7";
        :>> title = "whether ranked recall beats a keyword search at putting the answering record in front of the model";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snRecallBench : Sensor {
        :>> id = "3c4f5cf6-dada-5dd4-8682-7b51be864530";
        :>> title = "recallBench";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/recall_bench.py";
        :>> measures = "whether ranked recall beats a keyword search at putting the answering record in front of the model";
        :>> determinism = Determinism::seeded;
        :>> reading = SensorReading::number;
        :>> producesFeedback = fbRecallBench;
    }

    // recallAb: .engine/tools/recall_ab.py
    // KNOWN FAILURE MODES: issue391 (the hit definition); the set is small enough that one case moves the number by 12 points; issue406 (6/8 read against a relinking image - the run names its binary at header and footer, recall_binary.py)
    // PUBLISHED TO: a DoD criterion
    part fbRecallAb : Feedback {
        :>> id = "5b94e5c1-c8a4-5380-88a2-de70c50c545c";
        :>> title = "whether injection puts the answering element in front of the model on eight hand-written questions";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snRecallAb : Sensor {
        :>> id = "1553b52e-d2c9-5ba0-98bc-6523518fe66b";
        :>> title = "recallAb";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/recall_ab.py";
        :>> measures = "whether injection puts the answering element in front of the model on eight hand-written questions";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::number;
        :>> producesFeedback = fbRecallAb;
    }

    // routingProbe: .engine/tools/routing_probe.py
    // KNOWN FAILURE MODES: issue392 (two of four verdicts flipped between identical runs, so one sample per skill is not a measurement); issue415 (it scores the first Skill call, and intake is declared to run first on every human direction)
    // PUBLISHED TO: nothing standing (D0378 C) - an investigation rig (D0382) whose per-case findings and measured spend are recorded as evidence on the item that motivated the run; .keel/metrics/routing-investigation.json is the last run, machine-local
    part fbRoutingProbe : Feedback {
        :>> id = "698145f8-cb19-57f8-998d-def7a57379f0";
        :>> title = "whether a skill's prose actually causes a real model to invoke the process";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpEnforcementSurface;
        :>> reportsTo = ctAgent;
    }
    part snRoutingProbe : Sensor {
        :>> id = "09f6113d-cfe3-5f2b-b220-cdda78e09dde";
        :>> title = "routingProbe";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/routing_probe.py";
        :>> measures = "whether a skill's prose actually causes a real model to invoke the process";
        :>> determinism = Determinism::stochastic;
        :>> reading = SensorReading::number;
        :>> producesFeedback = fbRoutingProbe;
    }

    // guardProofCensus: .engine/tools/guard_proof_census.py
    // KNOWN FAILURE MODES: its own first version returned a false 64/64 by matching its own registry; textual, so a guard exercised by a fixture that never names it counts as unnamed
    // PUBLISHED TO: keel show controls -> controlProof (the binary computes it, sprint 674); the script stays as the independent cross-check the integration test compares against
    part fbGuardProofCensus : Feedback {
        :>> id = "eeee2910-63ba-5c4f-b3df-00d7d982438c";
        :>> title = "how many enforced guards have a test that constructs their defect and asserts they report it";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpEnforcementSurface;
        :>> reportsTo = ctAgent;
    }
    part snGuardProofCensus : Sensor {
        :>> id = "24756f5c-e2ec-5a80-bdae-02b6ae2cf7aa";
        :>> title = "guardProofCensus";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/guard_proof_census.py";
        :>> measures = "how many enforced guards have a test that constructs their defect and asserts they report it";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::number;
        :>> producesFeedback = fbGuardProofCensus;
    }

    // conformanceLane: .engine/tools/validate/conformance_lane.py
    // KNOWN FAILURE MODES: a rejection may be the pilot kernel's gap rather than ours, which is why it is an indicator and never a gate (D0132/issue081)
    // PUBLISHED TO: conformanceIndicator (keel show indicators)
    part fbConformanceLane : Feedback {
        :>> id = "91923ebb-d8b0-5b0d-84a9-030ef4ff5575";
        :>> title = "which authored constructs the SysML v2 pilot kernel rejects";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snConformanceLane : Sensor {
        :>> id = "ad814a64-e535-5c79-b2dc-ed2543da7c5b";
        :>> title = "conformanceLane";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/conformance_lane.py";
        :>> measures = "which authored constructs the SysML v2 pilot kernel rejects";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::number;
        :>> producesFeedback = fbConformanceLane;
    }

    // execBriefFacts: scripts/exec_brief/facts.py
    // KNOWN FAILURE MODES: issue389 (it typed the release tag it measured against, so every since-the-release number would have described a superseded release behind a provenance string naming a real command)
    // PUBLISHED TO: the standing decision brief — straight to the human, with no gate between
    part fbExecBriefFacts : Feedback {
        :>> id = "8a18e9e1-163f-59a1-b2d5-da487d482d0c";
        :>> title = "every number the standing decision brief asserts, each with the command or parse rule that produced it";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpWork;
        :>> reportsTo = ctHuman;
    }
    part snExecBriefFacts : Sensor {
        :>> id = "ba7f6f65-e590-52f9-938b-e28a0ed2f34c";
        :>> title = "execBriefFacts";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = "scripts/exec_brief/facts.py";
        :>> measures = "every number the standing decision brief asserts, each with the command or parse rule that produced it";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::number;
        :>> producesFeedback = fbExecBriefFacts;
    }

    // execBriefCharts: scripts/exec_brief/charts.py
    // KNOWN FAILURE MODES: issue390 (the legend spaced entries evenly so a long label overprinted its neighbour: correct data, unreadable picture)
    // PUBLISHED TO: the standing decision brief
    part fbExecBriefCharts : Feedback {
        :>> id = "7729e4ac-019f-5b32-9d1f-172ec6e19584";
        :>> title = "nothing itself — it RENDERS measured values as figures, and a figure is what the human actually reads";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpWork;
        :>> reportsTo = ctHuman;
    }
    part snExecBriefCharts : Sensor {
        :>> id = "afab2730-8ae6-57da-ae74-37969d528348";
        :>> title = "execBriefCharts";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = "scripts/exec_brief/charts.py";
        :>> measures = "nothing itself — it RENDERS measured values as figures, and a figure is what the human actually reads";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::figure;
        :>> producesFeedback = fbExecBriefCharts;
    }

    // execBriefLogicExhibits: scripts/exec_brief/logic_exhibits.py
    // KNOWN FAILURE MODES: issue390 (a note longer than the canvas was CUT mid-word: SVG does not wrap, so the reader saw a sentence that looked finished)
    // PUBLISHED TO: the standing decision brief
    part fbExecBriefLogicExhibits : Feedback {
        :>> id = "24fcd43a-0717-5cc7-9609-69aa14430750";
        :>> title = "nothing itself — it renders the today-versus-changed and downstream exhibits the brief argues from";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpWork;
        :>> reportsTo = ctHuman;
    }
    part snExecBriefLogicExhibits : Sensor {
        :>> id = "5f747b11-f542-5eb4-82c3-8b007428af96";
        :>> title = "execBriefLogicExhibits";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = "scripts/exec_brief/logic_exhibits.py";
        :>> measures = "nothing itself — it renders the today-versus-changed and downstream exhibits the brief argues from";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::figure;
        :>> producesFeedback = fbExecBriefLogicExhibits;
    }

    // templateContract: scripts/check_templates.py
    // KNOWN FAILURE MODES: it checks structure, not truth — a brief can pass while every number on it is wrong, which is exactly what issue389 would have produced
    // PUBLISHED TO: a DoD criterion; it gates no commit
    part fbTemplateContract : Feedback {
        :>> id = "99a5963c-90c3-5ef1-abd5-d041334971cc";
        :>> title = "whether a document meets the brand and executive-brief contracts";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpDeliverable;
        :>> reportsTo = ctAgent;
    }
    part snTemplateContract : Sensor {
        :>> id = "b610a2c8-1ed8-5fd3-bc57-d90a53f291be";
        :>> title = "templateContract";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = "scripts/check_templates.py";
        :>> measures = "whether a document meets the brand and executive-brief contracts";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbTemplateContract;
    }

    // fitCheck: scripts/exec_brief/fit_check.py (D0402, sprint 628)
    // KNOWN FAILURE MODES: the fallback face it measures is THIS host's default sans-serif; a wider fallback elsewhere can overrun where this host fits (the 2px tolerance and the OVERRUN rule leave a margin, not a proof). CI carries no browser, so its --probe does not run there.
    // PUBLISHED TO: the build of every brief - assert_fits is the builder's last line and removes a page that does not fit
    part fbFitCheck : Feedback {
        :>> id = "d522b85d-3b61-5505-876e-eb1b62f00923";
        :>> title = "whether every SVG text on a built brief lands inside its canvas, clear of its neighbours and inside the box it starts in, measured in a browser with and without web fonts";
        :>> createdAt = "2026-09-09"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpDeliverable;
        :>> reportsTo = ctAgent;
    }
    part snFitCheck : Sensor {
        :>> id = "005f7385-0a6e-5aea-a0a5-68a8287434ee";
        :>> title = "fitCheck";
        :>> createdAt = "2026-09-09"; :>> createdBy = "claudeOpus5";
        :>> mechanism = "scripts/exec_brief/fit_check.py";
        :>> measures = "rendered text geometry per figure: OVERFLOW past the viewBox, OVERLAP with a sibling, OVERRUN past the drawn box, horizontal SCROLL - each finding with figure, kind, pixels and text; four known cases run as --probe first";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbFitCheck;
    }

    // docRender: scripts/render.py
    // KNOWN FAILURE MODES: none recorded; UNPROVEN in the D0360 sense — no test constructs a render defect and shows this catching it
    // PUBLISHED TO: a DoD criterion
    part fbDocRender : Feedback {
        :>> id = "6dda0782-683c-5fa0-b0d6-82909d7e771f";
        :>> title = "whether a document renders and matches its golden output";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpDeliverable;
        :>> reportsTo = ctAgent;
    }
    part snDocRender : Sensor {
        :>> id = "3e1543ce-5b0a-511c-83b9-3d45124ea5d8";
        :>> title = "docRender";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = "scripts/render.py";
        :>> measures = "whether a document renders and matches its golden output";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbDocRender;
    }

    // statusDashboard: .engine/tools/report.py
    // KNOWN FAILURE MODES: none recorded; a materialised view is stale the moment the tree moves, which EHZ8 names as a hazard
    // PUBLISHED TO: a materialised view, marked as one
    part fbStatusDashboard : Feedback {
        :>> id = "59553cf7-7d14-59b7-a2bf-76c4242ea981";
        :>> title = "nothing itself — it materialises project state as an HTML dashboard";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpWork;
        :>> reportsTo = ctHuman;
    }
    part snStatusDashboard : Sensor {
        :>> id = "9b4d2502-86b9-5d70-8ade-484045b58e1b";
        :>> title = "statusDashboard";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/report.py";
        :>> measures = "nothing itself — it materialises project state as an HTML dashboard";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::figure;
        :>> producesFeedback = fbStatusDashboard;
    }

    // validateSchema: .engine/tools/validate/validate_schema.py
    // KNOWN FAILURE MODES: an external verdict can flip without a change here; the kernel's own gaps are indistinguishable from ours (D0132/issue081)
    // PUBLISHED TO: it BLOCKS a schema change (CLAUDE.md section 5)
    part fbValidateSchema : Feedback {
        :>> id = "43877a0e-96b8-5560-a3f0-e2c53cc5eef2";
        :>> title = "whether the schema packages parse and type-check against the pilot kernel";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snValidateSchema : Sensor {
        :>> id = "b9a56a57-1aff-5ccb-bd5f-2f1ab80ca782";
        :>> title = "validateSchema";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/validate_schema.py";
        :>> measures = "whether the schema packages parse and type-check against the pilot kernel";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbValidateSchema;
    }

    // validateWorkflows: .engine/tools/validate/validate_workflows.py
    // KNOWN FAILURE MODES: as validateSchema
    // PUBLISHED TO: it BLOCKS a workflow change
    part fbValidateWorkflows : Feedback {
        :>> id = "b3aad109-5694-5473-8e77-056dea8e0182";
        :>> title = "whether the process-as-data workflow files parse and type-check against the pilot kernel";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpEnforcementSurface;
        :>> reportsTo = ctAgent;
    }
    part snValidateWorkflows : Sensor {
        :>> id = "0b25b7b5-0c64-574a-887f-94b221b5775e";
        :>> title = "validateWorkflows";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/validate_workflows.py";
        :>> measures = "whether the process-as-data workflow files parse and type-check against the pilot kernel";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbValidateWorkflows;
    }

    // validateInstances: .engine/tools/validate/validate_instances.py
    // KNOWN FAILURE MODES: DEMOTED because it fails CORRECT files, which forced an all-or-nothing bypass that disabled every other layer (D0132/issue081) - the sharpest instance of an instrument's false verdict costing more than its true ones
    // PUBLISHED TO: opt-in only (KEEL_KERNEL_VALIDATE=1)
    part fbValidateInstances : Feedback {
        :>> id = "60c0c8d6-b47b-5d33-a888-0e73c362a240";
        :>> title = "whether each instance file types against the flat schema in the kernel";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snValidateInstances : Sensor {
        :>> id = "062f2928-a337-5f53-94bf-3d1de8262369";
        :>> title = "validateInstances";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/validate_instances.py";
        :>> measures = "whether each instance file types against the flat schema in the kernel";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbValidateInstances;
    }

    // validateTracking: .engine/tools/validate/validate_tracking.py
    // KNOWN FAILURE MODES: as validateInstances
    // PUBLISHED TO: the kernel lane
    part fbValidateTracking : Feedback {
        :>> id = "67a0ebdc-f020-533c-9efb-733f5ddce87c";
        :>> title = "whether tracking instances type against the full schema in the kernel";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snValidateTracking : Sensor {
        :>> id = "796dc10d-8808-580e-af38-4e4b60610f23";
        :>> title = "validateTracking";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/validate_tracking.py";
        :>> measures = "whether tracking instances type against the full schema in the kernel";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbValidateTracking;
    }

    // validateProbes: .engine/tools/validate/validate_probes.py
    // KNOWN FAILURE MODES: a cell FAILS iff the kernel emits a line containing ERROR, which is a text match on another program's output
    // PUBLISHED TO: schema design decisions
    part fbValidateProbes : Feedback {
        :>> id = "4e4ac40e-ecb4-5d95-a400-f3c239c5dd22";
        :>> title = "which exact SysML v2 patterns the kernel accepts, cell by cell";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snValidateProbes : Sensor {
        :>> id = "9765da67-d49d-5e5c-b8db-7a089dfc6212";
        :>> title = "validateProbes";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/validate_probes.py";
        :>> measures = "which exact SysML v2 patterns the kernel accepts, cell by cell";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbValidateProbes;
    }

    // importProbe: .engine/tools/validate/import_probe.py
    // KNOWN FAILURE MODES: as validateProbes
    // PUBLISHED TO: schema design decisions
    part fbImportProbe : Feedback {
        :>> id = "838031ef-bede-5888-b58d-43faac05b4f9";
        :>> title = "whether the kernel resolves imports the way the schema assumes";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snImportProbe : Sensor {
        :>> id = "92fd71cb-7729-5ce7-9241-257fa511afbe";
        :>> title = "importProbe";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/import_probe.py";
        :>> measures = "whether the kernel resolves imports the way the schema assumes";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbImportProbe;
    }

    // structureProbe: .engine/tools/validate/structure_probe.py
    // KNOWN FAILURE MODES: as validateProbes
    // PUBLISHED TO: schema design decisions
    part fbStructureProbe : Feedback {
        :>> id = "2e9c6183-a61c-5fd8-b15c-4cb48a554fac";
        :>> title = "whether the kernel accepts the structural constructs the schema relies on";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snStructureProbe : Sensor {
        :>> id = "980e651c-028e-5ef5-a456-660f1d5d48e5";
        :>> title = "structureProbe";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/structure_probe.py";
        :>> measures = "whether the kernel accepts the structural constructs the schema relies on";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbStructureProbe;
    }

    // spikeMetamodel: .engine/tools/validate/spike_metamodel.py
    // KNOWN FAILURE MODES: a one-off measurement whose answer is still cited; nothing re-runs it, so it is a claim about a kernel version nobody records
    // PUBLISHED TO: a foundation decision, when it was run
    part fbSpikeMetamodel : Feedback {
        :>> id = "68addb39-810d-5ef5-9c5c-900c575228a7";
        :>> title = "whether SysML v2 can express the process-as-data meta-model at all";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> sensedFrom = cpModel;
        :>> reportsTo = ctAgent;
    }
    part snSpikeMetamodel : Sensor {
        :>> id = "cb989093-9ce0-50dd-8d7c-d75f9027ff09";
        :>> title = "spikeMetamodel";
        :>> createdAt = "2026-09-07"; :>> createdBy = "claudeOpus5";
        :>> mechanism = ".engine/tools/validate/spike_metamodel.py";
        :>> measures = "whether SysML v2 can express the process-as-data meta-model at all";
        :>> determinism = Determinism::external;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbSpikeMetamodel;
    }

    // modGraph: scripts/modgraph.py (D0479, sprint 717)
    // KNOWN FAILURE MODES: it reads `crate::<module>` tokens with `//` comments and string literals stripped, so a reference reached through a `use` alias or a re-export is attributed to the alias's home, not the module named; only `//` comments (doc comments included) and string literals are stripped, so a `/* */` block comment that spells `crate::guards::` reads as a reference (a false positive, never a miss; none in the tree at the adding commit). The forbidden pairs are a list in the script, so a new layer is guarded only once its pair is added there. A build script's owner is the workspace member whose directory contains it (build/provenance.rs is owned by none), and `build = true` or an absent key resolves to the crate's own build.rs; the dependency closure follows PATH dependencies only, so a member reached through a registry version is not in it (a false upward, never a miss).
    // PUBLISHED TO: the DoD of dcGuardsViewOrientCycleIsBroken (its D0388 pair); the CI script-probe step and the pre-commit hook on staged scripts (`# ci-probe: --probe`, sprint 757 - the probe's last rung is `--check` on the landed tree); under dcWorkspaceLayeringIsGuarded, the guard that will carry it
    part fbModGraph : Feedback {
        :>> id = "612aa446-c514-4f63-a818-5735a9f591ee";
        :>> title = "whether the workspace holds the D0479 layering: no reference from view, orient, write or scaffold into guards, none from view into orient, and no crate built from a script a crate above it owns (build = read as a dependency edge, sprint 757)";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> sensedFrom = cpDeliverable;
        :>> reportsTo = ctAgent;
    }
    part snModGraph : Sensor {
        :>> id = "4abaf613-7b3e-4e35-a936-32fc05658821";
        :>> title = "modGraph";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> mechanism = "scripts/modgraph.py";
        :>> measures = "the top-level module graph of keel-cli/src (nodes: modules and keel_* re-exports; edges: crate::X references): module and edge counts, Tarjan cycles, and under --check the FORBIDDEN back-edges each named at file:line, exit 1 on any; the build-script edges of every workspace member (--build-edges: crate, script, the script's owner crate, the manifest line), and under --check an UPWARD row - a script owned by a crate that is neither the crate itself nor in its transitive path-dependency closure - is FORBIDDEN, exit 1; --probe runs the D0388 pair on two temp fixtures (an upward build.rs; a workspace-level build/provenance.rs) and then --check on the landed tree";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbModGraph;
    }

    // moduleHome: scripts/module_home.py (D0479, sprint 719, issue559)
    // KNOWN FAILURE MODES: it resolves a module by NAME against every workspace member's src, so two crates with a module of one name is refused (AmbiguousModule) rather than guessed, and a caller that wants one of them must say which crate; the anchor scan reads path EXPRESSIONS of two shapes (an os.path.join ending in keel-cli, src, x.rs and a pathlib / operator with a keel-cli/src/x.rs literal), so an anchor built any other way (a bare string constant later joined, a format string) passes the scan - a miss, never a false positive; the member list is read from the ROOT manifest only, so a nested workspace is not seen.
    // PUBLISHED TO: the CI script-probe step (D0394) and, under dcScriptProbesRunInTheVerifyLadder, the verify rung that will carry it; the DoD of dcScriptsResolveModuleHomesFromTheManifest (its D0388 pair)
    part fbModuleHome : Feedback {
        :>> id = "b942f668-f53d-4321-97a4-6790373bf9c3";
        :>> title = "whether every script that opens a Rust source file resolves its home from Cargo.toml [workspace] members, and none anchors keel-cli/src by hand - the Python side of issue557";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> sensedFrom = cpDeliverable;
        :>> reportsTo = ctAgent;
    }
    part snModuleHome : Sensor {
        :>> id = "a51b69d9-ef54-402e-a019-64440690fbdc";
        :>> title = "moduleHome";
        :>> createdAt = "2026-09-14"; :>> createdBy = "claudeFable5";
        :>> mechanism = "scripts/module_home.py";
        :>> measures = "where a named Rust module lives among the workspace members (a path, None, or a refusal naming both homes), and under --probe: a fixture workspace of two crates resolved as the D0388 pair, the two anchor shapes caught and the resolver call passed, and the real tree scanned for any .rs anchor under keel-cli/src in scripts/**/*.py or .engine/tools/*.py - exit 1 on any";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbModuleHome;
    }

    // verbHomes: scripts/verb_homes.py (D0479, D0512, sprint 740)
    // KNOWN FAILURE MODES: it reads the crates a body names by `keel_<x>::` and `crate::<module>` tokens with `//` comments and string literals stripped, so a name reached through a `use` alias or a glob import is attributed to the alias's home; the call graph among main.rs items is by bare identifier followed by `(`, so a fn called through a variable or a method of one name is missed (a miss, never a false positive); the least-member rule lets a member ADD a crate that reaches neither the member nor anything the member does not, read from the manifests' [dependencies] only, so a dev- or build-dependency is not a reach; the binary is the answer only when no member could hold the body.
    // PUBLISHED TO: the DoD of dcSharedHelpersDescendBelowTheVerbs (its D0388 pair, pos740.py) and dcKeelCliIsThinDispatch (--check exit 0 is its gate; --by-member names each body's home); the CI script-probe step (D0394) runs --probe.
    part fbVerbHomes : Feedback {
        :>> id = "3f682729-ed04-41a3-bc6d-365a113b1d22";
        :>> title = "for every top-level item of keel-cli/src/main.rs, which members its body reaches and the least member that could hold it - whether any body has no home but the binary";
        :>> createdAt = "2026-09-17"; :>> createdBy = "claudeFable5";
        :>> sensedFrom = cpDeliverable;
        :>> reportsTo = ctAgent;
    }
    part snVerbHomes : Sensor {
        :>> id = "d807cfcd-45f5-4695-bca1-b479f89d8bcf";
        :>> title = "verbHomes";
        :>> createdAt = "2026-09-17"; :>> createdBy = "claudeFable5";
        :>> mechanism = "scripts/verb_homes.py";
        :>> measures = "per top-level item of main.rs: the crates its body names directly and through the main.rs items it calls, the members reaching them all, and the least members (fewest additions, then undominated) labelled member(+added) - a table, --by-member totals, and under --check exit 1 naming any item whose least member is keel-cli or which no member reaches (main, cmd_show and mod tests excluded); --probe runs a fixture workspace of four members as the D0388 pair first";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbVerbHomes;
    }

    // verifyReceipt: scripts/verify_receipt.py (D0533, sprint 751, issue490/issue567)
    // KNOWN FAILURE MODES: it reads the four receipt files as the ladder left them, so a slot rewritten by a later writer (keel land after the ladder, issue660) is reported only when its at is later than the ladder's - a rewrite within the same second passes as the ladder's own; writer liveness is a pid probe plus a process-name scan (tasklist/pgrep), so a ladder whose pid was reused by another process reads as WAIT; the changed-stem rule mirrors members/keel-suite/src/touched.rs by hand and drifts if that file changes without this one; the test-summary line is the first `keel suite --touched:` line of verify-touched.out, so a ladder run twice into one .out file quotes the earlier run.
    // PUBLISHED TO: the verifier's receipt file (the D0425 ceremony, dispatched by the delegated-ceremony brief and the test-verify skill); the CI script-probe step (D0394) and the pre-commit script_probes.py run --probe; the DoD of dcVerifierReceiptOutcomeIsRenderedNotTyped (its D0388 pair).
    part fbVerifyReceipt : Feedback {
        :>> id = "1acdda2a-248a-44bd-8317-a8bc89f9264a";
        :>> title = "what the pre-commit ladder and its touched run actually recorded - every verdict below the receipt header as a field read, and the computed DISCREPANCIES between those receipts, HEAD, the changed set and the outside checks - in place of the verifier's typed summary (issue490)";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> sensedFrom = cpWork;
        :>> reportsTo = ctAgent;
    }
    part snVerifyReceipt : Sensor {
        :>> id = "4f0628ff-a0f6-405c-aa8b-f5d433401f8f";
        :>> title = "verifyReceipt";
        :>> createdAt = "2026-09-19"; :>> createdBy = "claudeFable5";
        :>> mechanism = "scripts/verify_receipt.py";
        :>> measures = "from verify-receipt.toml, touched-receipt.toml, verify-launch.epoch and verify-touched.out: the ladder's outcome, stopped-at rung and per-rung verdicts with exit and command; the touched run's outcome, passed, failed, failing names, runner, long pole and stems; check-engine and sync-claude --check run in-process; and DISCREPANCIES as the computed list - ladder or touched not pass, a failing rung or test, a rung missing or not run, a receipt older than the launch epoch or newer than the ladder, a head other than HEAD, stems other than the changed set - rendered as the verifier's receipt (exit 0), WAIT while the ladder's writer lives (exit 2), KILLED or NO RECEIPT when none does (exit 3); under --probe, twelve fixture checks incl. the issue490 red-as-NONE case, exit 1 on any";
        :>> determinism = Determinism::tree;
        :>> reading = SensorReading::verdict;
        :>> producesFeedback = fbVerifyReceipt;
    }
}
