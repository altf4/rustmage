// Authored from complete pinned Oracle and XMage sources; gated pending validation.
const TARGET: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 65,
        relation: 1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Colors(9),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TARGET),
            TargetSpec::one(TargetRule::creature_you_control()),
        ]),
        &[
            Instruction::RemoveTargetAbilitiesUntilEndOfTurn,
            Instruction::PowerDamage {
                source: 1,
                recipient: 0,
            },
        ],
    ),
    ..CardRules::EMPTY
};
