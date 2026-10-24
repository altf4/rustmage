// Direct typed multiplicities of the two printed modes, in printed resolution order.
const DESTROY: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const RETURN: TargetRule = TargetRule::Filtered {
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
    predicate: &TargetPredicate::WithoutKinds(12),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Choose no modes.",
            targets: TargetPlan::NONE,
            program: &[],
        },
        SpellMode {
            text: "Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[TargetSpec::one(RETURN)]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[TargetSpec::one(RETURN), TargetSpec::one(RETURN)]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(1),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(1),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(2),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(1),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(2),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(3),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent.",
            targets: TargetPlan::Groups(&[TargetSpec::one(DESTROY)]),
            program: &[Instruction::SelectGroup(0), Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[TargetSpec::one(DESTROY), TargetSpec::one(RETURN)]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(2),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(2),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(3),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Destroy target nonland permanent.",
            targets: TargetPlan::Groups(&[TargetSpec::one(DESTROY), TargetSpec::one(DESTROY)]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Destroy target nonland permanent. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
                Instruction::SelectGroup(2),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Destroy target nonland permanent. Return target nonland permanent card from your graveyard to the battlefield. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(RETURN),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
                Instruction::SelectGroup(2),
                Instruction::ReturnToBattlefield,
                Instruction::SelectGroup(3),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Destroy target nonland permanent. Destroy target nonland permanent.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
                Instruction::SelectGroup(2),
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Destroy target nonland permanent. Destroy target nonland permanent. Return target nonland permanent card from your graveyard to the battlefield.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(RETURN),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
                Instruction::SelectGroup(2),
                Instruction::Destroy,
                Instruction::SelectGroup(3),
                Instruction::ReturnToBattlefield,
            ],
        },
        SpellMode {
            text: "Destroy target nonland permanent. Destroy target nonland permanent. Destroy target nonland permanent. Destroy target nonland permanent.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
                TargetSpec::one(DESTROY),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
                Instruction::SelectGroup(2),
                Instruction::Destroy,
                Instruction::SelectGroup(3),
                Instruction::Destroy,
            ],
        },
    ]),
    ..CardRules::EMPTY
};
