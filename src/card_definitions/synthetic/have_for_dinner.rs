// Direct typed behavior from the pinned Oracle and XMage sources.
use rustmage_core::engine::replacements::TokenGroupView;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokenBatch(&[
            TokenGroupView {
                token: Card(5254),
                amount: 1,
                tapped: false,
            },
            TokenGroupView {
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
