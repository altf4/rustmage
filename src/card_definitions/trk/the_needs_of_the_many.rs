// Complete Oracle reviewed; pinned XMage has no class. The sacrifice is a
// committed casting cost; the three protections have independent durations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrantControlledCreaturesThisTurn(
                Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE),
            ),
            Instruction::PlayerHexproofThisTurn,
            Instruction::PreventPlayerDamageThisTurn,
            Instruction::ExileResolvingSpell,
        ],
    ),
    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
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
