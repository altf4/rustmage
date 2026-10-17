// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfOffspring,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5300),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::ExcludingKinds(3),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            })],
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        offspring_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 4,
        }),
        offspring_copy: Some(Card(5300)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
