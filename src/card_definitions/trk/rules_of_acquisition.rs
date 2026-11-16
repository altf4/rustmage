// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Follow the 3rd Rule",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::GainControl {
                    subject: Subject::Targets,
                    controller: PlayerSet::Controller,
                },
                Instruction::Draw(PlayerSet::Opponent, 3),
            ],
        },
        SpellMode {
            text: "Follow the 98th Rule",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: true,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::GainControlRememberCount {
                    subject: Subject::Targets,
                    controller: PlayerSet::Targeted,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::MovedCount,
                    minimum: 1,
                    skip: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 3),
            ],
        },
    ]),
    ..CardRules::EMPTY
};
