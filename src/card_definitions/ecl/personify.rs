// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::BlinkImmediately {
                counter: None,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card(5493),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
