const fn own_graveyard(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            own_graveyard(16),
            TargetRule::Filtered {
                base: &own_graveyard(32),
                predicate: &TargetPredicate::WithoutSubtype("Aura"),
            },
        ])),
        &[Instruction::ReturnGraveyardTargetsAnimated {
            animation: &AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Spirit"]),
                base: Some((1, 1)),
                keywords: Keywords::FLYING,
                duration: AnimationDuration::Permanent,
            },
            x_counters: Some(CounterKind::PlusOne),
        }],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
