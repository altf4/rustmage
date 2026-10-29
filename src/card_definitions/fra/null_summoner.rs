// Full frozen Oracle and immutable upstream PR implementation reviewed; gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::RevealTargetHandExileLinked],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastLinkedExileAtThreshold],
    }],
    ..CardRules::EMPTY
};
