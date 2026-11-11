const CREATE: Instruction = Instruction::CreateTokens {
    token: Card(5612),
    amount: 1,
    tapped: false,
};
const DESTROY: ManaCost = ManaCost {
    colored: [0, 0, 1, 0, 0, 0],
    generic: 2,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy a creature",
            targets: one(TargetRule::Creature),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Create a Mercenary",
            targets: TargetPlan::NONE,
            program: &[CREATE],
        },
        SpellMode {
            text: "Destroy a creature and create a Mercenary",
            targets: one(TargetRule::Creature),
            program: &[Instruction::Destroy, CREATE],
        },
    ]),
    mode_counts: Some(&[1, 1, 2]),
    mode_costs: Some(&[
        DESTROY,
        generic(1),
        ManaCost {
            generic: 3,
            ..DESTROY
        },
    ]),
    ..CardRules::EMPTY
};
