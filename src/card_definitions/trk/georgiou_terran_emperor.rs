// Pinned XMage: nonpositive-life protection is live while this permanent is
// present; the end-step token check follows the life loss and draw.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CantLoseFromNonpositiveLife],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Controller, 3),
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::ControllerLife,
                    minimum: None,
                    maximum: Some(0),
                },
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(6008),
                amount: 10,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
