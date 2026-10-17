// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 4,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(371),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::LandEntersYou,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 2,
            }],
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
