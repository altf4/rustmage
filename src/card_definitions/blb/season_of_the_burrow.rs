const RABBIT: Instruction = Instruction::CreateTokens {
    token: Card(5285),
    amount: 1,
    tapped: false,
};
const DRAW: Instruction = Instruction::Draw(PlayerSet::TargetController, 1);
const RESTORE: Instruction =
    Instruction::ReturnToBattlefieldWithCounters(&[(CounterKind::Indestructible, 1)]);
const NONLAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const PERMANENT_CARD: TargetSpec = TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: 3,
    },
    predicate: &TargetPredicate::Permanent,
});

// The printed costs are one, two, and three pawprints, with a budget of five.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Choose no modes",
            targets: TargetPlan::NONE,
            program: &[],
        },
        SpellMode {
            text: "Create 1 Rabbit",
            targets: TargetPlan::NONE,
            program: &[RABBIT],
        },
        SpellMode {
            text: "Create 2 Rabbits",
            targets: TargetPlan::NONE,
            program: &[RABBIT, RABBIT],
        },
        SpellMode {
            text: "Create 3 Rabbits",
            targets: TargetPlan::NONE,
            program: &[RABBIT, RABBIT, RABBIT],
        },
        SpellMode {
            text: "Create 4 Rabbits",
            targets: TargetPlan::NONE,
            program: &[RABBIT, RABBIT, RABBIT, RABBIT],
        },
        SpellMode {
            text: "Create 5 Rabbits",
            targets: TargetPlan::NONE,
            program: &[RABBIT, RABBIT, RABBIT, RABBIT, RABBIT],
        },
        SpellMode {
            text: "Exile a nonland permanent",
            targets: TargetPlan::Single(NONLAND),
            program: &[Instruction::Exile, DRAW],
        },
        SpellMode {
            text: "Create 1 Rabbit, then Exile a nonland permanent",
            targets: TargetPlan::Single(NONLAND),
            program: &[RABBIT, Instruction::Exile, DRAW],
        },
        SpellMode {
            text: "Create 2 Rabbits, then Exile a nonland permanent",
            targets: TargetPlan::Single(NONLAND),
            program: &[RABBIT, RABBIT, Instruction::Exile, DRAW],
        },
        SpellMode {
            text: "Create 3 Rabbits, then Exile a nonland permanent",
            targets: TargetPlan::Single(NONLAND),
            program: &[RABBIT, RABBIT, RABBIT, Instruction::Exile, DRAW],
        },
        SpellMode {
            text: "Exile two nonland permanents",
            targets: TargetPlan::Groups(&[NONLAND, NONLAND]),
            program: &[
                Instruction::Exile,
                DRAW,
                Instruction::SelectGroup(1),
                Instruction::Exile,
                DRAW,
            ],
        },
        SpellMode {
            text: "Create 1 Rabbit, then Exile two nonland permanents",
            targets: TargetPlan::Groups(&[NONLAND, NONLAND]),
            program: &[
                RABBIT,
                Instruction::Exile,
                DRAW,
                Instruction::SelectGroup(1),
                Instruction::Exile,
                DRAW,
            ],
        },
        SpellMode {
            text: "Return a permanent card with an indestructible counter",
            targets: TargetPlan::Single(PERMANENT_CARD),
            program: &[RESTORE],
        },
        SpellMode {
            text: "Create 1 Rabbit, then Return a permanent card with an indestructible counter",
            targets: TargetPlan::Single(PERMANENT_CARD),
            program: &[RABBIT, RESTORE],
        },
        SpellMode {
            text: "Create 2 Rabbits, then Return a permanent card with an indestructible counter",
            targets: TargetPlan::Single(PERMANENT_CARD),
            program: &[RABBIT, RABBIT, RESTORE],
        },
        SpellMode {
            text: "Exile a nonland permanent, then Return a permanent card with an indestructible counter",
            targets: TargetPlan::Groups(&[NONLAND, PERMANENT_CARD]),
            program: &[
                Instruction::Exile,
                DRAW,
                Instruction::SelectGroup(1),
                RESTORE,
            ],
        },
    ]),
    mode_counts: Some(&[0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 2, 3, 1, 2, 3, 2]),
    ..CardRules::EMPTY
};
