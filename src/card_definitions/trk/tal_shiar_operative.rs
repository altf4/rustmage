// Oracle and pinned XMage reviewed. Modal choice and its target are bound at trigger resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Target creature you control gains deathtouch until end of turn.",
                targets: one(TargetRule::creature_you_control()),
                program: &[Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::DEATHTOUCH,
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
