// Full Oracle and pinned XMage class reviewed. Chapter I creates the phasing-lock Human after phasing.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PhaseOutAll(1 | 16),
                Instruction::CreateTokens { token: Card(6017), amount: 1, tapped: false },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::LoseLife(PlayerSet::Each, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE, source_only: false, kinds: 1,
                        nonland: false, other: false, tokens: Some(true),
                        artifact_enchantment_or_token: false, subtype: None,
                    },
                    amount: 1, optional_skip: Some(1),
                },
                Instruction::GainLife(PlayerSet::Controller, 5),
            ],
        },
    ],
    ..CardRules::EMPTY
};
