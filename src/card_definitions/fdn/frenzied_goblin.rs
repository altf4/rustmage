// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: one(TargetRule::Creature),
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 1, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BLOCK,
            },
        ],
    }],
    ..CardRules::EMPTY
};
