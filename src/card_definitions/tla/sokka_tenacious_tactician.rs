const MENACE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Ally"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::MENACE,
};
const NONCREATURE_CAST: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::Noncreature,
    ordinal: None,
};
const PROWESS: &[Instruction] = &[Instruction::Grow {
    subject: Subject::Source,
    power: 1,
    toughness: 1,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::MENACE,
    triggers: &[
        TriggeredAbility {
            condition: NONCREATURE_CAST,
            targets: TargetPlan::NONE,
            program: PROWESS,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&NONCREATURE_CAST),
            targets: TargetPlan::NONE,
            program: PROWESS,
        },
        TriggeredAbility {
            condition: NONCREATURE_CAST,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5272),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, MENACE),
            StaticAbilityFragment::ControlledSubtypeCastTrigger {
                ability: 1,
                other: true,
                subtype: "Ally",
            },
        ],
    }],
    extras: &CardExtras {
        keyword_grants: &[MENACE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
