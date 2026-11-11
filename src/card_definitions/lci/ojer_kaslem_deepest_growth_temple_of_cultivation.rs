// Direct typed behavior from the paired Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 0,
                    count: LibrarySelectionCount::DistinctCardTypes(1 | 2),
                    filter: LibrarySelectionFilter::Any,
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::BottomRandom,
                },
                look: LibraryLook::Amount(AmountValue::DamageEventAmount),
                destination: LibraryPickDestination::Battlefield,
                public: true,
                maximum_mana_value: None,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDepartedSourceTransformedTapped { counters: &[] }],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
