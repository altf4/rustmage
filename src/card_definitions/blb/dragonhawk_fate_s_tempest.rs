// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ExileTopLibraryRemember {
                    players: PlayerSet::Controller,
                    amount: AmountValue::ControlledCreaturePowerCount { minimum: 4 },
                },
                Instruction::ScheduleStepWithExiledResult {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    ability: 1,
                },
                Instruction::GrantExiledResultPlayUntilNextEndStep,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayersScaled(
                PlayerSet::Opponent,
                AmountValue::Multiple {
                    value: &AmountValue::DelayedExiledCardCount,
                    factor: 2,
                },
            )],
        },
    ],
    ..CardRules::EMPTY
};
