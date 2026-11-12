// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::GreatestCreaturePower(CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                }),
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
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
                Instruction::RememberCount(AmountValue::Stat(StatValue::ControllerHandSize)),
                Instruction::AnimateEarthbendTargets,
                Instruction::PutCountersScaled {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::RememberedCount,
                },
                Instruction::RegisterEarthbendReturn,
                Instruction::AddBasicLandType {
                    subject: Subject::Targets,
                    kind: BasicLandType::Island,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileBattlefieldSourceReturn {
                transformed: true,
                under_owner: false,
            }],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
