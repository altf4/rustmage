// Direct typed behavior from complete Oracle and pinned XMage sources.
// All six legal choices of two printed modes, in printed resolution order.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Create a token that is a copy of target Elf you control. Return one or two target permanent cards from your graveyard to your hand.",
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
                    predicate: &TargetPredicate::Subtype("Elf"),
                }),
                TargetSpec {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Graveyard,
                            kinds: 0,
                            relation: 0,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Permanent,
                    },
                    min: 1,
                    max: 2,
                },
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::CreateCopyToken {
                    subject: Subject::Targets,
                    amount: 1,
                },
                Instruction::SelectGroup(1),
                Instruction::ReturnToHand,
            ],
        },
        SpellMode {
            text: "Create a token that is a copy of target Elf you control. Destroy target creature or enchantment.",
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
                    predicate: &TargetPredicate::Subtype("Elf"),
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 33,
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
            text: "Create a token that is a copy of target Elf you control. Creatures target player controls get +3/+3 until end of turn. Untap them.",
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
                    predicate: &TargetPredicate::Subtype("Elf"),
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
                    power: 3,
                    toughness: 3,
                    keywords: Keywords::NONE,
                },
                Instruction::UntapPlayerCreatures(PlayerSet::Targeted),
            ],
        },
        SpellMode {
            text: "Return one or two target permanent cards from your graveyard to your hand. Destroy target creature or enchantment.",
            targets: TargetPlan::Groups(&[
                TargetSpec {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Graveyard,
                            kinds: 0,
                            relation: 0,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Permanent,
                    },
                    min: 1,
                    max: 2,
                },
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 33,
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
                Instruction::ReturnToHand,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "Return one or two target permanent cards from your graveyard to your hand. Creatures target player controls get +3/+3 until end of turn. Untap them.",
            targets: TargetPlan::Groups(&[
                TargetSpec {
                    rule: TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Graveyard,
                            kinds: 0,
                            relation: 0,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Permanent,
                    },
                    min: 1,
                    max: 2,
                },
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ReturnToHand,
                Instruction::SelectGroup(1),
                Instruction::ModifyTargetPlayerCreatures {
                    power: 3,
                    toughness: 3,
                    keywords: Keywords::NONE,
                },
                Instruction::UntapPlayerCreatures(PlayerSet::Targeted),
            ],
        },
        SpellMode {
            text: "Destroy target creature or enchantment. Creatures target player controls get +3/+3 until end of turn. Untap them.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 33,
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
                Instruction::ModifyTargetPlayerCreatures {
                    power: 3,
                    toughness: 3,
                    keywords: Keywords::NONE,
                },
                Instruction::UntapPlayerCreatures(PlayerSet::Targeted),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
