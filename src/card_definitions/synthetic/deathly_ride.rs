// Adventure spell of Fell Horseman, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToHand],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1490)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
