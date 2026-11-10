const SPIDER_OR_LEGENDARY_CREATURE: &[ControlledPermanentFilter] = &[
    ControlledPermanentFilter {
        subtype: Some("Spider"),
        ..ControlledPermanentFilter::ANY
    },
    ControlledPermanentFilter {
        all_kinds: 1,
        legendary: true,
        ..ControlledPermanentFilter::ANY
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FIRST_STRIKE.union(Keywords::HASTE),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::HEXPROOF,
            },
            Instruction::SetColors {
                subject: Subject::Source,
                colors: ColorValue::Fixed(0),
                duration: AnimationDuration::EndOfTurn,
            },
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChangeControlledPermanentGroupCounters {
            filters: SPIDER_OR_LEGENDARY_CREATURE,
            effect: PermanentGroupCounters::DoubleEachKind,
        }],
    }],
    ..CardRules::EMPTY
};
