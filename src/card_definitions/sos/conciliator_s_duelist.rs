// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::LoseLife(PlayerSet::Each, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Targeted {
                    filter: &CastFilter::InstantSorcery,
                    target: CastTarget::Creature(CastRelation::Any),
                },
                ordinal: None,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::BlinkUntilEndStep { under_owner: true }],
        },
    ],
    ..CardRules::EMPTY
};
