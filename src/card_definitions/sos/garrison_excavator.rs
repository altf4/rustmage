// Direct typed Oracle behavior; full XMage source pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyard,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5350),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
