// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4,5),
    triggers: &[TriggeredAbility { condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE, program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect { spec: LibrarySelectionSpec { look: 0, count: LibrarySelectionCount::Fixed(1), filter: LibrarySelectionFilter::Any, optional: false, reveal: false, remainder: LibraryRemainder::Graveyard }, look: LibraryLook::Amount(AmountValue::Stat(StatValue::Count(CountFilter { zone: CountZone::Battlefield, relation: 0, kind: 1, exclude: 0, subtype: None, basic: false, other: false, cards_only: false }))), destination: LibraryPickDestination::Hand, public: false, maximum_mana_value: None })] }],
    ..CardRules::EMPTY
};
