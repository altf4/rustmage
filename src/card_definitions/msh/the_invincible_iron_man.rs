// The separately castable modal back face has its own printed 4UR mana value.
pub(in crate::cards) const RULES: CardRules = CardRules {
    deck_limit: None,
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutHandAndAttachEquipment {
            filter: SearchFilter::Artifact,
        }],
    }],
    ..CardRules::EMPTY
};
