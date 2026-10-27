// Directly authored from full pinned Oracle, XMage and search/cycling helpers.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SubtypeEntersYou("Mountain"),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::SourceCounters(CounterKind::Quest),
                    minimum: Some(6),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::SacrificeSourceOrSkip { skip: 1 },
            Instruction::SearchHandAndOrLibrary(SearchFilter::Subtype("Dragon")),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: true,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: true,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[SearchFilter::Land, SearchFilter::Subtype("Mountain")]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
