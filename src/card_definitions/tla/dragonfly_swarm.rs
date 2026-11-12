const POWER: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Graveyard,
            relation: 0,
            kind: 0,
            exclude: 1 | 2,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};

const LESSONS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Lesson"),
    basic: false,
    other: false,
    cards_only: true,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: generic(1),
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: LESSONS,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::SelfDies,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, POWER)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(POWER),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
