// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokenBatch(&[
            engine::replacements::TokenGroupView {
                token: Card(5254),
                amount: 1,
                tapped: false,
            },
            engine::replacements::TokenGroupView {
                token: Card(5246),
                amount: 1,
                tapped: false,
            },
        ])],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1065)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
