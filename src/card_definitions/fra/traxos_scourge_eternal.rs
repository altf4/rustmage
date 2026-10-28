// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::TRAMPLE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::SelfUntap(
                UntapRestriction::ControllerStep,
            )],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::AnyOf(&[CastFilter::Artifact, CastFilter::Creature]),
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapSource],
    }],
    ..CardRules::EMPTY
};
