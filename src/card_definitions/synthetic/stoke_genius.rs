// Adventure face of Hearth Elemental.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DiscardHand(PlayerSet::Controller),
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1980)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
