// Direct whole-card behavior from the frozen Oracle and complete pinned Java sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FLASH.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Permanent,
            },
            predicate: &TargetPredicate::EnteredGraveyardThisTurn,
        })),
        program: &[Instruction::ReturnToHand],
    }],
    ..CardRules::EMPTY
};
