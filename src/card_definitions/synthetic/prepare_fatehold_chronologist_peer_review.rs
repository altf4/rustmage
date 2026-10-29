// Peer Review prepared by Fatehold Chronologist.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card(5298),
                amount: 1,
                tapped: false,
            },
            Instruction::Surveil(1),
        ],
    ),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
        }],
        prepared_identity: Some("@prepare/fatehold_chronologist_peer_review"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
