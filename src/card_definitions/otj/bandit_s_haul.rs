pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CommittedCrime),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Loot,
            amount: 1,
        }],
    }],
    activated: &[
        ActivatedAbility {
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            guard: ActivationGuard::Always,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            })],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            guard: ActivationGuard::Always,
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                tap: true,
                remove: Some((Some(CounterKind::Loot), 2)),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
