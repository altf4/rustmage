// Direct Oracle and pinned XMage behavior; gated pending interaction validation.
const GRAVEYARD_SCALING: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard, relation: 0, kind: 1 | 64, exclude: 0,
        subtype: None, basic: false, other: false, cards_only: true,
    }),
    stats: Stats { power: 1, toughness: 0 },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice, players: PlayerSet::Controller,
                    filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false,
                        kinds: 1 | 64, nonland: false, other: false, tokens: None,
                        artifact_enchantment_or_token: false, subtype: None },
                    amount: 1, optional_skip: Some(1),
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive, targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice, players: PlayerSet::Opponent,
                filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false,
                    kinds: 1, nonland: false, other: false, tokens: None,
                    artifact_enchantment_or_token: false, subtype: None },
                amount: 1, optional_skip: None,
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1, first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, GRAVEYARD_SCALING)],
    }],
    extras: &CardExtras { scaling_modifiers: &[GRAVEYARD_SCALING], ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
