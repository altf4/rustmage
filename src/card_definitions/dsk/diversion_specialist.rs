// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords(0),
                    source_only: false,
                    kinds: 33,
                    nonland: false,
                    other: true,
                    tokens: None,
                    subtype: None,
                },
            }),
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: false,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(32768),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32768))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(32768))],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
