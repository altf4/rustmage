pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    extras: &CardExtras {
        split_faces: Some([Card::from_catalogue_id(5689), Card::from_catalogue_id(5690)]),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
