// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPayCosts {
                subject: CounterSubject::Related,
                costs: &[ResolutionCost {
                    life: Some(AmountValue::SourcePower),
                    ..ResolutionCost::FREE
                }],
                exile: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Groups(&[
                TargetSpec {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 16,
                            relation: 0,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Subtype("Equipment"),
                    },
                    min: 0,
                    max: 1,
                },
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Attacking,
                }),
            ]),
            program: &[Instruction::AttachEquipmentByGroups {
                equipment: 0,
                recipient: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
