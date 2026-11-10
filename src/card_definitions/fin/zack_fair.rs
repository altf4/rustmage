pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    enters: Counters {
        plus_one: 1,
        ..CardRules::EMPTY.enters
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::INDESTRUCTIBLE,
            },
            Instruction::PutSourceCountersOnTargets,
            Instruction::AttachPriorSourceEquipment { recipient: 0 },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
