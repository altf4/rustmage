const TAP: ActivatedAbility = ActivatedAbility {
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    guard: ActivationGuard::Always,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            ..TAP
        },
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::any_color(1),
                quality: &ManaQuality {
                    restriction: ManaRestriction::SpellSubtypeAny(&["Mount"]),
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 1,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Subtype("Mount"),
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::OptionalBottom,
                },
                look: LibraryLook::Fixed(1),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            })],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
