// Molten Tide; gated together with its prepared front face until validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::AddMountainManaTrigger]),
    extras: &CardExtras { prepared_identity: Some("@prepare/pyre_rhymer_molten_tide"), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
