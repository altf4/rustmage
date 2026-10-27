const CREATURE: TargetSpec = TargetSpec::one(TargetRule::Creature);
const PERMANENT: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const NO_OVERRIDES: crate::engine::CopyOverrides = crate::engine::CopyOverrides {
    name: None,
    legendary: None,
    kinds: None,
    types: None,
    keywords: Keywords::NONE,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[CREATURE, PERMANENT, CREATURE]),
        &[
            Instruction::BecomeCopy(InPlaceCopySpec {
                recipient_group: Some(0),
                blueprint_group: 1,
                overrides: NO_OVERRIDES,
                add_kinds: 1,
                base: Some((2, 2)),
                next_turn: false,
            }),
            Instruction::BecomeCopy(InPlaceCopySpec {
                recipient_group: Some(0),
                blueprint_group: 2,
                overrides: NO_OVERRIDES,
                add_kinds: 0,
                base: None,
                next_turn: false,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
