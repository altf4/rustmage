const ARTIFICERS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Artificer"),
    basic: false,
    other: false,
    cards_only: false,
};
const CONTROLLED_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupScaling {
            filters: &[
                CreatureFilter {
                    artifact: true,
                    ..CONTROLLED_CREATURES
                },
                CreatureFilter {
                    subtype: Some("Hero"),
                    ..CONTROLLED_CREATURES
                },
            ],
            value: StatValue::Sum(&[
                StatValue::Count(ARTIFICERS),
                StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    cards_only: true,
                    ..ARTIFICERS
                }),
            ]),
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: true,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [1, 1, 0, 0, 0, 0],
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
