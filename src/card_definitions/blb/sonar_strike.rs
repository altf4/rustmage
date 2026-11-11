const TAPPED_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: true,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CONTROLS_BAT: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Bat"),
    basic: false,
    other: false,
    cards_only: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::AttackingOrBlocking,
            },
            TAPPED_CREATURE,
        ]))),
        &[
            Instruction::Damage(4),
            Instruction::If {
                guard: StepGuard::Expression {
                    value: CONTROLS_BAT,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};
