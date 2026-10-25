// Whole-card direct Oracle behavior, with independent hybrid and additional costs.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterRelatedUnlessDiscardOrPay(ManaCost {
            colored: [0; 6],
            generic: 2,
        })],
    }],
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Discard,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
        }],
        additional_mana_alternative: Some(ManaCost {
            colored: [0; 6],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
