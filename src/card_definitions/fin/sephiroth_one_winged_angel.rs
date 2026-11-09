// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeAnyNumber(SacrificeFilter {
                kinds: 1,
                other: true,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            }),
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::MovedCount),
        ],
    }],
    extras: &CardExtras {
        as_transform: Some(AsTransformEffect::CreateEmblem(Card(5535))),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
