const VAMPIRE_DEMON: Card = Card::from_catalogue_id(5578);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Expression {
                value: StatValue::DescendedThisTurn,
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 5,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: VAMPIRE_DEMON,
            amount: 1,
            tapped: false,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
