// Authored from the pinned Oracle and XMage implementation; admission awaits validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::VIGILANCE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::VIGILANCE,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamaged,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::SearchDamageReceived(SearchSpec {
                filter: SearchFilter::Land,
                maximum: 0,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
