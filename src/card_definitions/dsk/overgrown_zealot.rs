// Directly authored from the paired Oracle and pinned XMage sources.
const MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::any_color(1))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    activated: &[
        MANA,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::any_color(2),
                quality: &ManaQuality {
                    restriction: ManaRestriction::TurnFaceUp,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..MANA
        },
    ],
    ..CardRules::EMPTY
};
