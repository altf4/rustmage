// Complete Saga; each damage recipient is frozen before one simultaneous batch.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5312),
                amount: 2,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageBattlefieldExceptSubtype {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                excluded_subtype: "Villain",
                players: Some(PlayerSet::Opponent),
                amount: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card(5302),
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Villain"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
