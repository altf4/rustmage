// Adventure spell of Virtue of Loyalty, with its own pinned Oracle face.
const VIGILANT_KNIGHT: Card = Card(5259);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: VIGILANT_KNIGHT,
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(4907)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
