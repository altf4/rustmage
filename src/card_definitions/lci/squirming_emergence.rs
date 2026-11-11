const GRAVEYARD_PERMANENTS: AmountValue = AmountValue::Stat(StatValue::Count(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 1 | 2 | 16 | 32 | 64 | 256,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
}));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::ManaValueAtMost {
            base: &TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Permanent,
            },
            amount: GRAVEYARD_PERMANENTS,
            maximum: None,
        }),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};
