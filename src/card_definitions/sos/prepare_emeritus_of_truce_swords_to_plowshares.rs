// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::ExileAndGainLifeEqualPower],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/emeritus_of_truce_swords_to_plowshares"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
