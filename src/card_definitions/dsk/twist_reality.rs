pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Counter target spell.",
            targets: one(TargetRule::Spell),
            program: &[Instruction::Counter],
        },
        SpellMode {
            text: "Manifest dread.",
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDread],
        },
    ]),
    ..CardRules::EMPTY
};
