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
                    restriction: ManaRestriction::SubtypeSpellOrActivation("Ally"),
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 5,
                mana_component: true,
                ..TAP.cost
            },
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5272),
                amount: 1,
                tapped: false,
            }],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
