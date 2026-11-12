const OPPONENT: TargetPlan = one(TargetRule::Player {
    opponent_only: true,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target opponent discards two cards.",
            targets: OPPONENT,
            program: &[Instruction::Discard(PlayerSet::Targeted, 2)],
        },
        SpellMode {
            text: "Reveal the target opponent's hand and choose a nonland card for them to discard.",
            targets: OPPONENT,
            program: &[Instruction::RevealHandDiscard { excluded_kinds: 2 }],
        },
    ]),
    ..CardRules::EMPTY
};
