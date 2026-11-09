// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SelectLibraryToHandRememberCount(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 1,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Kinds(1 | 16),
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::Keep,
                },
                look: LibraryLook::Fixed(1),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            }),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 2,
            },
            Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            },
            Instruction::TransformSource,
        ],
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
