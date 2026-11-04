// The delayed ability follows this exact target's graveyard departure under your control.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: -1,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::GraveyardUnderControllerOnce,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
