// Adventure spell of Virtue of Courage, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Any), &[Instruction::Damage(2)]),
    extras: &CardExtras {
        adventure_parent: Some(Card(4905)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
