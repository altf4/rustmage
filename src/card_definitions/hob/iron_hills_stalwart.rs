// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::REACH.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Filtered {
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
            }),
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            },
        ]),
        program: &[Instruction::AttachEquipmentByGroups {
            equipment: 0,
            recipient: 1,
        }],
    }],
    ..CardRules::EMPTY
};
