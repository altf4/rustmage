// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalOnce(&[
            SpellMode {
                text: "Draw a card.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::Draw(PlayerSet::Controller, 1),
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
            SpellMode {
                text: "Put a +1/+1 counter on Zuko.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::PutCounters {
                        subject: Subject::Source,
                        kind: CounterKind::PlusOne,
                        amount: 1,
                    },
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
            SpellMode {
                text: "Add {R}.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 1, 0, 0])),
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
            SpellMode {
                text: "Exile Zuko, then return him to the battlefield under an opponent's control.",
                targets: TargetPlan::NONE,
                program: &[
                    Instruction::ExileBattlefieldSourceReturnToOpponent,
                    Instruction::LoseLife(PlayerSet::Controller, 2),
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
