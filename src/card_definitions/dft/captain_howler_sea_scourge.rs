// Each discard batch carries its own count; the delayed draw retains the original controller.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayManaLife {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                life: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Discarded {
                relation: CastRelation::You,
                one_or_more: true,
            },
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::GrowScaled {
                    subject: Subject::Targets,
                    amount: AmountValue::DiscardEventAmount,
                    power: 2,
                    toughness: 0,
                },
                Instruction::DelayTargetEvent {
                    event: DelayedTargetEvent::CombatDamageToPlayer,
                    ability: 2,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
