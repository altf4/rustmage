pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::Nontoken,
        }),
        &[Instruction::BlinkExiledTargetsUntilEndStep],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(4722)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
