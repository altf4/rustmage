pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::SpellKinds {
            kinds: 1 | 16,
            excluded: 0,
        }),
        &[Instruction::CounterTargetThenDiscover],
    ),
    ..CardRules::EMPTY
};
