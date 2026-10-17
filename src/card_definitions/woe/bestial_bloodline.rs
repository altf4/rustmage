// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
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
        program: &[Instruction::ReturnSourceToHand(TargetZone::Graveyard)],
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
    keywords: Keywords(0),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 2,
                toughness: 2,
            },
            keywords: Keywords(0),
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
            power: 2,
            toughness: 2,
        })],
    }],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 2,
                    toughness: 2,
                },
                keywords: Keywords(0),
            },
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
                power: 2,
                toughness: 2,
            })],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
