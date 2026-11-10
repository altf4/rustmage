// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::FilteredCost {
                    action: CostAction::TapPermanents(2),
                    any: &[CostMatch::Kinds(1), CostMatch::Subtype("Treasure")],
                },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 2,
                optional_skip: Some(2),
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Skip(1),
            Instruction::SacrificeSource,
        ],
    }],
    ..CardRules::EMPTY
};
