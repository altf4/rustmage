// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::ExileAndCreateForController {
            token: Card(5344),
            amount: 1,
        }],
    ),
    keywords: Keywords::CHANGELING,
    ..CardRules::EMPTY
};
