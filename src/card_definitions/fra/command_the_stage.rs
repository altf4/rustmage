// Direct Oracle and pinned XMage behavior; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokenThenCounterOtherWizards { token: Card(5298) }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::Upkeep,
                    relation: CastRelation::Any,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::OpponentNoncombatDamageLastTurn,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceToHand(TargetZone::Graveyard)],
    }],
    ..CardRules::EMPTY
};
