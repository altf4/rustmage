// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: generic(3),
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::All,
                    filter: LibrarySelectionFilter::Kinds(2),
                    optional: true,
                    reveal: false,
                    remainder: LibraryRemainder::Shuffle,
                },
                look: LibraryLook::Amount(AmountValue::SourcePower),
                destination: LibraryPickDestination::BattlefieldTapped,
                public: false,
                maximum_mana_value: None,
            })],
        },
    ],
    extras: &CardExtras {
        devour: Some(DevourSpec {
            kinds: 2,
            counters_per_permanent: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
