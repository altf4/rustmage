pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Grouped {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                constraint: TargetGroupConstraint::TotalManaValueAtMost(3),
            },
            min: 0,
            max: 3,
        }),
        &[Instruction::ReturnToBattlefieldThenCounters(&[(
            CounterKind::PlusOne,
            1,
        )])],
    ),
    ..CardRules::EMPTY
};
