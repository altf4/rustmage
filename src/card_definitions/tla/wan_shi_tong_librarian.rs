// Complete typed Oracle behavior from the full pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLASH
        .union(Keywords::FLYING)
        .union(Keywords::VIGILANCE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCountersScaled {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::ChosenX,
                },
                Instruction::DrawScaled(
                    PlayerSet::Controller,
                    AmountValue::Divided {
                        value: &AmountValue::ChosenX,
                        divisor: std::num::NonZeroU16::new(2).unwrap(),
                    },
                ),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::LibrarySearched {
                relation: CastRelation::Opponent,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FLASH
                .union(Keywords::FLYING)
                .union(Keywords::VIGILANCE),
        )],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
