// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ManaSpent {
                minimum: 0,
                increment: true,
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Any,
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfCountersAdded {
                kind: CounterKind::PlusOne,
                controller_only: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::any_color(1))],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ActivationX,
            Instruction::CreateTokensWithCounters {
                token: Card::SosFractalToken,
                amount: 1,
                kind: CounterKind::PlusOne,
                counters: AmountValue::ChosenX,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
