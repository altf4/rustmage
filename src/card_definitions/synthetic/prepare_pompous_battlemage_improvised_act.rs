// Improvised Act is the prepared sorcery associated with Pompous Battlemage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::DiscardUpToOneDraw]),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/pompous_battlemage_improvised_act"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
