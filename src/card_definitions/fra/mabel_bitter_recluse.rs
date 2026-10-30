// Authored from pinned Oracle and complete XMage sources; awaiting rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::DEATHTOUCH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DEATHTOUCH,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 65,
            relation: -1,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::RemoveUpToTargetCounters(3)],
    }],
    ..CardRules::EMPTY
};
