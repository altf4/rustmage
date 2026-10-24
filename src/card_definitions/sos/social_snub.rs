// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Each,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::LoseLife(PlayerSet::Opponent, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::SelfCast,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::CopyCastSpell {
                prior_spells: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
