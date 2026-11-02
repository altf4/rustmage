// Direct typed behavior from complete pinned Oracle and XMage sources.
const CREATURES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Deal damage equal to your creatures and Vehicles",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::DamageScaled(AmountValue::Stat(
                StatValue::CountUnion(&[
                    CREATURES,
                    CountFilter {
                        kind: 0,
                        subtype: Some("Vehicle"),
                        ..CREATURES
                    },
                ]),
            ))],
        },
        SpellMode {
            text: "Destroy target artifact",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Destroy],
        },
    ]),
    ..CardRules::EMPTY
};
