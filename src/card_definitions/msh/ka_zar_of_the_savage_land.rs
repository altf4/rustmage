// Full frozen Oracle and direct pinned Java/helper audit; ordinary land timing.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::PlayLandFromLibraryTop],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5401),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
