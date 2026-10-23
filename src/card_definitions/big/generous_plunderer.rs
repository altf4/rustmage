// Complete Oracle body authored from the paired pinned XMage source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 2 },
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::CreateTokensFor {
                players: PlayerSet::Targeted,
                token: Card(5302),
                amount: 1,
                tapped: true,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayersScaled(
                PlayerSet::Opponent,
                AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 1,
                    kind: 16,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            )],
        },
    ],
    ..CardRules::EMPTY
};
