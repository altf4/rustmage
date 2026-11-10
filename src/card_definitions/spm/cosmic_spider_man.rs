const POWERS: Keywords = Keywords::FLYING
    .union(Keywords::FIRST_STRIKE)
    .union(Keywords::TRAMPLE)
    .union(Keywords::LIFELINK)
    .union(Keywords::HASTE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: POWERS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantOtherPermanentGroupKeywords {
            filters: &[BattlefieldFilter {
                kinds: 0,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: Some("Spider"),
                max_mana: u32::MAX,
            }],
            keywords: POWERS,
        }],
    }],
    ..CardRules::EMPTY
};
