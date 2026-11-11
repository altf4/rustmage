// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CaptureSearchResult,
            Instruction::SearchRequired(SearchSpec {
                filter: SearchFilter::Any,
                maximum: 1,
                destination: SearchDestination::ExileFaceDown,
                tapped: false,
                reveal: false,
            }),
            Instruction::IfBargained { skip: 1 },
            Instruction::CastExiledSearchResultFreeUpTo(4),
            Instruction::PutExiledSearchResultInHand,
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
