// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 3 }],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ControlledCreatureSubtype {
                grant: CreatureSubtypeGrant::Chosen,
                required_subtype: Some("Vehicle"),
            }],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ChosenModifier(ChosenModifier {
                kind: EntryChoiceKind::CreatureType,
                other: true,
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
                source_counter: None,
            })],
        },
    ],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
