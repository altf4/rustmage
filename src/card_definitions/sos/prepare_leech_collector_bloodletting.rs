// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseLife(PlayerSet::Opponent, 2)],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/leech_collector_bloodletting"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
