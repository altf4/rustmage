const BAT: Card = Card::from_catalogue_id(5276);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DiscardRememberCount {
                    players: PlayerSet::Opponent,
                    amount: 1,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::MovedCount,
                    minimum: 1,
                    skip: 1,
                },
                Instruction::Skip(1),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DiscardedKinds {
                relation: CastRelation::Opponent,
                kinds: 2,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: BAT,
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDepartedSourceTransformedTapped { counters: &[] }],
        },
    ],
    ..CardRules::EMPTY
};
