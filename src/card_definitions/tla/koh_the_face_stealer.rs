// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::OtherThanSource,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ExileTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::Any,
                other: true,
                nontoken: true,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 1 },
                Instruction::ExileEventGraveyardCardLinked,
            ],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            life: 1,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseLinkedCreatureAbilities],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantChosenLinkedCreatureAbilities],
    }],
    ..CardRules::EMPTY
};
