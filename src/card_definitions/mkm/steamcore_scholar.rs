// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::DiscardOneMatchingOr {
                filter: SearchFilter::AnyOf(&[
                    SearchFilter::Kinds(4 | 8),
                    SearchFilter::AllOf(&[
                        SearchFilter::Creature,
                        SearchFilter::Keyword(Keywords::FLYING),
                    ]),
                ]),
                otherwise: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
