// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode { text: "Target opponent reveals their hand. You choose a nonland permanent card from it. That player discards that card.", targets: one(TargetRule::Player { opponent_only: true }), program: &[Instruction::RevealHandDiscard { excluded_kinds: 2 | 4 | 8 }] },
        SpellMode { text: "Target creature gets -2/-2 until end of turn.", targets: one(TargetRule::Creature), program: &[Instruction::Grow { subject: Subject::Targets, power: -2, toughness: -2 }] },
    ]),
    ..CardRules::EMPTY
};
