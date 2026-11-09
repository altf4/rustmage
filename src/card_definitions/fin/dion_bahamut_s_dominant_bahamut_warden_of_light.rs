const CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const DION_FLYING: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CREATURES,
    source_only: true,
    own_turn: true,
    keywords: Keywords::FLYING,
};
const KNIGHT_FLYING: KeywordGrant = KeywordGrant {
    filter: CreatureFilter {
        other: true,
        subtype: Some("Knight"),
        ..CREATURES
    },
    source_only: false,
    ..DION_FLYING
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, DION_FLYING),
            StaticAbilityFragment::KeywordGrant(1, KNIGHT_FLYING),
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5236),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 4,
            colored: [2, 0, 0, 0, 0, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileBattlefieldSourceReturn {
            transformed: true,
            under_owner: true,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        keyword_grants: &[DION_FLYING, KNIGHT_FLYING],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
