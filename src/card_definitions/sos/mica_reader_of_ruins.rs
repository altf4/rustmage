// Direct typed Oracle behavior; full XMage source pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessPayLife(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: false,
                        tokens: None,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::CopyCastSpell {
                    prior_spells: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
