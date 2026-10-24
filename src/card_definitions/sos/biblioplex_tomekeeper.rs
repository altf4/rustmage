// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Choose no mode.",
                targets: TargetPlan::NONE,
                program: &[],
            },
            SpellMode {
                text: "Target creature becomes prepared.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
                program: &[Instruction::Prepare {
                    subject: Subject::Targets,
                }],
            },
            SpellMode {
                text: "Target creature becomes unprepared.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
                program: &[Instruction::Unprepare {
                    subject: Subject::Targets,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
