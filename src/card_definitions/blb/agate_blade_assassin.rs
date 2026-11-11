// The defending player also applies when attacking a planeswalker or battle.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksDefendingPlayer,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::EventPlayer, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
