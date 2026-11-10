const CREATURE: TargetSpec = TargetSpec::one(TargetRule::Creature);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Heckle",
                targets: TargetPlan::Single(CREATURE),
                program: &[Instruction::Tap],
            },
            SpellMode {
                text: "Hero Worship",
                targets: TargetPlan::Single(CREATURE),
                program: &[Instruction::Untap],
            },
            SpellMode {
                text: "Choose both",
                targets: TargetPlan::Groups(&[CREATURE, CREATURE]),
                program: &[
                    Instruction::Tap,
                    Instruction::SelectGroup(1),
                    Instruction::Untap,
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
