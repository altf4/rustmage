const INGENUITY: &[Instruction] = &[Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::Ingenuity,
    amount: 1,
}];

const fn draw_trigger(ordinal: u64) -> TriggeredAbility {
    TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(ordinal),
        },
        targets: TargetPlan::NONE,
        program: INGENUITY,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    triggers: &[draw_trigger(1), draw_trigger(2)],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CastHandFreeUpToOfType {
            maximum: AmountValue::SourceCounterQuantity(CounterKind::Ingenuity),
            permanent_type: PermanentType::Artifact,
        }],
    }],
    ..CardRules::EMPTY
};
