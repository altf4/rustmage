// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 5,
            alternatives: &[],
            additional: None,
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
        program: &[Instruction::GrantKeywords {
            subject: Subject::Source,
            keywords: Keywords::CANT_BE_BLOCKED,
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
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::VIGILANCE,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Base,
            fragments: &[StaticAbilityFragment::BaseStatSetter(
                0,
                BaseStatSetter {
                    power: Some(5),
                    toughness: Some(2),
                    own_turn: true,
                    condition: None,
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::VIGILANCE,
                )],
            },
            StaticAbilityGroup {
                line: 1,
                first_layer: AbilityLayer::Base,
                fragments: &[StaticAbilityFragment::BaseStatSetter(
                    0,
                    BaseStatSetter {
                        power: Some(5),
                        toughness: Some(2),
                        own_turn: true,
                        condition: None,
                    },
                )],
            },
        ],
        base_stat_setter: Some(BaseStatSetter {
            power: Some(5),
            toughness: Some(2),
            own_turn: true,
            condition: None,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
