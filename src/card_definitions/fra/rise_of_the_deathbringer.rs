// Direct Oracle and pinned XMage rules; gated pending integration validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode { text: "Draw cards equal to the greatest power among creatures you control. You lose life equal to the number of cards drawn this way.",
            targets: TargetPlan::NONE, program: &[Instruction::DrawGreatestCreaturePowerRemember, Instruction::LoseLifeRememberedDraws] },
        SpellMode { text: "All creatures get -3/-3 until end of turn.", targets: TargetPlan::NONE,
            program: &[Instruction::GrowPlayerCreatures { players: PlayerSet::Each, power: -3, toughness: -3 }] },
    ]),
    ..CardRules::EMPTY
};
