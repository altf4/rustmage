// Complete typed behavior from the pinned Oracle and XMage implementation.
const OWN_GRAVEYARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CREATURE: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const fn graveyard_subtype(subtype: &'static TargetPredicate) -> TargetRule {
    TargetRule::Filtered {
        base: &OWN_GRAVEYARD,
        predicate: subtype,
    }
}
const TRANSPORT: TargetSpec = TargetSpec::one(TargetRule::Union(&[
    graveyard_subtype(&TargetPredicate::Subtype("Mount")),
    graveyard_subtype(&TargetPredicate::Subtype("Vehicle")),
]));
const ATTACHMENT: TargetSpec = TargetSpec::one(TargetRule::Union(&[
    graveyard_subtype(&TargetPredicate::Subtype("Aura")),
    graveyard_subtype(&TargetPredicate::Subtype("Equipment")),
]));
const RETURN: Instruction = Instruction::ReturnToBattlefield;
const ATTACH: Instruction = Instruction::ReturnTargetAttachmentToControlledCreature;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Return a creature",
            targets: TargetPlan::Single(CREATURE),
            program: &[RETURN],
        },
        SpellMode {
            text: "Return a Mount or Vehicle",
            targets: TargetPlan::Single(TRANSPORT),
            program: &[RETURN],
        },
        SpellMode {
            text: "Return a creature and a Mount or Vehicle",
            targets: TargetPlan::Groups(&[CREATURE, TRANSPORT]),
            program: &[RETURN, Instruction::SelectGroup(1), RETURN],
        },
        SpellMode {
            text: "Return an Aura or Equipment attached to a creature",
            targets: TargetPlan::Single(ATTACHMENT),
            program: &[ATTACH],
        },
        SpellMode {
            text: "Return a creature and an attachment",
            targets: TargetPlan::Groups(&[CREATURE, ATTACHMENT]),
            program: &[RETURN, Instruction::SelectGroup(1), ATTACH],
        },
        SpellMode {
            text: "Return a Mount or Vehicle and an attachment",
            targets: TargetPlan::Groups(&[TRANSPORT, ATTACHMENT]),
            program: &[RETURN, Instruction::SelectGroup(1), ATTACH],
        },
        SpellMode {
            text: "Return a creature, a Mount or Vehicle, and an attachment",
            targets: TargetPlan::Groups(&[CREATURE, TRANSPORT, ATTACHMENT]),
            program: &[
                RETURN,
                Instruction::SelectGroup(1),
                RETURN,
                Instruction::SelectGroup(2),
                ATTACH,
            ],
        },
    ]),
    mode_costs: Some(&[
        generic(2),
        generic(1),
        generic(3),
        generic(1),
        generic(3),
        generic(2),
        generic(4),
    ]),
    mode_counts: Some(&[1, 1, 2, 1, 2, 2, 3]),
    ..CardRules::EMPTY
};
