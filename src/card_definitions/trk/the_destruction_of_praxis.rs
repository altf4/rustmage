// Complete Oracle and pinned TheDestructionOfPraxis.java reviewed. X is
// sampled once for the global modifier and the six-name token threshold.
const DISTINCT_OWN_LANDS: StatValue = StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ModifyPlayerCreaturesScaled {
            players: PlayerSet::Each,
            value: DISTINCT_OWN_LANDS,
            power_per_count: -1,
            toughness_per_count: -1,
            create_if_at_least: Some((6, Card(6010), 2)),
        }],
    ),
    ..CardRules::EMPTY
};
