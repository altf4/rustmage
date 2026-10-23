// Direct typed behavior from complete Oracle and pinned XMage sources.
// The six modes enumerate every legal unordered choice of two printed modes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a token that is a copy of target Kithkin you control. Target player creates a green and white Kithkin token.",
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
                    predicate: &TargetPredicate::Subtype("Kithkin"),
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
                    token: Card(5306),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Kithkin you control. Target creature you control gets +3/+3 until end of turn.",
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
                    predicate: &TargetPredicate::Subtype("Kithkin"),
                }),
                TargetSpec::one(TargetRule::creature_you_control()),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateCopyToken {
                    subject: Subject::Targets,
                    amount: 1,
                },
                Instruction::SelectGroup(1),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 3,
                },
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Kithkin you control. Target creature you control fights target opposing creature.",
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
                    predicate: &TargetPredicate::Subtype("Kithkin"),
                }),
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateCopyToken {
                    subject: Subject::Targets,
                    amount: 1,
                },
                Instruction::SelectGroup(1),
                Instruction::Fight {
                    source: 1,
                    recipient: 2,
                },
            ],
        },
        SpellMode {
            text: "Target player creates a green and white Kithkin token. Target creature you control gets +3/+3 until end of turn.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::creature_you_control()),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateTokensForTargetPlayer {
                    token: Card(5306),
                    amount: 1,
                    tapped: false,
                },
                Instruction::SelectGroup(1),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 3,
                },
            ],
        },
        SpellMode {
            text: "Target player creates a green and white Kithkin token. Target creature you control fights target opposing creature.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateTokensForTargetPlayer {
                    token: Card(5306),
                    amount: 1,
                    tapped: false,
                },
                Instruction::SelectGroup(1),
                Instruction::Fight {
                    source: 1,
                    recipient: 2,
                },
            ],
        },
        SpellMode {
            text: "Target creature you control gets +3/+3 until end of turn. Target creature you control fights target opposing creature.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 3,
                },
                Instruction::SelectGroup(1),
                Instruction::Fight {
                    source: 1,
                    recipient: 2,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
