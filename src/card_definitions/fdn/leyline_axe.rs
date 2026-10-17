// Direct Oracle/XMage port; opening-hand placement and normal equip share one definition.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 3,
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
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::AttachSource],
        sorcery: true,
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
    keywords: Keywords::NONE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentKeywords(
            Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE),
        )],
    }],
    extras: &CardExtras {
        opening_hand_battlefield: true,
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 1,
            },
            keywords: Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE),
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentKeywords(
                Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE),
            )],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
