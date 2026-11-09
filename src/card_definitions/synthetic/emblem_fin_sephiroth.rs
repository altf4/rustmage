// Direct typed behavior from the pinned SephirothOneWingedAngelEmblem.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::Any,
            other: false,
            nontoken: false,
            subtype: None,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::LoseLife(PlayerSet::Targeted, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    deck_limit: None,
    ..CardRules::EMPTY
};
