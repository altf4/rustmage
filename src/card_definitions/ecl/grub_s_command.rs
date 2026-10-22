// Direct typed behavior from complete Oracle and pinned XMage sources.
// Six unordered pairs preserve printed order and independent target groups.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a token that is a copy of target Goblin you control. Creatures target player controls get +1/+1 and gain haste until end of turn.",
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
                    predicate: &TargetPredicate::Subtype("Goblin"),
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
                    power: 1,
                    toughness: 1,
                    keywords: Keywords::HASTE,
                },
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Goblin you control. Destroy target artifact or creature.",
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
                    predicate: &TargetPredicate::Subtype("Goblin"),
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 17,
                    relation: -1,
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
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Goblin you control. Target player mills five cards and puts each Goblin card milled this way into their hand.",
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
                    predicate: &TargetPredicate::Subtype("Goblin"),
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
                Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(5)),
                Instruction::SelectTargetPlayerMilled(MilledSelectionSpec {
                    filter: MilledFilter::KindsOrSubtype {
                        kinds: 0,
                        subtype: "Goblin",
                    },
                    count: MilledCount::All,
                    destination: MilledDestination::Hand,
                }),
            ],
        },
        SpellMode {
            text: "Creatures target player controls get +1/+1 and gain haste until end of turn. Destroy target artifact or creature.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 17,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ModifyTargetPlayerCreatures {
                    power: 1,
                    toughness: 1,
                    keywords: Keywords::HASTE,
                },
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "Creatures target player controls get +1/+1 and gain haste until end of turn. Target player mills five cards and puts each Goblin card milled this way into their hand.",
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
                    power: 1,
                    toughness: 1,
                    keywords: Keywords::HASTE,
                },
                Instruction::SelectGroup(1),
                Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(5)),
                Instruction::SelectTargetPlayerMilled(MilledSelectionSpec {
                    filter: MilledFilter::KindsOrSubtype {
                        kinds: 0,
                        subtype: "Goblin",
                    },
                    count: MilledCount::All,
                    destination: MilledDestination::Hand,
                }),
            ],
        },
        SpellMode {
            text: "Destroy target artifact or creature. Target player mills five cards and puts each Goblin card milled this way into their hand.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 17,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(5)),
                Instruction::SelectTargetPlayerMilled(MilledSelectionSpec {
                    filter: MilledFilter::KindsOrSubtype {
                        kinds: 0,
                        subtype: "Goblin",
                    },
                    count: MilledCount::All,
                    destination: MilledDestination::Hand,
                }),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
