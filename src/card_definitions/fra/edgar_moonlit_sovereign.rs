// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            guard: StepGuard::Expression {
                value: StatValue::SpellsCastThisTurn,
                minimum: None,
                maximum: Some(0),
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 2,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutControlledCreatureCountersWith {
            required: CounterKind::PlusOne,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
