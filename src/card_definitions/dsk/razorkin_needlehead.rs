const FIRST_STRIKE_ON_OWN_TURN: KeywordGrant = KeywordGrant {
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
    keywords: Keywords::FIRST_STRIKE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::Opponent,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::DrawingPlayer, 1)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(
            0,
            FIRST_STRIKE_ON_OWN_TURN,
        )],
    }],
    extras: &CardExtras {
        keyword_grants: &[FIRST_STRIKE_ON_OWN_TURN],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
