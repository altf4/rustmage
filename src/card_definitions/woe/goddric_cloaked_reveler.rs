// Direct typed behavior from the pinned Oracle and XMage sources.
const DRAGON_BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(4),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::NonlandEntriesThisTurn,
        minimum: 2,
    }),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::SourceCelebrationCreatureForm {
                    subtypes: &["Dragon"],
                    keywords: Keywords::FLYING,
                    activation: 0,
                },
                StaticAbilityFragment::BaseStatSetter(0, DRAGON_BASE),
            ],
        },
    ],
    extras: &CardExtras {
        base_stat_setter: Some(DRAGON_BASE),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowFilteredScaled {
            // A kindred Dragon can animate after the modifier is created.
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Dragon"),
                basic: false,
                other: false,
                cards_only: false,
            },
            amount: AmountValue::Fixed(1),
            power: 1,
            toughness: 0,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
