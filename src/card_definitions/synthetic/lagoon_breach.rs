// Adventure spell of Horned Loch-Whale, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Attacking,
        })),
        &[Instruction::OwnerLibrary],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(2073)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
