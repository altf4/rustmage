// Complete printed Oracle obligations, reviewed against pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CounterPermanentExileCast],
    ),
    ..CardRules::EMPTY
};
