pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksDefendingPlayer,
        targets: one(TargetRule::DefendingCreature),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: -3,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
