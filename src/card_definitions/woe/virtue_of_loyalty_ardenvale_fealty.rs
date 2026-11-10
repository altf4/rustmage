// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::UntapPlayerCreatures(PlayerSet::Controller),
        ],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5565)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
