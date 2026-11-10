pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    extras: &CardExtras {
        warp: Some(&WarpSpec::fixed(ManaCost {
            generic: 1,
            colored: [1, 0, 0, 0, 0, 0],
        })),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::WithoutSubtype("Assassin"),
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ExileUntilSourceLeaves],
    }],
    ..CardRules::EMPTY
};
