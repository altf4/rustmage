// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[
                Instruction::AnimateEarthbendTargets,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                },
                Instruction::RegisterEarthbendReturn,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CounterPlacedByYou {
                kind: CounterKind::PlusOne,
                filter: CountFilter {
                    zone: CountZone::Battlefield,
                    relation: -1,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayOncePerTurn { skip: 2 },
                Instruction::RecordOncePerTurn,
                Instruction::GainLifeScaled(
                    PlayerSet::Controller,
                    AmountValue::CounterPlacementAmount,
                ),
            ],
        },
    ],
    ..CardRules::EMPTY
};
