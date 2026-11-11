pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: one(TargetRule::Creature),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChangeControlledPermanentGroupCounters {
            filters: &[ControlledPermanentFilter {
                all_kinds: 1,
                ..ControlledPermanentFilter::ANY
            }],
            effect: PermanentGroupCounters::DoubleKind(CounterKind::PlusOne),
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
