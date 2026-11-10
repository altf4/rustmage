pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FIRST_STRIKE
        .union(Keywords::TRAMPLE)
        .union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DoubleDamageToDamagedPlayerUntilNextTurn],
    }],
    ..CardRules::EMPTY
};
