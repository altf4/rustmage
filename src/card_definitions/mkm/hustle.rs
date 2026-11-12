pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::RequireSubjectCombat {
            subject: Subject::Targets,
            requirement: CombatRequirement::AttackOrBlock,
            duration: CombatRequirementDuration::ThisTurn,
        }],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(2110)),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
