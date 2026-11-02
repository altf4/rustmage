// Entry destruction is independent of the hand cycling ability and crew activation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::REACH.union(Keywords::MENACE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 1,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: 4,
        }),
        program: &[Instruction::Destroy],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 3 }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Hand,
            cycling: true,
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                discard: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
