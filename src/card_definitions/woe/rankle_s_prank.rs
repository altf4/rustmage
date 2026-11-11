// Direct typed behavior from the paired Oracle and pinned XMage sources.
const DISCARD: Instruction = Instruction::Discard(PlayerSet::Each, 2);
const LOSE_LIFE: Instruction = Instruction::LoseLife(PlayerSet::Each, 4);
const SACRIFICE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::Sacrifice,
    players: PlayerSet::Each,
    filter: SacrificeFilter {
        kinds: 1,
        other: false,
        keywords: Keywords::NONE,
        source_only: false,
        nonland: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 2,
    optional_skip: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Each player discards two cards.",
            targets: TargetPlan::NONE,
            program: &[DISCARD],
        },
        SpellMode {
            text: "Each player loses 4 life.",
            targets: TargetPlan::NONE,
            program: &[LOSE_LIFE],
        },
        SpellMode {
            text: "Each player discards two cards and loses 4 life.",
            targets: TargetPlan::NONE,
            program: &[DISCARD, LOSE_LIFE],
        },
        SpellMode {
            text: "Each player sacrifices two creatures of their choice.",
            targets: TargetPlan::NONE,
            program: &[SACRIFICE],
        },
        SpellMode {
            text: "Each player discards two cards and sacrifices two creatures of their choice.",
            targets: TargetPlan::NONE,
            program: &[DISCARD, SACRIFICE],
        },
        SpellMode {
            text: "Each player loses 4 life and sacrifices two creatures of their choice.",
            targets: TargetPlan::NONE,
            program: &[LOSE_LIFE, SACRIFICE],
        },
        SpellMode {
            text: "Each player discards two cards, loses 4 life, and sacrifices two creatures of their choice.",
            targets: TargetPlan::NONE,
            program: &[DISCARD, LOSE_LIFE, SACRIFICE],
        },
    ]),
    mode_counts: Some(&[1, 1, 2, 1, 2, 2, 3]),
    ..CardRules::EMPTY
};
