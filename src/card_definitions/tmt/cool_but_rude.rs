// Direct typed class abilities; level-three trigger fires only on a genuine level change.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Discard,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::Discarded {
                    relation: CastRelation::You,
                    one_or_more: false,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::BecomesClassLevel(3),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::SearchRequired(SearchSpec {
                    filter: SearchFilter::Any,
                    maximum: 1,
                    destination: SearchDestination::Hand,
                    tapped: false,
                    reveal: false,
                }),
                Instruction::DiscardRandom(PlayerSet::Controller, 1),
            ],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::ClassLevelExactly(1),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(2)],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::ClassLevelExactly(2),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(3)],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
