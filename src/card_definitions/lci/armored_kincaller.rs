pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayRevealHandSubtype {
                subtype: "Dinosaur",
                skip: 2,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
            Instruction::Skip(2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Dinosaur"),
                    basic: false,
                    other: true,
                    cards_only: false,
                })),
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    }],
    ..CardRules::EMPTY
};
