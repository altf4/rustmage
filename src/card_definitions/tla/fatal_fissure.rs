// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::DelayTargetEvent {
            event: DelayedTargetEvent::DiesOnce,
            ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::AnimateEarthbendTargets,
            Instruction::RegisterEarthbendReturn,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 4,
            },
        ],
    }],
    ..CardRules::EMPTY
};
