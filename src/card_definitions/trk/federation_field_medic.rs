// Pinned XMage: mode two checks the live controlled counter-bearing permanent at resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Put a +1/+1 counter on target creature. It gains vigilance until end of turn.",
                targets: one(TargetRule::Creature),
                program: &[
                    Instruction::PutCounters {
                        subject: Subject::Targets,
                        kind: CounterKind::PlusOne,
                        amount: 1,
                    },
                    Instruction::GrantKeywords {
                        subject: Subject::Targets,
                        keywords: Keywords::VIGILANCE,
                    },
                ],
            },
            SpellMode {
                text: "Draw a card if you control a permanent with a counter on it.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::IfAmountAtLeast {
                        value: AmountValue::Stat(StatValue::CounterBearingPermanents),
                        minimum: 1,
                        skip: 1,
                    },
                    Instruction::Draw(PlayerSet::Controller, 1),
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
