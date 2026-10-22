// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[
        Instruction::IfOptionalAdditionalPaid { skip: 2 }, Instruction::ExileTopPlay { amount: 3, next_turn: true }, Instruction::Skip(1), Instruction::ExileTopPlay { amount: 2, next_turn: true },
    ]),
    extras: &CardExtras { additional_cost: Some(AdditionalCost { kind: EffectChoiceKind::OptionalBlightCost(1), filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None } }), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
