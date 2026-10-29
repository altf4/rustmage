// Partial draft: ward and hand-exile mana permission remain unsupported; gated.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(12, 12),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapControlledLands],
    }],
    ..CardRules::EMPTY
};
