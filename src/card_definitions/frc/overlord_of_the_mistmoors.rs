pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        ImpendingSpec::END_STEP,
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5673), // White flying Insect Token
                amount: 2,
                tapped: false,
            }],
        },
    ],
    extras: &CardExtras {
        impending: Some(ImpendingSpec {
            cost: ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 2,
            },
            time_counters: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
