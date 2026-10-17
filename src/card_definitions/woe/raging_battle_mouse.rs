// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Expression {
                value: StatValue::NonlandEntriesThisTurn,
                minimum: Some(2),
                maximum: None,
            },
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: 1,
            toughness: 1,
        }],
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
                filter: CastFilter::Any,
                amount: SpellReductionAmount::Fixed(1),
                turn: CostTurn::Any,
                guard: StepGuard::Threshold {
                    value: StepValue::SpellsCast,
                    minimum: Some(1),
                    maximum: Some(1),
                },
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
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::Fixed(1),
                    turn: CostTurn::Any,
                    guard: StepGuard::Threshold {
                        value: StepValue::SpellsCast,
                        minimum: Some(1),
                        maximum: Some(1),
                    },
                },
            )],
        }],
        cost_reductions: &[SpellReduction {
            source_only: false,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(1),
            turn: CostTurn::Any,
            guard: StepGuard::Threshold {
                value: StepValue::SpellsCast,
                minimum: Some(1),
                maximum: Some(1),
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
