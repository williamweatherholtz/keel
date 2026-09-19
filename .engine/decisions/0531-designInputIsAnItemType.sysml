// D0531 (PROPOSED — NOT YET ACCEPTED) — designInputIsAnItemType: a value the design consumes but does not decide is a DesignInput item in a non-core schema package, with value, unit, source and asOf, consumed by constraints and gated - or it is an Indicator with a Measurement and a documented reading - or nothing changes
// Recorded via `keel record decision` (D0105 RMWX axis; issue054). Acceptance is a separate explicit
// human gate (method=confirmation, D0106): flip status + add the d0531Accept event on sign-off.
package Decision0531 {
    private import EngineElement::*;
    private import EngineWork::*;
    private import EngineVerification::*;
    private import EngineRelationships::*;

    // RESEARCH: the item type list walked in st055 (Decision, Requirement, Indicator+Measurement, Statement, Claim), invariant 7's Indicator rule (monitored, never gated), Measurement's value/measuredAt/source fields, the propriety and safety packages as the non-core precedent, and the irrigation project's stated need for frost depth, static pressure and available supply flow as calculation inputs
    part d0531 : Decision {
        :>> id = "8d843e96-a884-456a-b273-9a5e96965185";
        :>> title = "designInputIsAnItemType: a value the design consumes but does not decide is a DesignInput item in a non-core schema package, with value, unit, source and asOf, consumed by constraints and gated - or it is an Indicator with a Measurement and a documented reading - or nothing changes";
        :>> createdAt = "2026-09-19";
        :>> createdBy = "claudeFable5";
        :>> status = DecisionStatus::proposed;
        :>> context = "GH#31 (st055, re-ingested as st178): a landscape-irrigation project has values the design consumes and does not decide - frost depth, static pressure, soil infiltration rate, available supply flow - and every existing type is a choice, a shall, a watched-never-gated measure, or an utterance. The author's words: I'll be providing more facts to serve as basis for calculations. There is nowhere correct to put them.";
        :>> decision = "RESEARCH: read before asking - the type walk in st055, invariant 7, Measurement's fields, propriety.sysml as the non-core precedent. OPTION A (recommended): a non-core schema package declares DesignInput with value, unit, source, asOf and an optional uncertainty, referenced by constraints through a typed edge, so zone sizing may be gated against available flow without calling the flow an indicator; the frozen core is untouched. COST: a package, a guard that a DesignInput with no source is refused, a lens listing inputs and what consumes them, and the first non-core type that is a fact about the world rather than about the work. OPTION B: use Indicator plus Measurement and document that an indicator a constraint reads is gated after all. COST: invariant 7 is contradicted in a doc line, and the semantics the reporter walked stay mis-signalled. OPTION C: nothing changes. COST: the values live in a Decision nobody made or a Statement that is true, and calculations cite neither.";
        :>> rationale = "The type walk in st055 is correct on every row, and the closest fit contradicts an invariant this engine states in its own CLAUDE.md; a documented exception to an invariant is a weaker fact than a type with the right meaning. The propriety and safety packages are the precedent for adding vocabulary outside the frozen core when the canon cannot express a kind, and a design input is such a kind: it is neither decided nor required nor merely watched. A is the smallest honest shape; B is the shortcut the reporter already rejected with reasons; C leaves a real project without a home for its most-cited facts.";
        :>> consequences = "If A: a CR adding the package, a guard, a lens, and one real project's inputs as the known-positive. If B: a doc line and a worked example. If C: the story is dispositioned with the human's words.";
    }
    #DerivedFrom dependency from d0531 to st055;
    #DerivedFrom dependency from d0531 to st178;
}
