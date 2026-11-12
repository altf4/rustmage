// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayDraw { skip: 3 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::SkipIfFaceDownEntryOrTurnUp(1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
