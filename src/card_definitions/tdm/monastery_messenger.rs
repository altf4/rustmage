// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::WithoutKinds(1),
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::TopLibrary],
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
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 1, 0, 0, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [1, 1, 0, 1, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
