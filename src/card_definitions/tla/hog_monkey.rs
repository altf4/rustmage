// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::HasCounter(CounterKind::PlusOne),
        }),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::MENACE,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: true,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 5,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 2,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
