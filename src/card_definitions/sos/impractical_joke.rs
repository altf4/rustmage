// Whole-card typed Oracle behavior; full XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 65,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        &[
            Instruction::DamageCannotBePreventedThisTurn,
            Instruction::Damage(3),
        ],
    ),
    ..CardRules::EMPTY
};
