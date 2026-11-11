const fn first_strike(subtype: &'static str, slot: u16) -> StaticAbilityFragment {
    StaticAbilityFragment::KeywordGrant(
        slot,
        KeywordGrant {
            condition: None,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: Some(subtype),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            source_only: false,
            own_turn: true,
            keywords: Keywords::FIRST_STRIKE,
        },
    )
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            first_strike(OUTLAW_SUBTYPES[0], 0),
            first_strike(OUTLAW_SUBTYPES[1], 1),
            first_strike(OUTLAW_SUBTYPES[2], 2),
            first_strike(OUTLAW_SUBTYPES[3], 3),
            first_strike(OUTLAW_SUBTYPES[4], 4),
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CommittedCrime),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5612),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
