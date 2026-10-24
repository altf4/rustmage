// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CreateTokens {
            token: Card::SosSpiritToken,
            amount: 1,
            tapped: false,
        }],
    ),
    extras: &CardExtras {
        flashback_cost: Some(generic(0)),
        flashback_additional: Some(AdditionalCost {
            kind: EffectChoiceKind::TapPermanents(3),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
