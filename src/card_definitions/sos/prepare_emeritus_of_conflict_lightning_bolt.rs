// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Any), &[Instruction::Damage(3)]),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/emeritus_of_conflict_lightning_bolt"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
