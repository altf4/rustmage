const AFFINITY: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 64,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, AFFINITY)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesAttackYouOrPlaneswalkersAtLeast(2),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::EventPlayer, 3),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        cost_reductions: &[AFFINITY],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
