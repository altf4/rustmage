// Directly authored from full Oracle, pinned Java and CR608.2h damage LKI.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
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
                    max: usize::MAX,
                },
                TargetSpec::one(TargetRule::creature_you_control()),
            ]),
            program: &[Instruction::AttachEquipmentGroupsReflexive {
                equipment: 0,
                recipient: 1,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::EventObjectPowerDamage],
        },
    ],
    ..CardRules::EMPTY
};
