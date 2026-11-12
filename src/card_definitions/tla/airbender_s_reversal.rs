// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target attacking creature.",
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            }),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Airbend target creature you control.",
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AirbendTargets],
        },
    ]),
    ..CardRules::EMPTY
};
