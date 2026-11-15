// Pinned XMage: the entry trigger chooses a targeted boost or surveil.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Target creature gets +1/+1 until end of turn.",
                targets: one(TargetRule::Creature),
                program: &[Instruction::Grow {
                    subject: Subject::Targets,
                    power: 1,
                    toughness: 1,
                }],
            },
            SpellMode {
                text: "Surveil 1.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(1)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
