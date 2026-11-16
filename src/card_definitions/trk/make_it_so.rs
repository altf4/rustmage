pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Untap target permanent you control. Target player draws two cards.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
            ]),
            program: &[
                Instruction::Untap,
                Instruction::SelectGroup(1),
                Instruction::Draw(PlayerSet::Targeted, 2),
            ],
        },
        SpellMode {
            text: "Create an Officer and draw two cards.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CreateTokens {
                    token: Card::from_catalogue_id(6008),
                    amount: 1,
                    tapped: false,
                },
                Instruction::Draw(PlayerSet::Controller, 2),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
