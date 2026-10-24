// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 5, 0, 0,
        ]))],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/blazing_firesinger_seething_song"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
