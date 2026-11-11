const MILL: Instruction = Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4));
const RECOVER: Instruction = Instruction::SelectMilled(MilledSelectionSpec {
    filter: MilledFilter::Kinds(1 | 2),
    count: MilledCount::Range {
        minimum: 0,
        maximum: 2,
    },
    destination: MilledDestination::Hand,
});
const DEPLOY: Instruction = Instruction::PutHandCreatures { maximum: 2 };
const PROTECT: Instruction = Instruction::GrantCreaturesWithPowerAtLeast {
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    minimum: 4,
    keywords: Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
};
const fn mode(text: &'static str, program: &'static [Instruction]) -> SpellMode {
    SpellMode {
        text,
        targets: TargetPlan::NONE,
        program,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        mode(
            "Mill four and recover up to two creatures or lands",
            &[MILL, RECOVER],
        ),
        mode(
            "Put up to two creatures from your hand onto the battlefield",
            &[DEPLOY],
        ),
        mode("Protect your creatures with power 4 or greater", &[PROTECT]),
        mode(
            "Mill, recover, and deploy creatures",
            &[MILL, RECOVER, DEPLOY],
        ),
        mode(
            "Mill, recover, and protect creatures",
            &[MILL, RECOVER, PROTECT],
        ),
        mode("Deploy and protect creatures", &[DEPLOY, PROTECT]),
        mode(
            "Mill, recover, deploy, and protect creatures",
            &[MILL, RECOVER, DEPLOY, PROTECT],
        ),
    ]),
    mode_costs: Some(&[
        generic(2),
        ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
        },
        generic(1),
        ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 6,
        },
        generic(3),
        ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 5,
        },
        ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 7,
        },
    ]),
    ..CardRules::EMPTY
};
