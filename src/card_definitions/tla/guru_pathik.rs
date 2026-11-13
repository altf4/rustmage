pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 5,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Subtype("Lesson"),
                    LibrarySelectionFilter::Subtype("Saga"),
                    LibrarySelectionFilter::Subtype("Shrine"),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::AnyOf(&[
                    CastFilter::Subtype("Lesson"),
                    CastFilter::Subtype("Saga"),
                    CastFilter::Subtype("Shrine"),
                ]),
                ordinal: None,
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 2,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
