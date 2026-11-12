const LAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});

const COUNTER: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::PlusOne,
    amount: 1,
};

const CREATURES_WITH_COUNTERS: AmountValue = AmountValue::Stat(
    StatValue::ControlledCreaturesWithCounter(CounterKind::PlusOne),
);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Groups(&[LAND, LAND]),
            program: &[
                Instruction::AnimateEarthbendTargets,
                COUNTER,
                Instruction::RegisterEarthbendReturn,
                Instruction::SelectGroup(1),
                Instruction::AnimateEarthbendTargets,
                COUNTER,
                Instruction::RegisterEarthbendReturn,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLifeScaled(PlayerSet::Opponent, CREATURES_WITH_COUNTERS),
                Instruction::GainLifeScaled(PlayerSet::Controller, CREATURES_WITH_COUNTERS),
            ],
        },
    ],
    ..CardRules::EMPTY
};
