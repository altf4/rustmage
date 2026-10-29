// Authored from pinned Oracle and XMage sources; gated pending validation.
const DOMAIN: DynamicStats = DynamicStats {
    power: Some(StatExpr { value: StatValue::ControllerDomain, multiplier: 1, offset: 0 }),
    toughness: None,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Cda,
    fragments: &[StaticAbilityFragment::DynamicStats(0, DOMAIN)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    static_groups: GROUPS,
    extras: &CardExtras { x_symbols: 1, static_ability_groups: GROUPS, dynamic_stats: Some(DOMAIN), ..CardExtras::EMPTY },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchDifferentNamesChosenX(SearchSpec {
            filter: SearchFilter::BasicLand, maximum: 0,
            destination: SearchDestination::Hand, tapped: false, reveal: true,
        })],
    }],
    ..CardRules::EMPTY
};
