pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamageBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            players: Some(PlayerSet::Opponent),
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
