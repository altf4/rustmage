// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::JobSelect { token: Card(5328) }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            }),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::DiscardManaValueLifeLoss,
            ],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 2,
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
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedSubtype("Ninja"),
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 1,
            }),
            StaticAbilityFragment::AttachmentTrigger(1),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 1,
            },
            keywords: Keywords::NONE,
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::AttachedSubtype("Ninja"),
                StaticAbilityFragment::AttachmentStats(Stats {
                    power: 1,
                    toughness: 1,
                }),
                StaticAbilityFragment::AttachmentTrigger(1),
            ],
        }],
        attached_subtype: Some("Ninja"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
