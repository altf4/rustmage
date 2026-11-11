const CONTROLLED_CAVES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Cave"),
    basic: false,
    other: false,
    cards_only: false,
};
const GRAVEYARD_CAVES: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    cards_only: true,
    ..CONTROLLED_CAVES
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamageBattlefieldScaled {
            filters: &[BattlefieldFilter {
                kinds: 1 | 64,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            amount: AmountValue::Stat(StatValue::Sum(&[
                StatValue::Count(CONTROLLED_CAVES),
                StatValue::Count(GRAVEYARD_CAVES),
            ])),
        }],
    ),
    ..CardRules::EMPTY
};
