// Oracle and pinned XMage reviewed. The blink returns the exact exiled card at the next end step.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Power to Engines — surveil two, then draw",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Surveil(2),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        SpellMode {
            text: "Power to Life Support — blink your creature",
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::BlinkUntilEndStep { under_owner: true }],
        },
    ]),
    ..CardRules::EMPTY
};
