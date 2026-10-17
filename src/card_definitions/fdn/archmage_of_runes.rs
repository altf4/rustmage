// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(
            0,
            SpellReduction {
                source_only: false,
                filter: CastFilter::InstantSorcery,
                amount: SpellReductionAmount::Fixed(1),
                turn: CostTurn::Any,
                guard: StepGuard::Always,
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                SpellReduction {
                    source_only: false,
                    filter: CastFilter::InstantSorcery,
                    amount: SpellReductionAmount::Fixed(1),
                    turn: CostTurn::Any,
                    guard: StepGuard::Always,
                },
            )],
        }],
        cost_reductions: &[SpellReduction {
            source_only: false,
            filter: CastFilter::InstantSorcery,
            amount: SpellReductionAmount::Fixed(1),
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
