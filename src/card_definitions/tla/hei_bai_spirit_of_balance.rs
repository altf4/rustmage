const OFFER: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::SelfEnters,
    targets: TargetPlan::NONE,
    program: &[
        Instruction::ChooseObjects {
            kind: EffectChoiceKind::Sacrifice,
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1 | 16,
                nonland: false,
                other: true,
                tokens: None,
                subtype: None,
            },
            amount: 1,
            optional_skip: Some(1),
        },
        Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 2,
        },
    ],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        OFFER,
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            ..OFFER
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                subject: DepartureSubject::SelfOnly,
                ..DepartureRule::ANY
            }),
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::PutEventCountersOnTargets],
        },
    ],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
