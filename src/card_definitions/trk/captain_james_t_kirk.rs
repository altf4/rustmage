// Oracle and pinned XMage reviewed. The mode ceiling is captured at trigger time.
const DISCARD_DRAW: &[Instruction] = &[
    Instruction::Discard(PlayerSet::Controller, 1),
    Instruction::Draw(PlayerSet::Controller, 1),
];
const OFFICER: Instruction = Instruction::CreateTokens {
    token: Card(6008),
    amount: 1,
    tapped: false,
};
const GROW: Instruction = Instruction::GrowPlayerCreatures {
    players: PlayerSet::Controller,
    power: 1,
    toughness: 0,
};
const MODES: &[SpellMode] = &[
    SpellMode {
        text: "Discard, then draw",
        targets: TargetPlan::NONE,
        program: DISCARD_DRAW,
    },
    SpellMode {
        text: "Create an Officer",
        targets: TargetPlan::NONE,
        program: &[OFFICER],
    },
    SpellMode {
        text: "Creatures get +1/+0",
        targets: TargetPlan::NONE,
        program: &[GROW],
    },
    SpellMode {
        text: "Discard, draw, and create an Officer",
        targets: TargetPlan::NONE,
        program: &[DISCARD_DRAW[0], DISCARD_DRAW[1], OFFICER],
    },
    SpellMode {
        text: "Discard, draw, and creatures get +1/+0",
        targets: TargetPlan::NONE,
        program: &[DISCARD_DRAW[0], DISCARD_DRAW[1], GROW],
    },
    SpellMode {
        text: "Create an Officer and creatures get +1/+0",
        targets: TargetPlan::NONE,
        program: &[OFFICER, GROW],
    },
    SpellMode {
        text: "Discard, draw, create an Officer, and creatures get +1/+0",
        targets: TargetPlan::NONE,
        program: &[DISCARD_DRAW[0], DISCARD_DRAW[1], OFFICER, GROW],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalUpToStat {
            modes: MODES,
            counts: &[1, 1, 1, 2, 2, 2, 3],
            maximum: StatValue::Sum(&[
                StatValue::Fixed(1),
                StatValue::Not(&StatValue::ControllerHandSize),
                StatValue::Not(&StatValue::ControllerHandSize),
            ]),
        }],
    }],
    ..CardRules::EMPTY
};
