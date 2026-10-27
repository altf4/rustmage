// Direct typed behavior from pinned Oracle and XMage sources.
const LAND_COUNT: StatExpr = StatExpr {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 2,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: false,
    }),
    multiplier: 1,
    offset: 0,
};
const STATS: DynamicStats = DynamicStats {
    power: Some(LAND_COUNT),
    toughness: Some(LAND_COUNT),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::NO_POWER_LE_TWO_BLOCKERS,
        )],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::NO_POWER_LE_TWO_BLOCKERS,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        dynamic_stats: Some(STATS),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 3,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::ReturnSourceAndTargetsTapped],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
