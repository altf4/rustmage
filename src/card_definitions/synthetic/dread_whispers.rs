// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card::from_catalogue_id(2859)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
