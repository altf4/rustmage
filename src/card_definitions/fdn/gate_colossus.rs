// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
const AFFINITY: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Gate"),
            basic: false,
            other: false,
            cards_only: false,
        }),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const STATIC_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, AFFINITY)],
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
    effect: Effect::Creature(8, 8),
    keywords: Keywords::NO_POWER_LE_TWO_BLOCKERS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::SubtypeEntersYou("Gate"),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::PutSourceOnLibraryTop {
                origin: TargetZone::Graveyard,
            },
        ],
    }],
    static_groups: STATIC_GROUPS,
    extras: &CardExtras {
        static_ability_groups: STATIC_GROUPS,
        cost_reductions: &[AFFINITY],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
