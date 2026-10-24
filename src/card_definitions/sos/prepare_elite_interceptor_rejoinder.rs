// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::May { skip: 2 },
            Instruction::Tap,
            Instruction::Skip(2),
            Instruction::May { skip: 1 },
            Instruction::Untap,
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/elite_interceptor_rejoinder"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
