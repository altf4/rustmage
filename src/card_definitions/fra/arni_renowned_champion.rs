// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 5),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowResolved {
            subject: Subject::Source,
            value: ResolvedStatValue::EnteringPower,
            power: 1,
            toughness: 0,
        }],
    }],
    ..CardRules::EMPTY
};
