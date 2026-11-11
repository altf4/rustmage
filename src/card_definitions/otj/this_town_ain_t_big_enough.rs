const fn nonland(relation: i8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 0,
        relation,
        other: false,
        nonland: true,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: nonland(-1),
            min: 0,
            max: 2,
        }),
        &[Instruction::ReturnToHand],
    ),
    extras: &CardExtras {
        target_cost_reduction: Some(&TargetSpellReduction {
            rule: nonland(0),
            discount: generic(3),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
