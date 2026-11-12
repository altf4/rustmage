// Directly authored from the paired Oracle and pinned XMage sources.
const GRIMOIRE_RULES: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::NoMaximumHandSize,
        StaticAbilityFragment::CantLoseFromNonpositiveLife,
    ],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersIfCastByController,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 5)],
        },
        TriggeredAbility {
            condition: TriggerCondition::LifeGained {
                first_only: false,
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::LifeEventAmount,
            )],
        },
        TriggeredAbility {
            condition: TriggerCondition::LifeLost {
                relation: CastRelation::You,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DiscardScaled(PlayerSet::Controller, AmountValue::LifeEventAmount),
                Instruction::If {
                    guard: StepGuard::Threshold {
                        value: StepValue::Hand,
                        minimum: None,
                        maximum: Some(0),
                    },
                    skip: 1,
                },
                Instruction::LoseGame(PlayerSet::Controller),
            ],
        },
    ],
    static_groups: GRIMOIRE_RULES,
    extras: &CardExtras {
        static_ability_groups: GRIMOIRE_RULES,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
