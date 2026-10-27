// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DamageBattlefieldExceptSubtype {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                excluded_subtype: "Dragon",
                players: None,
                amount: 3,
            },
            Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Combination {
                    mask: 31,
                    amount: 4,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::SpellSubtype("Dragon"),
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            }),
        ],
    ),
    ..CardRules::EMPTY
};
