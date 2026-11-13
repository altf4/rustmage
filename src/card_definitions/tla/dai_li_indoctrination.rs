// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target opponent reveals their hand. You choose a nonland permanent card from it. That player discards that card.",
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::RevealHandDiscard {
                excluded_kinds: 2 | 4 | 8,
            }],
        },
        SpellMode {
            text: "Earthbend 2.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 2,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::AnimateEarthbendTargets,
                Instruction::RegisterEarthbendReturn,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 2,
                },
            ],
        },
    ]),
    ..CardRules::EMPTY
};
