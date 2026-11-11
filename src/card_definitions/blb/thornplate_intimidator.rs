pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Source,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((1, 1)),
                    ..CopyExceptions::NONE
                },
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program: &[Instruction::OpponentSacrificeDiscardOrLoseLife {
                repetitions: AmountValue::Fixed(1),
                life: 3,
            }],
        },
    ],
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0; 6],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
