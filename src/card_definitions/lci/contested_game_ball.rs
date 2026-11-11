const TREASURE: Card = Card::from_catalogue_id(5302);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CombatDamageToController,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainControl {
                subject: Subject::Source,
                controller: PlayerSet::EventPlayer,
            },
            Instruction::UntapSource,
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Point,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Point),
                minimum: 5,
                skip: 2,
            },
            Instruction::SacrificeSource,
            Instruction::CreateTokens {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
