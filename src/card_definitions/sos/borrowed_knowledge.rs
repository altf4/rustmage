// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Discard your hand, then draw cards equal to the number of cards in target opponent's hand.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program: &[
                Instruction::DiscardHand(PlayerSet::Controller),
                Instruction::DrawScaled(PlayerSet::Controller, AmountValue::TargetPlayerHandSize),
            ],
        },
        SpellMode {
            text: "Discard your hand, then draw cards equal to the number of cards discarded this way.",
            targets: TargetPlan::NONE,
            program: &[Instruction::DiscardAnyDraw {
                players: PlayerSet::Controller,
                bonus: 0,
                required_all: true,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
