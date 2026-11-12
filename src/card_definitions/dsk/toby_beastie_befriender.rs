// Directly authored from the paired Oracle and pinned XMage sources.
const TOKEN_FLIGHT: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::AtLeast {
        value: &StatValue::Tokens(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 1,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        minimum: 4,
    }),
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: Some(true),
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::FLYING,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5674),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, TOKEN_FLIGHT)],
    }],
    extras: &CardExtras {
        keyword_grants: &[TOKEN_FLIGHT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
