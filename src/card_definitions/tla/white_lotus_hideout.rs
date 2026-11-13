// Direct typed behavior from the pinned Oracle and XMage sources.
const TAP: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
    targets: TargetPlan::NONE,
    program: &[],
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
                production: &ManaProduction::Choice {
                    mask: 31,
                    amount: 1,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::SpellSubtypeAny(&["Lesson", "Shrine"]),
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                ..TAP.cost
            },
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            })],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
