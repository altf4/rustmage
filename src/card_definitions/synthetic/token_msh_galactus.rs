pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(16, 16),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::Destroy],
    }],
    ..CardRules::EMPTY
};
