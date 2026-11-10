// Oracle and pinned XMage evidence are recorded in the paired provenance.
const FIRST_STRIKE_DURING_OWN_TURN: KeywordGrant = KeywordGrant {
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
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Creature),
        program: &[Instruction::DamageWithController {
            damage: 2,
            controller_damage: 2,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(
            0,
            FIRST_STRIKE_DURING_OWN_TURN,
        )],
    }],
    extras: &CardExtras {
        keyword_grants: &[FIRST_STRIKE_DURING_OWN_TURN],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
