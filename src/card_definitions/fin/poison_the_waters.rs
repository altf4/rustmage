// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "All creatures get -1/-1 until end of turn.",
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowPlayerCreatures {
                players: PlayerSet::Each,
                power: -1,
                toughness: -1,
            }],
        },
        SpellMode {
            text: "Reveal target player's hand and choose an artifact or creature to discard.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            })),
            program: &[Instruction::RevealTargetHandChoose {
                kinds: 17,
                exile: false,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
