// Direct Oracle and pinned XMage behavior; gated pending rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfNotCastFromHand { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::Skip(1),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        }],
        flashback_cost: Some(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 1,
        }),
        flashback_cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 1,
            },
        ],
        flashback_additional: Some(AdditionalCost {
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
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
