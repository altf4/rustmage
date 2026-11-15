// Oracle and pinned XMage: choose the ETB mode before selecting its mode-specific target.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Target creature gets +1/+0 until end of turn and can't be blocked this turn.",
                targets: one(TargetRule::Creature),
                program: &[
                    Instruction::Grow {
                        subject: Subject::Targets,
                        power: 1,
                        toughness: 0,
                    },
                    Instruction::GrantKeywords {
                        subject: Subject::Targets,
                        keywords: Keywords::CANT_BE_BLOCKED,
                    },
                ],
            },
            SpellMode {
                text: "Surveil 2.",
                targets: TargetPlan::NONE,
                program: &[Instruction::Surveil(2)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
