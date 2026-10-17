pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::Single(TargetSpec::one(TargetRule::Creature)), &[Instruction::DoubleTargetPower]),
    extras: &CardExtras { flashback_cost: Some(ManaCost { colored: [0,0,0,2,0,0], generic: 4 }), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
