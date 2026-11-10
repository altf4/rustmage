// Adventure spell of Virtue of Knowledge, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::YourAbility),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(4906)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
