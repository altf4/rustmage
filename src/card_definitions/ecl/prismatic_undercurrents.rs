// Direct typed behavior from the pinned Oracle and XMage sources.
const COLORS: AmountValue = AmountValue::Stat(StatValue::Colors(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchScaled {
            spec: SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 5,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            },
            maximum: COLORS,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalLandPlays(1)],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalLandPlays(1)],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
