// Complete typed behavior from full pinned Oracle and XMage sources.
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::AtLeast {
        value: &StatValue::Count(CountFilter {
            zone: CountZone::Graveyard,
            relation: 0,
            kind: 1,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        minimum: 2,
    },
    stats: Stats {
        power: 2,
        toughness: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, BONUS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
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
                        kinds: 1,
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
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Destroy],
        },
    ],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
