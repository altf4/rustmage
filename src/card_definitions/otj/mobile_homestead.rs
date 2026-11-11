// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Mount"),
                basic: false,
                other: false,
                cards_only: false,
            }),
            keywords: Keywords::HASTE,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 1,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::Kinds(2),
                optional: true,
                reveal: false,
                remainder: LibraryRemainder::Keep,
            },
            look: LibraryLook::Fixed(1),
            destination: LibraryPickDestination::BattlefieldTapped,
            public: false,
            maximum_mana_value: None,
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 2 }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
