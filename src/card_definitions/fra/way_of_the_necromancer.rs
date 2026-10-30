// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::You,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPermanentCountersScaled {
                filters: &[BattlefieldFilter {
                    kinds: 64,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                kind: CounterKind::Loyalty,
                amount: AmountValue::Fixed(1),
            }],
        },
    ],
    ..CardRules::EMPTY
};
