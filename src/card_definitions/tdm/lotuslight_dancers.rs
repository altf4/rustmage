// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 6),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchDistinctColors {
            spec: SearchSpec {
                filter: SearchFilter::Any,
                maximum: 3,
                destination: SearchDestination::Graveyard,
                tapped: false,
                reveal: false,
            },
            colors: 22,
        }],
    }],
    ..CardRules::EMPTY
};
