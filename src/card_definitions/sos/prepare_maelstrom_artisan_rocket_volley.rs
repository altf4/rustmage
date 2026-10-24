// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Nonbasic,
        }),
        &[Instruction::Destroy],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/maelstrom_artisan_rocket_volley"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
