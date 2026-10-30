// Gated direct Oracle behavior; ward payer is captured when the target is acquired.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessDiscard],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureEnters {
                relation: CastRelation::Opponent,
                other: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::EventPlayer, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::OpponentLoyaltyAbilityActivated,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayers(PlayerSet::EventPlayer, 1)],
        },
    ],
    ..CardRules::EMPTY
};
