// Complete typed behavior from the pinned Oracle and XMage implementation.
const COPY_TARGET: TargetSpec = TargetSpec::one(TargetRule::Union(&[
    TargetRule::SpellKinds {
        kinds: 4 | 8,
        excluded: 0,
    },
    TargetRule::Ability,
]));
const REDIRECT_TARGET: TargetSpec = TargetSpec::one(TargetRule::SingleTargetStackObject);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Copy an instant, sorcery, or ability",
            targets: TargetPlan::Single(COPY_TARGET),
            program: &[Instruction::CopyTargetSpellOrAbility { group: 0 }],
        },
        SpellMode {
            text: "Change the single target of a spell or ability",
            targets: TargetPlan::Single(REDIRECT_TARGET),
            program: &[Instruction::ChangeSingleStackTarget],
        },
        SpellMode {
            text: "Copy an instant, sorcery, or ability and change a single target",
            targets: TargetPlan::Groups(&[COPY_TARGET, REDIRECT_TARGET]),
            program: &[
                Instruction::CopyTargetSpellOrAbility { group: 0 },
                Instruction::SelectGroup(1),
                Instruction::ChangeSingleStackTarget,
            ],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(1), generic(2)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
