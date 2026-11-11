// Adventure spell of Gumdrop Poisoner, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card(5246),
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1908)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
