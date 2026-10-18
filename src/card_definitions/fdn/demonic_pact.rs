pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "This enchantment deals 4 damage to any target and you gain 4 life.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
                program: &[
                    Instruction::Damage(4),
                    Instruction::GainLife(PlayerSet::Controller, 4),
                ],
            },
            SpellMode {
                text: "Target opponent discards two cards.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                    opponent_only: true,
                })),
                program: &[Instruction::Discard(PlayerSet::Targeted, 2)],
            },
            SpellMode {
                text: "Draw two cards.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::Draw(PlayerSet::Controller, 2)],
            },
            SpellMode {
                text: "You lose the game.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::LoseGame(PlayerSet::Controller)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
