// Direct typed behavior from complete Oracle and pinned XMage sources.
// The six modes enumerate every legal unordered choice of two printed modes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a token that is a copy of target Elemental you control. Target player draws two cards.",
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
                    predicate: &TargetPredicate::Subtype("Elemental"),
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
                Instruction::Draw(PlayerSet::Targeted, 2),
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Elemental you control. Deal 2 damage to each creature target player controls.",
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
                    predicate: &TargetPredicate::Subtype("Elemental"),
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
                Instruction::DamagePlayerCreatures {
                    players: PlayerSet::Targeted,
                    amount: 2,
                },
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Elemental you control. Target player creates two Treasure tokens.",
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
                    predicate: &TargetPredicate::Subtype("Elemental"),
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
                Instruction::CreateTokensForTargetPlayer {
                    token: Card(5302),
                    amount: 2,
                    tapped: false,
                },
            ],
        },
        SpellMode {
            text: "Target player draws two cards. Deal 2 damage to each creature target player controls.",
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
                Instruction::Draw(PlayerSet::Targeted, 2),
                Instruction::SelectGroup(1),
                Instruction::DamagePlayerCreatures {
                    players: PlayerSet::Targeted,
                    amount: 2,
                },
            ],
        },
        SpellMode {
            text: "Target player draws two cards. Target player creates two Treasure tokens.",
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
                Instruction::Draw(PlayerSet::Targeted, 2),
                Instruction::SelectGroup(1),
                Instruction::CreateTokensForTargetPlayer {
                    token: Card(5302),
                    amount: 2,
                    tapped: false,
                },
            ],
        },
        SpellMode {
            text: "Deal 2 damage to each creature target player controls. Target player creates two Treasure tokens.",
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
                Instruction::DamagePlayerCreatures {
                    players: PlayerSet::Targeted,
                    amount: 2,
                },
                Instruction::SelectGroup(1),
                Instruction::CreateTokensForTargetPlayer {
                    token: Card(5302),
                    amount: 2,
                    tapped: false,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
