// Whole-card typed Oracle behavior; full XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Copy your instant or sorcery spell.",
            targets: one(TargetRule::YourSpellKinds { kinds: 12 }),
            program: &[Instruction::CopyTargetSpell {
                group: 0,
                retarget: true,
                exceptions: CopyExceptions::NONE,
            }],
        },
        SpellMode {
            text: "Copy your creature spell with haste and an end-step sacrifice ability.",
            targets: one(TargetRule::YourSpellKinds { kinds: 1 }),
            program: &[Instruction::CopyTargetSpell {
                group: 0,
                retarget: false,
                exceptions: CopyExceptions::HASTE_AND_SACRIFICE,
            }],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::YourSpellKinds { kinds: 12 }),
                TargetSpec::one(TargetRule::YourSpellKinds { kinds: 1 }),
            ]),
            program: &[
                Instruction::CopyTargetSpell {
                    group: 0,
                    retarget: true,
                    exceptions: CopyExceptions::NONE,
                },
                Instruction::CopyTargetSpell {
                    group: 1,
                    retarget: false,
                    exceptions: CopyExceptions::HASTE_AND_SACRIFICE,
                },
            ],
        },
    ]),
    extras: &CardExtras {
        spell_cannot_be_copied: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
