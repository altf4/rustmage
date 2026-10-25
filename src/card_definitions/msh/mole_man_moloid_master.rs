// Full frozen Oracle and direct pinned Java/helper audit; no per-source land limit.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::PlayLandFromGraveyard],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5402),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
