// Direct Oracle behavior; graveyard groups are counted after token creation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokensWithGraveyardGroups {
            token: Card(5298),
            amount: 2,
            group: std::num::NonZeroU16::new(3).unwrap(),
        }],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 1, 1, 0, 0, 0],
            generic: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
