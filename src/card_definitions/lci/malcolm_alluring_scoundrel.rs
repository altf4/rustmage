// Check the live or last chorus total after the exact discard is complete.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Chorus,
                amount: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::DiscardOneRememberCard,
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Chorus),
                minimum: 4,
                skip: 1,
            },
            Instruction::CastRememberedDiscardFree,
        ],
    }],
    ..CardRules::EMPTY
};
