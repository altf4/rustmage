// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Modal(&[
            SpellMode {
                text: "Use the Relic — Target creature gets +3/+2 until end of turn.",
                targets: one(TargetRule::Creature),
                program: &[Instruction::Grow {
                    subject: Subject::Targets,
                    power: 3,
                    toughness: 2,
                }],
            },
            SpellMode {
                text: "Destroy the Relic — Destroy target artifact.",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 16,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                program: &[Instruction::Destroy],
            },
        ])],
    ),
    ..CardRules::EMPTY
};
