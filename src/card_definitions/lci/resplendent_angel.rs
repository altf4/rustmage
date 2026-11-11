// Count gross life gained, including gains during the opponent's turn.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::Any,
            guard: StepGuard::Expression {
                value: StatValue::LifeGainedThisTurn,
                minimum: Some(5),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5569),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [3, 0, 0, 0, 0, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::LIFELINK,
            },
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
