// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::VIGILANCE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ControlledPermanentTapMana {
                kinds: 1,
                production: ManaProduction::any_color(1),
            }],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceAsEnchantment],
    }],
    ..CardRules::EMPTY
};
