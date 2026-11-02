// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::CreatureOrVehicle {
            relation: 1,
        })),
        program: &[
            Instruction::ReturnToHand,
            Instruction::DecreaseLeadingSpeed {
                players: PlayerSet::TargetController,
                minimum: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
