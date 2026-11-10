// Adventure spell of Heartflame Duelist, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::Damage(3)],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1979)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
