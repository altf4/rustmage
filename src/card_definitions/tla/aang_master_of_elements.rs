const fn discount(color: ChoiceColor) -> SpellReduction {
    SpellReduction {
        source_only: false,
        filter: CastFilter::Any,
        amount: SpellReductionAmount::ColoredStat {
            color,
            value: StatValue::Fixed(1),
            factor: 1,
        },
        turn: CostTurn::Any,
        guard: StepGuard::Always,
    }
}

const DISCOUNTS: &[SpellReduction] = &[
    discount(ChoiceColor::White),
    discount(ChoiceColor::Blue),
    discount(ChoiceColor::Black),
    discount(ChoiceColor::Red),
    discount(ChoiceColor::Green),
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::CostReduction(0, DISCOUNTS[0]),
            StaticAbilityFragment::CostReduction(1, DISCOUNTS[1]),
            StaticAbilityFragment::CostReduction(2, DISCOUNTS[2]),
            StaticAbilityFragment::CostReduction(3, DISCOUNTS[3]),
            StaticAbilityFragment::CostReduction(4, DISCOUNTS[4]),
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayTransformSource { skip: 5 },
            Instruction::TransformSource,
            Instruction::GainLife(PlayerSet::Controller, 4),
            Instruction::Draw(PlayerSet::Controller, 4),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 4,
            },
            Instruction::DamagePlayers(PlayerSet::Opponent, 4),
        ],
    }],
    extras: &CardExtras {
        cost_reductions: DISCOUNTS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
