// Direct typed behavior from complete pinned Oracle and XMage sources.
const RHINOS: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some("Rhino"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::MENACE,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, RHINOS)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::MENACE,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: true,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: true,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Subtype("Swamp"),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        keyword_grants: &[RHINOS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
