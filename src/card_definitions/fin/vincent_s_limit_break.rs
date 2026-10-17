// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Galian Beast — {0} — 3/2.",
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
            program: &[
                Instruction::GrantUntilEndTurn { ability: 0 },
                Instruction::SetBaseStats {
                    subject: Subject::Targets,
                    power: Some(3),
                    toughness: Some(2),
                },
            ],
        },
        SpellMode {
            text: "Death Gigas — {1} — 5/2.",
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
            program: &[
                Instruction::GrantUntilEndTurn { ability: 0 },
                Instruction::SetBaseStats {
                    subject: Subject::Targets,
                    power: Some(5),
                    toughness: Some(2),
                },
            ],
        },
        SpellMode {
            text: "Hellmasker — {3} — 7/2.",
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
            program: &[
                Instruction::GrantUntilEndTurn { ability: 0 },
                Instruction::SetBaseStats {
                    subject: Subject::Targets,
                    power: Some(7),
                    toughness: Some(2),
                },
            ],
        },
    ]),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfDies),
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ReturnGrantedSource {
            counter: None,
            role: None,
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
        tiered_costs: Some(&[
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 0, 0],
                generic: 3,
            },
        ]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
