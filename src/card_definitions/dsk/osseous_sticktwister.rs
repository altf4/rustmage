pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Expression {
                value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                minimum: Some(4),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                players: PlayerSet::Opponent,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: true,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Skip(1),
            Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::SourcePower),
        ],
    }],
    ..CardRules::EMPTY
};
