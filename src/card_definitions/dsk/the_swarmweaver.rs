// Directly authored from the paired Oracle and pinned XMage sources.
const DELIRIUM: StatValue = StatValue::AtLeast {
    value: &StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
    minimum: 4,
};

const INSECT: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: Some("Insect"),
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const SPIDER: CreatureFilter = CreatureFilter {
    subtype: Some("Spider"),
    ..INSECT
};
const DEATHTOUCH: KeywordGrant = KeywordGrant {
    condition: Some(DELIRIUM),
    requires_life_gain: false,
    filter: INSECT,
    source_only: false,
    own_turn: false,
    keywords: Keywords::DEATHTOUCH,
};
const GRANTS: &[KeywordGrant] = &[
    DEATHTOUCH,
    KeywordGrant {
        filter: SPIDER,
        ..DEATHTOUCH
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5244),
            amount: 2,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, GRANTS[0]),
            StaticAbilityFragment::KeywordGrant(1, GRANTS[1]),
            StaticAbilityFragment::CreatureGroupScaling {
                filters: &[INSECT, SPIDER],
                value: DELIRIUM,
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        ],
    }],
    extras: &CardExtras {
        keyword_grants: GRANTS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
