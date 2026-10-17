// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 12),
    keywords: Keywords::TRAMPLE,
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CounterUnlessPay {
                cost: generic(4),
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5335),
                amount: 4,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
