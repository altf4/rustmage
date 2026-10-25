// Gated typed attachment consumer; whole paid admission follows shared-core proof.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OtherSubtypeEntersYou("Villain"),
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
                TargetSpec::one(TargetRule::creature_you_control()),
            ]),
            program: &[Instruction::AttachEquipmentByGroups {
                equipment: 0,
                recipient: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledEquippedCreatureAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::Connive {
                subject: Subject::Entering,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
