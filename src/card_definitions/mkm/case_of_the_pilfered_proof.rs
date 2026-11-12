// Direct typed behavior from the pinned Oracle and XMage sources.
const EXTRA_CLUE: ReplacementSpec = ReplacementSpec::AddToken(Card::from_catalogue_id(5296));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
                entry: EntryPredicate::SubtypeAny(&["Detective"]),
                other_on_entry: false,
                face_up_subtype: "Detective",
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersOnEventPermanent {
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::SourceUnsolved(&StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 0,
                        exclude: 0,
                        subtype: Some("Detective"),
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: Some(3),
                    maximum: None,
                }),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, EXTRA_CLUE)],
    }],
    extras: &CardExtras {
        solved_static_groups: &[2],
        replacement_effects: &[EXTRA_CLUE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
