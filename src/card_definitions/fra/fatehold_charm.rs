// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Draw a card. Empower Jace 2.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::EnsureJaceToken,
                Instruction::EmpowerJace(2),
            ],
        },
        SpellMode {
            text: "Return target spell or creature to its owner's hand.",
            targets: one(TargetRule::Union(&[
                TargetRule::Spell,
                TargetRule::Creature,
            ])),
            program: &[Instruction::ReturnSpellToHand, Instruction::ReturnToHand],
        },
        SpellMode {
            text: "Creatures you control get +1/+2 until end of turn.",
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowPlayerCreatures {
                players: PlayerSet::Controller,
                power: 1,
                toughness: 2,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
