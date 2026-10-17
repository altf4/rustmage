// Direct typed behavior; full Oracle and pinned XMage dependencies reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::AddMana(ManaProduction::CountSubtype {
            color: 4,
            subtype: "Elf",
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(
            0,
            StaticModifier {
                own_turn: false,
                requires_life_gain: false,
                filter: CreatureFilter {
                    relation: 0,
                    other: true,
                    subtype: Some("Elf"),
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_modifiers: &[StaticModifier {
            own_turn: false,
            requires_life_gain: false,
            filter: CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Elf"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
