// Pinned XMage: kicker expands the announced spell target's mana-value range.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::SpellCharacteristics {
            colors: 0,
            min_mana: 0,
            max_mana: 2,
        }),
        &[Instruction::Counter],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }),
        kicked_targets: Some(one(TargetRule::Spell)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
