// Complete Oracle/XMage: public unrestricted name, bounded private three-zone search.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[
            Instruction::ChooseCardName { nonland: false },
            Instruction::ExileTargetNamedCards { maximum: 4 },
        ],
    ),
    ..CardRules::EMPTY
};
