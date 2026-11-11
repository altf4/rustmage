use rustmage_core::engine::Zone;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDepartureBatch(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            subject: DepartureSubject::Other,
            destinations: &[
                Zone::Hand,
                Zone::Library,
                Zone::Exile,
                Zone::Command,
                Zone::Stack,
            ],
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::ReturnToHand],
    }],
    ..CardRules::EMPTY
};
