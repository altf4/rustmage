// Captured cast X bounds the entry trigger independently of counters or source presence.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        x_symbols: 1,
        x_entry_counters: true,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalUpToX {
            modes: &[
                SpellMode {
                    text: "Choose no modes",
                    targets: TargetPlan::NONE,
                    program: &[],
                },
                SpellMode {
                    text: "Rummage",
                    targets: TargetPlan::NONE,
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                    ],
                },
                SpellMode {
                    text: "Lose 2 life",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Player {
                        opponent_only: true,
                    })]),
                    program: &[
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                    ],
                },
                SpellMode {
                    text: "Rummage + Lose 2 life",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Player {
                        opponent_only: true,
                    })]),
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                    ],
                },
                SpellMode {
                    text: "Destroy token",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Token,
                    })]),
                    program: &[Instruction::SelectGroup(0), Instruction::Destroy],
                },
                SpellMode {
                    text: "Rummage + Destroy token",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Token,
                    })]),
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::SelectGroup(0),
                        Instruction::Destroy,
                    ],
                },
                SpellMode {
                    text: "Lose 2 life + Destroy token",
                    targets: TargetPlan::Groups(&[
                        TargetSpec::one(TargetRule::Player {
                            opponent_only: true,
                        }),
                        TargetSpec::one(TargetRule::Filtered {
                            base: &TargetRule::Object {
                                zone: TargetZone::Battlefield,
                                kinds: 0,
                                relation: -1,
                                other: false,
                                nonland: false,
                                tapped: false,
                                min_mana: 0,
                                max_mana: u32::MAX,
                            },
                            predicate: &TargetPredicate::Token,
                        }),
                    ]),
                    program: &[
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                        Instruction::SelectGroup(1),
                        Instruction::Destroy,
                    ],
                },
                SpellMode {
                    text: "Rummage + Lose 2 life + Destroy token",
                    targets: TargetPlan::Groups(&[
                        TargetSpec::one(TargetRule::Player {
                            opponent_only: true,
                        }),
                        TargetSpec::one(TargetRule::Filtered {
                            base: &TargetRule::Object {
                                zone: TargetZone::Battlefield,
                                kinds: 0,
                                relation: -1,
                                other: false,
                                nonland: false,
                                tapped: false,
                                min_mana: 0,
                                max_mana: u32::MAX,
                            },
                            predicate: &TargetPredicate::Token,
                        }),
                    ]),
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                        Instruction::SelectGroup(1),
                        Instruction::Destroy,
                    ],
                },
                SpellMode {
                    text: "Each player sacrifices",
                    targets: TargetPlan::NONE,
                    program: &[Instruction::ChooseObjects {
                        kind: EffectChoiceKind::Sacrifice,
                        players: PlayerSet::Each,
                        filter: SacrificeFilter {
                            kinds: 1,
                            keywords: Keywords::NONE,
                            source_only: false,
                            nonland: false,
                            other: false,
                            tokens: None,
                            artifact_enchantment_or_token: false,
                            subtype: None,
                        },
                        amount: 1,
                        optional_skip: None,
                    }],
                },
                SpellMode {
                    text: "Rummage + Each player sacrifices",
                    targets: TargetPlan::NONE,
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
                SpellMode {
                    text: "Lose 2 life + Each player sacrifices",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Player {
                        opponent_only: true,
                    })]),
                    program: &[
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
                SpellMode {
                    text: "Rummage + Lose 2 life + Each player sacrifices",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Player {
                        opponent_only: true,
                    })]),
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
                SpellMode {
                    text: "Destroy token + Each player sacrifices",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Token,
                    })]),
                    program: &[
                        Instruction::SelectGroup(0),
                        Instruction::Destroy,
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
                SpellMode {
                    text: "Rummage + Destroy token + Each player sacrifices",
                    targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Token,
                    })]),
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::SelectGroup(0),
                        Instruction::Destroy,
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
                SpellMode {
                    text: "Lose 2 life + Destroy token + Each player sacrifices",
                    targets: TargetPlan::Groups(&[
                        TargetSpec::one(TargetRule::Player {
                            opponent_only: true,
                        }),
                        TargetSpec::one(TargetRule::Filtered {
                            base: &TargetRule::Object {
                                zone: TargetZone::Battlefield,
                                kinds: 0,
                                relation: -1,
                                other: false,
                                nonland: false,
                                tapped: false,
                                min_mana: 0,
                                max_mana: u32::MAX,
                            },
                            predicate: &TargetPredicate::Token,
                        }),
                    ]),
                    program: &[
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                        Instruction::SelectGroup(1),
                        Instruction::Destroy,
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
                SpellMode {
                    text: "Rummage + Lose 2 life + Destroy token + Each player sacrifices",
                    targets: TargetPlan::Groups(&[
                        TargetSpec::one(TargetRule::Player {
                            opponent_only: true,
                        }),
                        TargetSpec::one(TargetRule::Filtered {
                            base: &TargetRule::Object {
                                zone: TargetZone::Battlefield,
                                kinds: 0,
                                relation: -1,
                                other: false,
                                nonland: false,
                                tapped: false,
                                min_mana: 0,
                                max_mana: u32::MAX,
                            },
                            predicate: &TargetPredicate::Token,
                        }),
                    ]),
                    program: &[
                        Instruction::Discard(PlayerSet::Controller, 1),
                        Instruction::Draw(PlayerSet::Controller, 1),
                        Instruction::SelectGroup(0),
                        Instruction::LoseLife(PlayerSet::Targeted, 2),
                        Instruction::SelectGroup(1),
                        Instruction::Destroy,
                        Instruction::ChooseObjects {
                            kind: EffectChoiceKind::Sacrifice,
                            players: PlayerSet::Each,
                            filter: SacrificeFilter {
                                kinds: 1,
                                keywords: Keywords::NONE,
                                source_only: false,
                                nonland: false,
                                other: false,
                                tokens: None,
                                artifact_enchantment_or_token: false,
                                subtype: None,
                            },
                            amount: 1,
                            optional_skip: None,
                        },
                    ],
                },
            ],
            counts: &[0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4],
        }],
    }],
    ..CardRules::EMPTY
};
