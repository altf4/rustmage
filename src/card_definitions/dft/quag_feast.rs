// Full pinned Oracle and XMage: choose before milling, compare at resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::CreatureOrVehicle { relation: -1 },
            TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 64,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        ]))),
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
            Instruction::DestroyTargetsAtMostManaValue(AmountValue::Stat(StatValue::Count(
                CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                },
            ))),
        ],
    ),
    ..CardRules::EMPTY
};
