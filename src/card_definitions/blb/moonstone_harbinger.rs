const BATS: &[BattlefieldFilter] = &[BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: Some("Bat"),
    max_mana: u32::MAX,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::FLYING.union(Keywords::DEATHTOUCH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::LifeGainedOrLostOwnTurn),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowPermanentGroup {
                filters: BATS,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantPermanentGroupKeywords {
                filters: BATS,
                keywords: Keywords::DEATHTOUCH,
            },
        ],
    }],
    ..CardRules::EMPTY
};
