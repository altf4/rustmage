// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::Incubation,
                amount: AmountValue::DamageEventAmount,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        cost: ActivationCost {
            colored: [0; 6],
            alternatives: &[],
            exile_self: false,
            discard: false,
            life: 0,
            loyalty: None,
            generic: 0,
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            remove: Some((Some(CounterKind::Incubation), 3)),
        },
        program: &[Instruction::CreateTokens {
            token: Card::DrakeToken,
            amount: 1,
            tapped: false,
        }],
        sorcery: false,
    }],
    ..CardRules::EMPTY
};
