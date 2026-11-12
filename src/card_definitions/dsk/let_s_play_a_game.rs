const DELIRIUM: StatValue = StatValue::AtLeast {
    value: &StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
    minimum: 4,
};
const SHRINK: Instruction = Instruction::GrowCreatures {
    filter: CreatureFilter {
        relation: 1,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    power: -1,
    toughness: -1,
};
const DISCARD: Instruction = Instruction::Discard(PlayerSet::Opponent, 2);
const LOSE: Instruction = Instruction::LoseLife(PlayerSet::Opponent, 3);
const GAIN: Instruction = Instruction::GainLife(PlayerSet::Controller, 3);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Opposing creatures get -1/-1",
            targets: TargetPlan::NONE,
            program: &[SHRINK],
        },
        SpellMode {
            text: "Each opponent discards two cards",
            targets: TargetPlan::NONE,
            program: &[DISCARD],
        },
        SpellMode {
            text: "Each opponent loses 3 life and you gain 3 life",
            targets: TargetPlan::NONE,
            program: &[LOSE, GAIN],
        },
        SpellMode {
            text: "Shrink and discard",
            targets: TargetPlan::NONE,
            program: &[SHRINK, DISCARD],
        },
        SpellMode {
            text: "Shrink and change life totals",
            targets: TargetPlan::NONE,
            program: &[SHRINK, LOSE, GAIN],
        },
        SpellMode {
            text: "Discard and change life totals",
            targets: TargetPlan::NONE,
            program: &[DISCARD, LOSE, GAIN],
        },
        SpellMode {
            text: "Shrink, discard, and change life totals",
            targets: TargetPlan::NONE,
            program: &[SHRINK, DISCARD, LOSE, GAIN],
        },
    ]),
    mode_counts: Some(&[1, 1, 1, 2, 2, 2, 3]),
    extras: &CardExtras {
        cast_mode_requirements: &[(3, DELIRIUM), (4, DELIRIUM), (5, DELIRIUM), (6, DELIRIUM)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
