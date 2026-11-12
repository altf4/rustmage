pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrTurnsFaceUp,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::RequireSubjectCombat {
            subject: Subject::Targets,
            requirement: CombatRequirement::Block,
            duration: CombatRequirementDuration::ThisTurn,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
