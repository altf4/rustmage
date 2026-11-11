pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceNoncreature,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::NoSpellCastFromOwnHandThisTurn,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AnimateSource(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Additional(&["Spirit"]),
            base: Some((3, 3)),
            keywords: Keywords::FLYING,
            duration: AnimationDuration::Permanent,
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
