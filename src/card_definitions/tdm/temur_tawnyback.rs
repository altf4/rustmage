// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
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
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 1, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
