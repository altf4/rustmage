// Oracle and pinned XMage reviewed. Beam me up is a cast permission and stack-exit replacement.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Any), &[Instruction::Damage(3)]),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            generic: 3,
            colored: [0, 0, 0, 3, 0, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
