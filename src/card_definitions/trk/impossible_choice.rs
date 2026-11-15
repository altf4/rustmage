pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Surgical Separation — Destroy target creature. Its controller creates two Officers.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::DestroyAndCreateForController {
                token: Card::from_catalogue_id(6008),
                amount: 2,
            }],
        },
        SpellMode {
            text: "Remain Fused — Draw two cards and each player loses 2 life.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::LoseLife(PlayerSet::Each, 2),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
