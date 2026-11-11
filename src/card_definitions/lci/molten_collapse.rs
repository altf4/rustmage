// The combined mode keeps both target occurrences and resolves them in printed order.
const CREATURE_OR_PLANESWALKER: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1 | 64,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const SMALL_NONCREATURE: TargetSpec = TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: -1,
        other: false,
        nonland: true,
        tapped: false,
        min_mana: 0,
        max_mana: 1,
    },
    predicate: &TargetPredicate::WithoutKinds(1),
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature or planeswalker",
            targets: TargetPlan::Single(CREATURE_OR_PLANESWALKER),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy target noncreature, nonland permanent with mana value 1 or less",
            targets: TargetPlan::Single(SMALL_NONCREATURE),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Choose both",
            targets: TargetPlan::Groups(&[CREATURE_OR_PLANESWALKER, SMALL_NONCREATURE]),
            program: &[
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
    ]),
    extras: &CardExtras {
        cast_mode_requirements: &[(2, StatValue::DescendedThisTurn)],
        ..CardExtras::EMPTY
    },
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
