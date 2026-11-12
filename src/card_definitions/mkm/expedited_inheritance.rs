pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDamaged {
            relation: CastRelation::Any,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayForPlayer {
                player: PlayerSet::EventObjectController,
                skip: 1,
            },
            Instruction::ExileTopPlayForPlayer {
                player: PlayerSet::EventObjectController,
                amount: AmountValue::DamageReceivedAmount,
                next_turn: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};
