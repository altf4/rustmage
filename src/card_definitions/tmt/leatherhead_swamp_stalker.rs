// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::TRAMPLE,
    extras: &CardExtras {
        fixed_entry_counters: &[(CounterKind::Hexproof, 1)],
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::MayRemoveAnySourceCounterReflexive(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::DamagedPlayerPermanent { kinds: 16 | 32 }),
            program: &[Instruction::Destroy],
        },
    ],
    ..CardRules::EMPTY
};
