// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Creatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                qualification: DamageQualification::Any,
            },
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Quest,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Quest),
                minimum: 4,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 4,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[
            Instruction::WaterbendCost(4),
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BE_BLOCKED,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
