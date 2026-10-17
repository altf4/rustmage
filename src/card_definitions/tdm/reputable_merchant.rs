// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrDies,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
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
    keywords: Keywords::NONE,
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 1, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 1, 0, 1, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
