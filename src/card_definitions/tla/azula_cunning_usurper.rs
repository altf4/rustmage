const NONLAND: SacrificeFilter = SacrificeFilter {
    artifact_enchantment_or_token: false,
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: true,
    other: false,
    tokens: None,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ExileControlledLinked,
                    players: PlayerSet::Targeted,
                    filter: SacrificeFilter {
                        kinds: 1,
                        nonland: false,
                        tokens: Some(false),
                        ..NONLAND
                    },
                    amount: 1,
                    optional_skip: None,
                },
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ExileGraveyardLinked,
                    players: PlayerSet::Targeted,
                    filter: NONLAND,
                    amount: 1,
                    optional_skip: None,
                },
            ],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LinkedExilePaidCastOnOwnTurnWithFlash],
    }],
    ..CardRules::EMPTY
};
