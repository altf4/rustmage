// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
const KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::TRAMPLE);
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: KEYWORDS,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::SearchRequired(SearchSpec {
                filter: SearchFilter::Any,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: false,
            }),
            Instruction::DiscardRandom(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
