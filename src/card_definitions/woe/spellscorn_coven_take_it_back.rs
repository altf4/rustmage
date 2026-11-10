pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Discard(PlayerSet::Opponent, 1)],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5561)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
