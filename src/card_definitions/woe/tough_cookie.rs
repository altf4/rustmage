pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5246),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 2,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 16,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::WithoutKinds(1),
            }),
            program: &[Instruction::AnimateTargets(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            })],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
