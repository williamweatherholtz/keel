// D0472 (PROPOSED — NOT YET ACCEPTED) — The two doc guards read one living surface
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0472Accept event on sign-off.
package Decision0472 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // #ProspectiveChange (D0070) - this Decision governs a process/schema change; the co-committed
    // change rides the process-change guard's hard lock.
    #ProspectiveChange part d0472 : Decision {
        :>> id = "ea4dac00-23c0-4e65-937d-41529bcd7091";
        :>> title = "The two doc guards read one living surface";
        :>> createdAt = "2026-09-13";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "D0471's consequences and sprint 703's Story DoD both say cli_reference sits beside tool_reference sharing living_doc_files, so the two guards mean the same surface. At aba396e that was false of the source: tool_reference (keel-cli/src/guards.rs:4336) carried its own copy of the walk - the same six .engine directories, the same md/sysml/toml filter, the same CLAUDE.md push - and only cli_reference called living_doc_files. The two copies agreed today, which is why every run scanned the same files and no test could tell them apart; they would have parted the first time one gained a directory the other did not. The twenty-fourth brief builder refused the page for it: the page says both doc guards walk one surface; the source does not share living_doc_files. Nothing in the tree had checked that sentence before a page repeated it.";
        :>> decision = "The living doc surface a guard reads exists once in source: living_doc_files, and both tool-reference and cli-reference take their file list from it. A guard that means the same surface as another does not carry a second walk of it.";
        :>> rationale = "A guard's scope is part of what the guard means, and two copies of a scope are two meanings that happen to coincide. The D0471 rationale rests on cli-reference covering exactly the procedure surface tool-reference already covers - the surface a delegated agent runs from - and that rests on the two lists being one list, not two lists that match at one commit. The record said so; the source did not; a consequence that is true of the record and false of the tree is the dual-truth class. The fix is twenty lines removed, not added, and the guard's own test tool_reference_flags_only_missing_files_on_the_living_surface passes unchanged. MEASURED: keel gate guard tool-reference --no-receipt . before and after the edit on this Windows 11 host: 3 scanned, 0 warning(s), 0 violation(s) both ways; cli-reference 438 scanned both ways. NOT A FORK: keeping two walks and a test that they agree is the alternative, and it keeps the meaning in two places to hold a fact that one place holds by construction.";
        :>> consequences = "keel-cli/src/guards.rs tool_reference begins let files = living_doc_files(root); its inline walk, directory loop and CLAUDE.md push are gone. The sentence in D0471 and in sprint 703's Story DoD is true from the commit that lands this. The finding itself - a DELIVERED claim about source structure recorded as a proposed pass and caught only by a brief builder - is issue533, with a resolver that has the VERIFIER read such claims back from the source before the RECORDER writes the result.";
    }

    // D0303: this Decision layers on D0471, whose consequences it makes true of the source.
    #DependsOn dependency from d0472 to d0471;
}
