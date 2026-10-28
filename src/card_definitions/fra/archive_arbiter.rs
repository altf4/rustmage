// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Destroy target noncreature, nonland permanent.",
                targets: one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: -1,
                        other: false,
                        nonland: true,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::WithoutKinds(1),
                }),
                program: &[Instruction::Destroy],
            },
            SpellMode {
                text: "You gain 4 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
