const SPACECRAFT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Spacecraft"),
};
const REDUCTION: TargetSpellReduction = TargetSpellReduction {
    rule: TargetRule::Filtered {
        base: &TargetRule::Creature,
        predicate: &TargetPredicate::Attacking,
    },
    discount: ManaCost {
        colored: [0; 6],
        generic: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::TargetCostReduction(0, REDUCTION)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[TargetRule::Creature, SPACECRAFT])),
        &[Instruction::OwnerLibrary],
    ),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        target_cost_reduction: Some(&REDUCTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
