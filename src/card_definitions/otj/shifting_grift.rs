const fn two_permanents(kinds: u8) -> TargetSpec {
    TargetSpec {
        rule: object(kinds),
        min: 2,
        max: 2,
    }
}

const CREATURES: TargetSpec = two_permanents(1);
const ARTIFACTS: TargetSpec = two_permanents(16);
const ENCHANTMENTS: TargetSpec = two_permanents(32);
const EXCHANGE_ONE: &[Instruction] = &[Instruction::ExchangeControlTargetGroup(0)];
const EXCHANGE_TWO: &[Instruction] = &[
    Instruction::ExchangeControlTargetGroup(0),
    Instruction::ExchangeControlTargetGroup(1),
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Exchange two creatures",
            targets: TargetPlan::Single(CREATURES),
            program: EXCHANGE_ONE,
        },
        SpellMode {
            text: "Exchange two artifacts",
            targets: TargetPlan::Single(ARTIFACTS),
            program: EXCHANGE_ONE,
        },
        SpellMode {
            text: "Exchange two enchantments",
            targets: TargetPlan::Single(ENCHANTMENTS),
            program: EXCHANGE_ONE,
        },
        SpellMode {
            text: "Exchange creatures and artifacts",
            targets: TargetPlan::Groups(&[CREATURES, ARTIFACTS]),
            program: EXCHANGE_TWO,
        },
        SpellMode {
            text: "Exchange creatures and enchantments",
            targets: TargetPlan::Groups(&[CREATURES, ENCHANTMENTS]),
            program: EXCHANGE_TWO,
        },
        SpellMode {
            text: "Exchange artifacts and enchantments",
            targets: TargetPlan::Groups(&[ARTIFACTS, ENCHANTMENTS]),
            program: EXCHANGE_TWO,
        },
        SpellMode {
            text: "Exchange creatures, artifacts, and enchantments",
            targets: TargetPlan::Groups(&[CREATURES, ARTIFACTS, ENCHANTMENTS]),
            program: &[
                Instruction::ExchangeControlTargetGroup(0),
                Instruction::ExchangeControlTargetGroup(1),
                Instruction::ExchangeControlTargetGroup(2),
            ],
        },
    ]),
    mode_counts: Some(&[1, 1, 1, 2, 2, 2, 3]),
    mode_costs: Some(&[
        generic(2),
        generic(1),
        generic(1),
        generic(3),
        generic(3),
        generic(2),
        generic(4),
    ]),
    ..CardRules::EMPTY
};
