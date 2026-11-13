const ALLY_COLORS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Colors(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Ally"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, ALLY_COLORS)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        exhaust: false,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            colored: [1, 1, 1, 1, 1, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::AnimateEarthbendTargets,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 5,
            },
            Instruction::RegisterEarthbendReturn,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        scaling_modifiers: &[ALLY_COLORS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
