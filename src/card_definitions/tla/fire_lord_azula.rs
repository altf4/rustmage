pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::Expression {
                    value: StatValue::SourceAttacking,
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Any,
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell {
                prior_spells: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
