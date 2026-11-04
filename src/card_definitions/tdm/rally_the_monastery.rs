// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create two Monks",
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5486),
                amount: 2,
                tapped: false,
            }],
        },
        SpellMode {
            text: "Pump up to two creatures",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            }],
        },
        SpellMode {
            text: "Destroy a large creature",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::PowerAtLeast(4),
            })),
            program: &[Instruction::Destroy],
        },
    ]),
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(2),
            turn: CostTurn::Any,
            guard: StepGuard::Threshold {
                value: StepValue::SpellsCast,
                minimum: Some(1),
                maximum: None,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
