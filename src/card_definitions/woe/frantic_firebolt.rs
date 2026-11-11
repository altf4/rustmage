// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::DamageScaled(AmountValue::Sum(&[
            AmountValue::Fixed(2),
            AmountValue::Stat(StatValue::GraveyardInstantSorceryAdventure),
        ]))],
    ),
    ..CardRules::EMPTY
};
