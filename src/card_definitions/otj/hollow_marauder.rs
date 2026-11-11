const GRAVE_CREATURE_REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Graveyard,
            relation: 0,
            kind: 1,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(
            0,
            GRAVE_CREATURE_REDUCTION,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Player {
                opponent_only: true,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::DiscardOneEachDrawUnlessManaValue {
            players: PlayerSet::Targeted,
            minimum: 4,
        }],
    }],
    extras: &CardExtras {
        cost_reductions: &[GRAVE_CREATURE_REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
