// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DestroyAndCreateForController {
            token: Card(5315),
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
