// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    triggers: &[
        ImpendingSpec::END_STEP,
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5677),
                amount: 1,
                tapped: true,
            }],
        },
    ],
    extras: &CardExtras {
        impending: Some(ImpendingSpec {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 1,
            },
            time_counters: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
