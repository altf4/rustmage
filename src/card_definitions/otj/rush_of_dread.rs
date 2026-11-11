// Complete typed behavior from the pinned Oracle and XMage implementation.
const OPPONENT: TargetSpec = TargetSpec::one(TargetRule::Player {
    opponent_only: true,
});
const ALL: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const SACRIFICE: Instruction = Instruction::ChooseHalfRoundedUp {
    players: PlayerSet::Targeted,
    kind: EffectChoiceKind::Sacrifice,
    filter: SacrificeFilter { kinds: 1, ..ALL },
};
const DISCARD: Instruction = Instruction::ChooseHalfRoundedUp {
    players: PlayerSet::Targeted,
    kind: EffectChoiceKind::Discard,
    filter: ALL,
};
const LOSE_LIFE: Instruction = Instruction::LoseHalfLifeRoundedUp(PlayerSet::Targeted);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Sacrifice half the creatures",
            targets: TargetPlan::Single(OPPONENT),
            program: &[SACRIFICE],
        },
        SpellMode {
            text: "Discard half the hand",
            targets: TargetPlan::Single(OPPONENT),
            program: &[DISCARD],
        },
        SpellMode {
            text: "Sacrifice half the creatures and discard half the hand",
            targets: TargetPlan::Groups(&[OPPONENT, OPPONENT]),
            program: &[SACRIFICE, Instruction::SelectGroup(1), DISCARD],
        },
        SpellMode {
            text: "Lose half the life total",
            targets: TargetPlan::Single(OPPONENT),
            program: &[LOSE_LIFE],
        },
        SpellMode {
            text: "Sacrifice half the creatures and lose half the life total",
            targets: TargetPlan::Groups(&[OPPONENT, OPPONENT]),
            program: &[SACRIFICE, Instruction::SelectGroup(1), LOSE_LIFE],
        },
        SpellMode {
            text: "Discard half the hand and lose half the life total",
            targets: TargetPlan::Groups(&[OPPONENT, OPPONENT]),
            program: &[DISCARD, Instruction::SelectGroup(1), LOSE_LIFE],
        },
        SpellMode {
            text: "Sacrifice half the creatures, discard half the hand, and lose half the life total",
            targets: TargetPlan::Groups(&[OPPONENT, OPPONENT, OPPONENT]),
            program: &[
                SACRIFICE,
                Instruction::SelectGroup(1),
                DISCARD,
                Instruction::SelectGroup(2),
                LOSE_LIFE,
            ],
        },
    ]),
    mode_costs: Some(&[
        generic(1),
        generic(2),
        generic(3),
        generic(2),
        generic(3),
        generic(4),
        generic(5),
    ]),
    mode_counts: Some(&[1, 1, 2, 1, 2, 2, 3]),
    ..CardRules::EMPTY
};
