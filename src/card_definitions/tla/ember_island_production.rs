// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Copy a creature you control as a nonlegendary 4/4 Hero",
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((4, 4)),
                    add_hero_subtype: true,
                    remove_legendary: true,
                    ..CopyExceptions::NONE
                },
            }],
        },
        SpellMode {
            text: "Copy a creature an opponent controls as a nonlegendary 2/2 Coward",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: CopyExceptions {
                    base: Some((2, 2)),
                    add_coward_subtype: true,
                    remove_legendary: true,
                    ..CopyExceptions::NONE
                },
            }],
        },
    ]),
    ..CardRules::EMPTY
};
