pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        ImpendingSpec::END_STEP,
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::Discard(PlayerSet::Controller, 1),
            ],
        },
    ],
    extras: &CardExtras {
        impending: Some(ImpendingSpec {
            cost: ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 1,
            },
            time_counters: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
