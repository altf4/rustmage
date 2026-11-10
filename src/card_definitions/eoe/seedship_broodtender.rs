const GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Mill(
            PlayerSet::Controller,
            MillAmount::Fixed(3),
        )],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 3,
            colored: [0, 0, 1, 0, 1, 0],
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Union(&[
            TargetRule::Filtered {
                base: &GRAVE,
                predicate: &TargetPredicate::Kinds(1),
            },
            TargetRule::Filtered {
                base: &GRAVE,
                predicate: &TargetPredicate::Subtype("Spacecraft"),
            },
        ])),
        program: &[Instruction::ReturnToBattlefield],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
