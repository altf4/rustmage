const SACRIFICED_MANA_VALUE: AmountValue = AmountValue::ResolutionSacrificedManaValue;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::MENACE
        .union(Keywords::TRAMPLE)
        .union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MaySacrificeForManaValue {
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
                skip: 2,
            },
            Instruction::DrawScaled(PlayerSet::Controller, SACRIFICED_MANA_VALUE),
            Instruction::PutHandPermanent {
                maximum: SACRIFICED_MANA_VALUE,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
