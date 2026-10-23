// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensForPlayers {
                players: PlayerSet::Each,
                token: Card(5246),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::PermanentToGraveyard {
                    kinds: 16,
                    relation: CastRelation::Opponent,
                    exclude_source_kinds: 0,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::OpponentAttacked,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Attacking,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::Stat(StatValue::ControllerHandSize),
                power: 1,
                toughness: 1,
            }],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SetClassLevel(2)],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SetClassLevel(3)],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        class_level_activations: &[(0, 2), (1, 3)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
