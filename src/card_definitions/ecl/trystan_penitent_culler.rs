// Back-face milling precedes an optional, non-targeted exact Elf-card exile cost.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::DEATHTOUCH,
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(4697),
            back: Card(5349),
            front_name: "Trystan, Callous Cultivator",
        }),
        ..CardExtras::EMPTY
    },
    deck_limit: None,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTransforms,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::ExileGraveyardCard,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: Some("Elf"),
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::LoseLife(PlayerSet::Opponent, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::Main1,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 0, 0, 1, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::TransformSource,
            ],
        },
    ],
    ..CardRules::EMPTY
};
