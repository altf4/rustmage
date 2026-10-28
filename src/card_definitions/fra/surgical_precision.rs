// Authored from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target creature with toughness 4 or greater. You gain 1 life.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::ToughnessAtLeast(4),
            })),
            program: &[
                Instruction::Destroy,
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
        SpellMode {
            text: "You draw a card and gain 2 life.",
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::GainLife(PlayerSet::Controller, 2),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
