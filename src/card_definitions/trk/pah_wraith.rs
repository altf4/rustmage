// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FIRST_STRIKE.union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreaturesScaled {
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            excluding_subtype: None,
            amount: AmountValue::Stat(StatValue::DistinctNames(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
            power: 1,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
