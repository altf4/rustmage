// Pinned Oracle and complete XMage class: the delayed trigger follows the exact creature.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::DiesOnce,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
