// Oracle and pinned XMage evidence are recorded in the paired provenance.
const OWN_TURN_FLIGHT: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: true,
    own_turn: true,
    keywords: Keywords::FLYING,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfLeaves,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5314),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, OWN_TURN_FLIGHT)],
    }],
    extras: &CardExtras {
        keyword_grants: &[OWN_TURN_FLIGHT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
