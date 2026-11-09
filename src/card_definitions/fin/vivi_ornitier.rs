pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OwnTurn,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::SourcePowerSplit {
            first: ChoiceColor::Blue,
            second: ChoiceColor::Red,
        })],
        sorcery: false,
        once_per_turn: true,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::DamagePlayers(PlayerSet::Opponent, 1),
        ],
    }],
    ..CardRules::EMPTY
};
