// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::SpellKinds { kinds: 0, excluded: 1 }),
        &[Instruction::CounterAndCreateForController { token: Card(5302), amount: 2 }]),
    ..CardRules::EMPTY
};
