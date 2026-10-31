// Complete Oracle and pinned XMage implementation.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
const CONTROLLED_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const TOKEN_DOUBLING: ReplacementSpec = ReplacementSpec::DoubleTokens {
    effects_only: false,
};
const STATIC: StaticAbilityGroup = StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, TOKEN_DOUBLING)],
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[STATIC],
    extras: &CardExtras {
        starting_loyalty: 5,
        static_ability_groups: &[STATIC],
        replacement_effects: &[TOKEN_DOUBLING],
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5318),
                amount: 1,
                tapped: false,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(0),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCreatureCountersAndRemember {
                    filter: CONTROLLED_CREATURES,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GrantRememberedCreaturesUntilNextTurn(Keywords::FLYING),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-3),
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 3,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Destroy],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
