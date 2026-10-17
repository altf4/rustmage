// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::SpellColors,
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
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: Some((Some(CounterKind::PlusOne), 5)),
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            2, 2, 2, 2, 2, 0,
        ]))],
        sorcery: false,
        once_per_turn: true,
    }],
    ..CardRules::EMPTY
};
