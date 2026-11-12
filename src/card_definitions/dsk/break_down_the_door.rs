// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exile target artifact.",
            targets: one(object(16)),
            program: &[Instruction::Exile],
        },
        SpellMode {
            text: "Exile target enchantment.",
            targets: one(object(32)),
            program: &[Instruction::Exile],
        },
        SpellMode {
            text: "Manifest dread.",
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDread],
        },
    ]),
    ..CardRules::EMPTY
};
