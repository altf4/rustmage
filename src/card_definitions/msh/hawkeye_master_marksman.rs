// Direct typed finite mode combinations; payment precedes the separate reflexive trigger.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH.union(Keywords::FIRST_STRIKE),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[Instruction::PayXReflexiveLimited {
                ability: 1,
                maximum: 3,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::ModalUpToX {
                modes: &[
                    SpellMode {
                        text: "Choose no modes",
                        targets: TargetPlan::NONE,
                        program: &[],
                    },
                    SpellMode {
                        text: "Net",
                        targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Creature)]),
                        program: &[
                            Instruction::SelectGroup(0),
                            Instruction::GrantKeywords {
                                subject: Subject::Targets,
                                keywords: Keywords::CANT_BLOCK,
                            },
                        ],
                    },
                    SpellMode {
                        text: "Explosive",
                        targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Player {
                            opponent_only: false,
                        })]),
                        program: &[Instruction::SelectGroup(0), Instruction::Damage(2)],
                    },
                    SpellMode {
                        text: "Net + Explosive",
                        targets: TargetPlan::Groups(&[
                            TargetSpec::one(TargetRule::Creature),
                            TargetSpec::one(TargetRule::Player {
                                opponent_only: false,
                            }),
                        ]),
                        program: &[
                            Instruction::SelectGroup(0),
                            Instruction::GrantKeywords {
                                subject: Subject::Targets,
                                keywords: Keywords::CANT_BLOCK,
                            },
                            Instruction::SelectGroup(1),
                            Instruction::Damage(2),
                        ],
                    },
                    SpellMode {
                        text: "Boomerang",
                        targets: TargetPlan::NONE,
                        program: &[
                            Instruction::Discard(PlayerSet::Controller, 1),
                            Instruction::Draw(PlayerSet::Controller, 1),
                        ],
                    },
                    SpellMode {
                        text: "Net + Boomerang",
                        targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Creature)]),
                        program: &[
                            Instruction::SelectGroup(0),
                            Instruction::GrantKeywords {
                                subject: Subject::Targets,
                                keywords: Keywords::CANT_BLOCK,
                            },
                            Instruction::Discard(PlayerSet::Controller, 1),
                            Instruction::Draw(PlayerSet::Controller, 1),
                        ],
                    },
                    SpellMode {
                        text: "Explosive + Boomerang",
                        targets: TargetPlan::Groups(&[TargetSpec::one(TargetRule::Player {
                            opponent_only: false,
                        })]),
                        program: &[
                            Instruction::SelectGroup(0),
                            Instruction::Damage(2),
                            Instruction::Discard(PlayerSet::Controller, 1),
                            Instruction::Draw(PlayerSet::Controller, 1),
                        ],
                    },
                    SpellMode {
                        text: "Net + Explosive + Boomerang",
                        targets: TargetPlan::Groups(&[
                            TargetSpec::one(TargetRule::Creature),
                            TargetSpec::one(TargetRule::Player {
                                opponent_only: false,
                            }),
                        ]),
                        program: &[
                            Instruction::SelectGroup(0),
                            Instruction::GrantKeywords {
                                subject: Subject::Targets,
                                keywords: Keywords::CANT_BLOCK,
                            },
                            Instruction::SelectGroup(1),
                            Instruction::Damage(2),
                            Instruction::Discard(PlayerSet::Controller, 1),
                            Instruction::Draw(PlayerSet::Controller, 1),
                        ],
                    },
                ],
                counts: &[0, 1, 1, 2, 1, 2, 2, 3],
            }],
        },
    ],
    ..CardRules::EMPTY
};
