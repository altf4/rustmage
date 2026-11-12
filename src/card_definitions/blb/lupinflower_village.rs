const TAP: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Fixed([1, 0, 0, 0, 0, 0]),
                quality: &ManaQuality {
                    restriction: ManaRestriction::CreatureSpells,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 1,
                mana_component: true,
                sacrifice: true,
                ..TAP.cost
            },
            program: &[Instruction::SelectLibrary(LibrarySelectionSpec {
                look: 6,
                count: LibrarySelectionCount::Fixed(1),
                filter: LibrarySelectionFilter::AnyOf(&[
                    LibrarySelectionFilter::Subtype("Bat"),
                    LibrarySelectionFilter::Subtype("Bird"),
                    LibrarySelectionFilter::Subtype("Mouse"),
                    LibrarySelectionFilter::Subtype("Rabbit"),
                ]),
                optional: true,
                reveal: true,
                remainder: LibraryRemainder::BottomRandom,
            })],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
