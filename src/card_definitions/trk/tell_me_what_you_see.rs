// Oracle and official Star Trek release notes reviewed; XMage class absent at the pinned revision.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::MayPlayer {
                    player: PlayerSet::Targeted,
                    skip: 2,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::Skip(2),
                Instruction::LoseLife(PlayerSet::Targeted, 4),
                Instruction::Surveil(1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::FaceDilemma(CastRelation::You),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPlayer {
                    player: PlayerSet::Controller,
                    skip: 1,
                },
                Instruction::ReturnSourceToHand(TargetZone::Battlefield),
            ],
        },
    ],
    ..CardRules::EMPTY
};
