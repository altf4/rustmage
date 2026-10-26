// Complete printed obligations from pinned Oracle and XMage sources.
const BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::HasEnduringStory,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Storied],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::HasEnduringStory,
                keywords: Keywords::HASTE,
            },
            StaticAbilityFragment::ScalingModifier(0, BONUS),
        ],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    static_groups: GROUPS,
    extras: &CardExtras {
        scaling_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 1,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
            }),
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
