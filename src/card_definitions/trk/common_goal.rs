// Pinned Oracle and complete XMage class: one or two controlled creature sources
// deal their current power as simultaneous damage to an opposing creature.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 1,
                max: 2,
            },
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[Instruction::GroupPowerDamage {
            source: 0,
            recipient: 1,
        }],
    ),
    ..CardRules::EMPTY
};
