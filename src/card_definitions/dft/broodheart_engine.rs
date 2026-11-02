// Sorcery timing, tap, sacrifice and exact graveyard targets use the normal payment route.
const OWN_GRAVE: TargetRule = TargetRule::Object {
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
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Surveil(1)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 0, 1, 0, 1, 0],
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Union(&[
            TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            TargetRule::Filtered {
                base: &OWN_GRAVE,
                predicate: &TargetPredicate::Subtype("Vehicle"),
            },
        ])),
        program: &[Instruction::ReturnToBattlefield],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
