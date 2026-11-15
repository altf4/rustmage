// Pinned Oracle and complete XMage class: only an actual cast entry triggers the
// simultaneous destruction of creatures without the artifact card type.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::DestroyCreaturesWithoutKinds(16)],
    }],
    ..CardRules::EMPTY
};
