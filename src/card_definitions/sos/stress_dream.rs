// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::Single(TargetSpec { rule: TargetRule::Creature, min: 0, max: 1 }), &[Instruction::Damage(5), Instruction::SelectLibraryTo(LibrarySelectionEffect { spec: LibrarySelectionSpec { look: 0, count: LibrarySelectionCount::Fixed(1), filter: LibrarySelectionFilter::Any, optional: false, reveal: false, remainder: LibraryRemainder::BottomAny }, look: LibraryLook::Fixed(2), destination: LibraryPickDestination::Hand, public: false, maximum_mana_value: None })]),
    ..CardRules::EMPTY
};
