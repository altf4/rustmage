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
    effect: Effect::Creature(6, 6),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::ScaledSingle {
            spec: TargetSpec {
                min: 0,
                max: 5,
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
            },
            maximum: COLORS,
        },
        program: &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: COLORS,
            power: 1,
            toughness: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    ..CardRules::EMPTY
};
