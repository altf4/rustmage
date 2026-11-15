// Pinned XMage: modal impulse play through your next turn or universal damage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Come in Peace — Exile the top two cards of your library. Until the end of your next turn, you may play those cards.",
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlay {
                amount: 2,
                next_turn: true,
            }],
        },
        SpellMode {
            text: "Shoot to Kill — Encounter the Unknown deals 2 damage to each creature and each player.",
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                players: Some(PlayerSet::Each),
                amount: 2,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
