// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
            spec: LibrarySelectionSpec {
                look: 7,
                count: LibrarySelectionCount::All,
                filter: LibrarySelectionFilter::Kinds(32),
                optional: false,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            },
            look: LibraryLook::Fixed(7),
            destination: LibraryPickDestination::Hand,
            public: true,
            maximum_mana_value: None,
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Subtype("Room"),
        }),
        program: &[Instruction::ChooseRoomDoor {
            selection: crate::engine::RoomSelection::Target,
            operation: crate::engine::RoomOperation::Either,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
