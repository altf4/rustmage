// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::WithoutSubtype("Pilot"),
        }),
        &[Instruction::BlinkImmediately { counter: None }],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/skycoach_conductor_all_aboard"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
