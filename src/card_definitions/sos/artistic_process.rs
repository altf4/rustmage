// Direct typed Oracle behavior; full XMage source pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Artistic Process deals 6 damage to target creature.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::Damage(6)],
        },
        SpellMode {
            text: "Artistic Process deals 2 damage to each creature you don't control.",
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                players: None,
                amount: 2,
            }],
        },
        SpellMode {
            text: "Create a 3/3 blue and red Elemental creature token with flying. It gains haste until end of turn.",
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensWithTemporaryKeywords {
                token: Card(5349),
                amount: AmountValue::Fixed(1),
                keywords: Keywords::HASTE,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
