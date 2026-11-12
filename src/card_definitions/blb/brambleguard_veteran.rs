// Direct typed behavior from complete pinned Oracle and XMage sources.
const RACCOONS: &[BattlefieldFilter] = &[BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: Some("Raccoon"),
    max_mana: u32::MAX,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Expend(4),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowPermanentGroup {
                filters: RACCOONS,
                power: 1,
                toughness: 1,
            },
            Instruction::GrantPermanentGroupKeywords {
                filters: RACCOONS,
                keywords: Keywords::VIGILANCE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
