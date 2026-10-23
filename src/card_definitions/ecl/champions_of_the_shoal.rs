// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrBecomesTapped,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::Tap,
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::Stun,
                    amount: 1,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeavesBattlefield,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnLinkedExileToHand],
        },
    ],
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdAndExile,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Merfolk"),
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
