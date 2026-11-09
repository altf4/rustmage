const YOUR_CREATURE: TargetSpec = TargetSpec::one(TargetRule::creature_you_control());
const OPPOSING_CREATURE: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const YOUR_SAGA: TargetSpec = TargetSpec::one(TargetRule::Filtered {
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
    predicate: &TargetPredicate::Subtype("Saga"),
});

const FIGHT: Instruction = Instruction::Fight {
    source: 0,
    recipient: 1,
};
const REMOVE_LORE: Instruction = Instruction::RemoveCounter {
    subject: Subject::Targets,
    kind: CounterKind::Lore,
};
const ADD_LORE: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::Lore,
    amount: 1,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target creature you control fights target creature an opponent controls.",
            targets: TargetPlan::Groups(&[YOUR_CREATURE, OPPOSING_CREATURE]),
            program: &[FIGHT],
        },
        SpellMode {
            text: "Remove a lore counter from target Saga you control.",
            targets: TargetPlan::Single(YOUR_SAGA),
            program: &[REMOVE_LORE],
        },
        SpellMode {
            text: "Put a lore counter on target Saga you control.",
            targets: TargetPlan::Single(YOUR_SAGA),
            program: &[ADD_LORE],
        },
        SpellMode {
            text: "Fight, then remove a lore counter.",
            targets: TargetPlan::Groups(&[YOUR_CREATURE, OPPOSING_CREATURE, YOUR_SAGA]),
            program: &[FIGHT, Instruction::SelectGroup(2), REMOVE_LORE],
        },
        SpellMode {
            text: "Fight, then put a lore counter.",
            targets: TargetPlan::Groups(&[YOUR_CREATURE, OPPOSING_CREATURE, YOUR_SAGA]),
            program: &[FIGHT, Instruction::SelectGroup(2), ADD_LORE],
        },
        SpellMode {
            text: "Remove a lore counter, then put a lore counter.",
            targets: TargetPlan::Groups(&[YOUR_SAGA, YOUR_SAGA]),
            program: &[REMOVE_LORE, Instruction::SelectGroup(1), ADD_LORE],
        },
        SpellMode {
            text: "Fight, remove a lore counter, then put a lore counter.",
            targets: TargetPlan::Groups(&[YOUR_CREATURE, OPPOSING_CREATURE, YOUR_SAGA, YOUR_SAGA]),
            program: &[
                FIGHT,
                Instruction::SelectGroup(2),
                REMOVE_LORE,
                Instruction::SelectGroup(3),
                ADD_LORE,
            ],
        },
    ]),
    ..CardRules::EMPTY
};
