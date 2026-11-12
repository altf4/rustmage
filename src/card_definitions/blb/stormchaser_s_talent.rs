const OTTER: &[Instruction] = &[Instruction::CreateTokens {
    token: Card(5467),
    amount: 1,
    tapped: false,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: OTTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::BecomesClassLevel(2),
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 4 | 8,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::InstantSorcery,
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: OTTER,
        },
    ],
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(1),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
            cost: ActivationCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(2)],
        },
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
            cost: ActivationCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 5,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(3)],
        },
    ],
    ..CardRules::EMPTY
};
