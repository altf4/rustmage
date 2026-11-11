const REMOVE: Instruction = Instruction::RemovePlayerCreatureAbilities {
    players: PlayerSet::Each,
    duration: AnimationDuration::EndOfTurn,
};
const CHOOSE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::RememberPermanents {
        require_battlefield_source: false,
    },
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        kinds: 1,
        keywords: Keywords::NONE,
        source_only: false,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 1,
    optional_skip: None,
};
const PROTECT: Instruction = Instruction::GrantRememberedCreatures(Keywords::INDESTRUCTIBLE);
const DESTROY: Instruction = Instruction::MoveBattlefield {
    filters: &[BattlefieldFilter {
        kinds: 1,
        relation: -1,
        nonland: false,
        tapped: None,
        without: Keywords::NONE,
        subtype: None,
        max_mana: u32::MAX,
    }],
    exile: false,
};
const fn destroy_cost(generic: u32) -> ManaCost {
    ManaCost {
        colored: [2, 0, 0, 0, 0, 0],
        generic,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Remove creature abilities",
            targets: TargetPlan::NONE,
            program: &[REMOVE],
        },
        SpellMode {
            text: "Grant indestructible",
            targets: TargetPlan::NONE,
            program: &[CHOOSE, PROTECT],
        },
        SpellMode {
            text: "Destroy all creatures",
            targets: TargetPlan::NONE,
            program: &[DESTROY],
        },
        SpellMode {
            text: "Remove abilities + Grant indestructible",
            targets: TargetPlan::NONE,
            program: &[REMOVE, CHOOSE, PROTECT],
        },
        SpellMode {
            text: "Remove abilities + Destroy creatures",
            targets: TargetPlan::NONE,
            program: &[REMOVE, DESTROY],
        },
        SpellMode {
            text: "Grant indestructible + Destroy creatures",
            targets: TargetPlan::NONE,
            program: &[CHOOSE, PROTECT, DESTROY],
        },
        SpellMode {
            text: "Remove abilities + Grant indestructible + Destroy creatures",
            targets: TargetPlan::NONE,
            program: &[REMOVE, CHOOSE, PROTECT, DESTROY],
        },
    ]),
    mode_costs: Some(&[
        generic(1),
        generic(1),
        destroy_cost(3),
        generic(2),
        destroy_cost(4),
        destroy_cost(4),
        destroy_cost(5),
    ]),
    mode_counts: Some(&[1, 1, 1, 2, 2, 2, 3]),
    ..CardRules::EMPTY
};
