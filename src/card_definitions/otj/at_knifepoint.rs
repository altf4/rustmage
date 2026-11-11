const fn first_strike(subtype: &'static str) -> KeywordGrant {
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
    }
}
const GRANTS: &[KeywordGrant] = &[
    first_strike(OUTLAW_SUBTYPES[0]),
    first_strike(OUTLAW_SUBTYPES[1]),
    first_strike(OUTLAW_SUBTYPES[2]),
    first_strike(OUTLAW_SUBTYPES[3]),
    first_strike(OUTLAW_SUBTYPES[4]),
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, GRANTS[0]),
            StaticAbilityFragment::KeywordGrant(1, GRANTS[1]),
            StaticAbilityFragment::KeywordGrant(2, GRANTS[2]),
            StaticAbilityFragment::KeywordGrant(3, GRANTS[3]),
            StaticAbilityFragment::KeywordGrant(4, GRANTS[4]),
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
    extras: &CardExtras {
        keyword_grants: GRANTS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
