const GORGON_BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(4),
    own_turn: false,
    condition: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Damaged,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::Destroy],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::SourceUnsolved(&StepGuard::Expression {
                    value: StatValue::CreatureCardsPutIntoGraveyardsThisTurn,
                    minimum: Some(3),
                    maximum: None,
                }),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceAdditionalCreature {
                subtypes: &["Gorgon"],
                keywords: Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
            },
            StaticAbilityFragment::BaseStatSetter(0, GORGON_BASE),
        ],
    }],
    extras: &CardExtras {
        base_stat_setter: Some(GORGON_BASE),
        solved_static_groups: &[2],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
