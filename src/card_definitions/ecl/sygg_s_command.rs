// Direct typed behavior from complete Oracle and pinned XMage sources.
// The six modes enumerate every legal unordered choice of two printed modes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a token that is a copy of target Merfolk you control. Creatures target player controls gain lifelink until end of turn.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Merfolk"),
                }),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateCopyToken {
                    subject: Subject::Targets,
                    amount: 1,
                },
                Instruction::SelectGroup(1),
                Instruction::ModifyTargetPlayerCreatures {
                    power: 0,
                    toughness: 0,
                    keywords: Keywords::LIFELINK,
                },
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Merfolk you control. Target player draws a card.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Merfolk"),
                }),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateCopyToken {
                    subject: Subject::Targets,
                    amount: 1,
                },
                Instruction::SelectGroup(1),
                Instruction::Draw(PlayerSet::Targeted, 1),
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Merfolk you control. Tap target creature. Put a stun counter on it.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Subtype("Merfolk"),
                }),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateCopyToken {
                    subject: Subject::Targets,
                    amount: 1,
                },
                Instruction::SelectGroup(1),
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
            ],
        },
        SpellMode {
            text: "Creatures target player controls gain lifelink until end of turn. Target player draws a card.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ModifyTargetPlayerCreatures {
                    power: 0,
                    toughness: 0,
                    keywords: Keywords::LIFELINK,
                },
                Instruction::SelectGroup(1),
                Instruction::Draw(PlayerSet::Targeted, 1),
            ],
        },
        SpellMode {
            text: "Creatures target player controls gain lifelink until end of turn. Tap target creature. Put a stun counter on it.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ModifyTargetPlayerCreatures {
                    power: 0,
                    toughness: 0,
                    keywords: Keywords::LIFELINK,
                },
                Instruction::SelectGroup(1),
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
            ],
        },
        SpellMode {
            text: "Target player draws a card. Tap target creature. Put a stun counter on it.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Draw(PlayerSet::Targeted, 1),
                Instruction::SelectGroup(1),
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
