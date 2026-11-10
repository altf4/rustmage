// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Groups(&[
                TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: -1,
                        other: false,
                        nonland: true,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    min: 0,
                    max: 1,
                },
                TargetSpec {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Graveyard,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: true,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Permanent,
                    },
                    min: 0,
                    max: 1,
                },
            ]),
            program: &[Instruction::ExileAllTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::BottomDepartedSourceAndReturnLinkedExile],
        },
    ],
    ..CardRules::EMPTY
};
