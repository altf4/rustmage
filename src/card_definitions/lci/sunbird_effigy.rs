const COLORS: StatExpr = StatExpr {
    value: StatValue::CraftMaterialColorCount,
    multiplier: 1,
    offset: 0,
};
const STATS: DynamicStats = DynamicStats {
    power: Some(COLORS),
    toughness: Some(COLORS),
};
const KEYWORDS: Keywords = Keywords::FLYING
    .union(Keywords::VIGILANCE)
    .union(Keywords::HASTE);
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, STATS)],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: KEYWORDS,
    static_groups: GROUPS,
    extras: &CardExtras {
        dynamic_stats: Some(STATS),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::CraftMaterialColors)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
