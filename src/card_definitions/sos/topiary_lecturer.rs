// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
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
    }],
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
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::SourcePower {
            color: 4,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
