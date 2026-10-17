// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 16,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ExileUntilSourceLeaves],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 0, 0],
                    generic: 1,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
    ],
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
        kinds: 16,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords(0),
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentTrigger(1)],
    }],
    extras: &CardExtras {
        aura: Some(AuraSpec {
            kinds: 16,
            relation: 0,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords(0),
            },
        }),
        static_ability_groups: &[StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentTrigger(1)],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
