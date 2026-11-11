// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileTopPlayUntilNextEndStep(2)],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card::from_catalogue_id(3310)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
