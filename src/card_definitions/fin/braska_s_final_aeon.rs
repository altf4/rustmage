const JECHT_BEAM: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::Chapter { number: 1 },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::Discard(PlayerSet::Opponent, 1),
        Instruction::Draw(PlayerSet::Controller, 1),
    ],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::MENACE,
    triggers: &[
        JECHT_BEAM,
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            ..JECHT_BEAM
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Opponent,
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
                amount: 2,
                optional_skip: None,
            }],
        },
    ],
    ..CardRules::EMPTY
};
