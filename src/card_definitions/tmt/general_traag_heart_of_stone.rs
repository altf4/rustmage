// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(4)],
        },
    ],
    ..CardRules::EMPTY
};
