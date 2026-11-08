// Direct typed behavior from full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ReturnBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                attacking: false,
                players: None,
            },
            Instruction::BeginOptionalShuffleWheel(7),
            Instruction::AskOptionalWheel {
                first: true,
                skip: 2,
            },
            Instruction::RecordOptionalWheel {
                first: true,
                accept: true,
            },
            Instruction::Skip(1),
            Instruction::RecordOptionalWheel {
                first: true,
                accept: false,
            },
            Instruction::AskOptionalWheel {
                first: false,
                skip: 2,
            },
            Instruction::RecordOptionalWheel {
                first: false,
                accept: true,
            },
            Instruction::Skip(1),
            Instruction::RecordOptionalWheel {
                first: false,
                accept: false,
            },
            Instruction::ExecuteOptionalWheel,
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
