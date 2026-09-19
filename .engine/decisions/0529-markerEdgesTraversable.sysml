// D0529 (PROPOSED — NOT YET ACCEPTED) — markerEdgesTraversable: a project-declared metadata def is a traversable edge kind in TOML views by its own name - or the dependency kind covers every marker form with a target filter - or nothing changes
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0529Accept event on sign-off.
package Decision0529 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: members/keel-model/src/schema.rs:43,251-262 (edge kinds read from the embedded SCHEMA_DIR only), the marker-vocabulary guard's dynamic reading of project metadata defs, D0133's type-check of project markers, D0150's derived vocabulary, and the reporter's view error listing the known kinds
    part d0529 : Decision {
        :>> id = "7ff106e2-8c57-4c14-b7d2-a340ad31f7d7";
        :>> title = "markerEdgesTraversable: a project-declared metadata def is a traversable edge kind in TOML views by its own name - or the dependency kind covers every marker form with a target filter - or nothing changes";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "GH#67 (st101, re-ingested as st142): a project declared eleven marker edges in its relationships.sysml under D0139(B), used them 200+ times in an STPA model, and a TOML view that traverses one fails with unknown edge kind - while `edges = ['dependency']` does not include marker-form dependencies either. The vocabulary is validated and guarded and invisible to every declared view.";
        :>> decision = "RESEARCH: read before asking - schema.rs:43,251-262, the marker-vocabulary guard, D0133, D0150, the view error text in st101. OPTION A (recommended): the view engine's edge-kind vocabulary is derived from every metadata def the model declares, embedded and project alike, lower-cased by the same rule the embedded kinds use, so `traverse causes` works the day the project declares Causes; an undeclared kind is still an error naming the known set. COST: a project marker with a name colliding with an embedded kind must be refused at declaration, and the schema reader that today reads one directory reads the model. OPTION B: `dependency` covers every `#Marker dependency from A to B;` regardless of marker and views filter with [traverse.target] on the marker name. COST: every existing view over dependency widens silently to marker edges it did not mean; the filter is the only guard against that and no existing view carries one. OPTION C: nothing changes. COST: a project's own edge vocabulary is authored, validated, guarded and invisible to every view - the safety viewpoint falls back to the verify slice.";
        :>> rationale = "D0150 already says the schema vocabulary is derived, not baked, and D0133 already type-checks a project's metadata defs, so the engine knows every marker a project declares; reading the edge kinds from one embedded directory is the one place that knowledge is not used. A finishes the D0150 rule for edges. B widens a kind every existing view depends on and would change their results without any of them changing text, which is the silent-drift class the view discipline exists to prevent. C leaves the most-used authoring path of a downstream STPA model unqueryable.";
        :>> consequences = "If A: schema.rs derives edge kinds from the model, a collision refusal, and a test where a fake project marker traverses and an undeclared kind errors with the project kind listed among the known. If B: a widening of dependency and a target filter, with every shipped view audited. If C: the story is dispositioned with the human's words.";
    }
    #DerivedFrom dependency from d0529 to st101;
    #DerivedFrom dependency from d0529 to st142;
}
