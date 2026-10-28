// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "You lose 2 life. Destroy target creature or planeswalker.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 65,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::LoseLife(PlayerSet::Controller, 2),
                Instruction::Destroy,
            ],
        },
        SpellMode {
            text: "You lose 2 life. Empower Jace 6.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Controller, 2),
                Instruction::EnsureJaceToken,
                Instruction::EmpowerJace(6),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
