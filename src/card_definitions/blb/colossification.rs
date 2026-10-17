// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::TapAttached],
    }],
    activated: &[],
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
                power: 20,
                toughness: 20,
            },
            keywords: Keywords(0),
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
            power: 20,
            toughness: 20,
        })],
    }],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 1,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 20,
                    toughness: 20,
                },
                keywords: Keywords(0),
            },
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::AttachmentStats(Stats {
                power: 20,
                toughness: 20,
            })],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
