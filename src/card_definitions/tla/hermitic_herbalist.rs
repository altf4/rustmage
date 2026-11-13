const UNRESTRICTED: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::any_color(1))],
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[
        UNRESTRICTED,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Combination {
                    mask: 31,
                    amount: 2,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::SpellSubtype("Lesson"),
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..UNRESTRICTED
        },
    ],
    ..CardRules::EMPTY
};
