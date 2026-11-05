// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: 3,
        })),
        program: &[Instruction::ExileTargetsCopyFreeWithBudget {
            mana_value: u64::MAX,
        }],
    }],
    ..CardRules::EMPTY
};
