// Speed is bounded by the rules at four; these four live fragments sum to exactly that speed.
const fn speed_bonus(minimum: u16) -> StaticAbilityFragment {
    StaticAbilityFragment::ConditionalCreatureModifier {
        condition: StatValue::AtLeast {
            value: &StatValue::ControllerSpeed,
            minimum,
        },
        modifier: StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 1,
                toughness: 0,
            },
        },
    }
}
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Noncreature,
    amount: SpellReductionAmount::Stat {
        value: StatValue::ControllerSpeed,
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::FIRST_STRIKE
                .union(Keywords::VIGILANCE)
                .union(Keywords::HASTE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Modify,
        fragments: &[
            speed_bonus(1),
            speed_bonus(2),
            speed_bonus(3),
            speed_bonus(4),
        ],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FIRST_STRIKE
        .union(Keywords::VIGILANCE)
        .union(Keywords::HASTE),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
