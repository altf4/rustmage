const RETURN: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const DISCARD: TargetSpec = TargetSpec::one(TargetRule::Player {
    opponent_only: true,
});
const LOOK: Instruction = Instruction::SelectLibraryTo(LibrarySelectionEffect {
    spec: LibrarySelectionSpec {
        look: 3,
        count: LibrarySelectionCount::Fixed(1),
        filter: LibrarySelectionFilter::Any,
        optional: false,
        reveal: false,
        remainder: LibraryRemainder::Graveyard,
    },
    look: LibraryLook::Fixed(3),
    destination: LibraryPickDestination::Hand,
    public: false,
    maximum_mana_value: None,
});
const DESCEND: StatValue = StatValue::AtLeast {
    value: &StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 1 | 2 | 16 | 32 | 64 | 256,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    minimum: 8,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Return a nonland permanent",
            targets: TargetPlan::Single(RETURN),
            program: &[Instruction::ReturnToHand],
        },
        SpellMode {
            text: "An opponent discards a card",
            targets: TargetPlan::Single(DISCARD),
            program: &[Instruction::Discard(PlayerSet::Targeted, 1)],
        },
        SpellMode {
            text: "Look at the top three and keep one",
            targets: TargetPlan::NONE,
            program: &[LOOK],
        },
        SpellMode {
            text: "Return and discard",
            targets: TargetPlan::Groups(&[RETURN, DISCARD]),
            program: &[
                Instruction::ReturnToHand,
                Instruction::SelectGroup(1),
                Instruction::Discard(PlayerSet::Targeted, 1),
            ],
        },
        SpellMode {
            text: "Return and look",
            targets: TargetPlan::Single(RETURN),
            program: &[Instruction::ReturnToHand, LOOK],
        },
        SpellMode {
            text: "Discard and look",
            targets: TargetPlan::Single(DISCARD),
            program: &[Instruction::Discard(PlayerSet::Targeted, 1), LOOK],
        },
        SpellMode {
            text: "Return, discard, and look",
            targets: TargetPlan::Groups(&[RETURN, DISCARD]),
            program: &[
                Instruction::ReturnToHand,
                Instruction::SelectGroup(1),
                Instruction::Discard(PlayerSet::Targeted, 1),
                LOOK,
            ],
        },
    ]),
    extras: &CardExtras {
        cast_mode_requirements: &[(3, DESCEND), (4, DESCEND), (5, DESCEND), (6, DESCEND)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
