// Permanent, exact-incarnation evolution with resolution-time subtype prerequisites.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 0,
                alternatives: &[ManaCost {
                    colored: [1, 0, 0, 0, 0, 0],
                    generic: 0,
                }],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ChangeSourceCreatureForm(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Kithkin", "Scout"]),
                base: Some((2, 3)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            })],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 1,
                alternatives: &[
                    ManaCost {
                        colored: [1, 0, 0, 0, 1, 0],
                        generic: 1,
                    },
                    ManaCost {
                        colored: [2, 0, 0, 0, 0, 0],
                        generic: 1,
                    },
                ],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::EvolveSource {
                required: "Scout",
                types: &["Kithkin", "Soldier"],
                stats: Some(Stats {
                    power: 4,
                    toughness: 5,
                }),
                keywords: Keywords::NONE,
                ability: None,
            }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 3, 0],
                generic: 3,
                alternatives: &[
                    ManaCost {
                        colored: [1, 0, 0, 0, 2, 0],
                        generic: 3,
                    },
                    ManaCost {
                        colored: [2, 0, 0, 0, 1, 0],
                        generic: 3,
                    },
                    ManaCost {
                        colored: [3, 0, 0, 0, 0, 0],
                        generic: 3,
                    },
                ],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::EvolveSource {
                required: "Soldier",
                types: &["Kithkin", "Avatar"],
                stats: Some(Stats {
                    power: 7,
                    toughness: 8,
                }),
                keywords: Keywords::PROTECTION_OPPONENTS,
                ability: None,
            }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
