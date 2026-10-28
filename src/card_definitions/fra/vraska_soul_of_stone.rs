// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
const ARTIFACT_CREATURE_VIGILANCE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: true,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::VIGILANCE,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::KeywordGrant(
        0,
        ARTIFACT_CREATURE_VIGILANCE,
    )],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5437),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        keyword_grants: &[ARTIFACT_CREATURE_VIGILANCE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
