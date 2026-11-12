pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: one(TargetRule::Any),
            program: &[Instruction::Damage(4)],
        },
        ImpendingSpec::END_STEP,
    ],
    extras: &CardExtras {
        impending: Some(ImpendingSpec {
            cost: ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 2,
            },
            time_counters: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
