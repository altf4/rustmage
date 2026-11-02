// Flash and the mandatory other-controlled creature-or-Vehicle entry target.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::CreatureOrVehicle { relation: 0 },
            predicate: &TargetPredicate::OtherThanSource,
        })),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: 2,
            toughness: 2,
        }],
    }],
    ..CardRules::EMPTY
};
