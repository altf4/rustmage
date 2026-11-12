pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Damage(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDiesBatch {
                filter: CreatureFilter {
                    relation: -1,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::Bloodstain,
                    amount: 1,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::Stat(StatValue::SourceCounters(CounterKind::Bloodstain)),
                    minimum: 5,
                    skip: 1,
                },
                Instruction::SacrificeSourceReflexive { ability: 2 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnToHand],
        },
    ],
    ..CardRules::EMPTY
};
