// Endure uses all source counters at resolution, including departure LKI.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PermanentEntersYou {
                kinds: 1,
                nontoken: true,
                subtype: None,
                include_source: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::EndureScaled {
                recipient: EndureRecipient::Entering,
                amount: AmountValue::SourceTotalCounters,
                token: Card(5326),
            }],
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
