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

const fn subtype(name: &'static str) -> ControlledPermanentFilter {
    ControlledPermanentFilter {
        subtype: Some(name),
        ..ControlledPermanentFilter::ANY
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Fixed([0, 0, 0, 0, 1, 0]),
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
                colored: [0, 0, 0, 0, 1, 0],
                mana_component: true,
                ..TAP.cost
            },
            program: &[Instruction::ChangeNewlyEnteredPermanentGroupCounters {
                filters: &[
                    subtype("Frog"),
                    subtype("Rabbit"),
                    subtype("Raccoon"),
                    subtype("Squirrel"),
                ],
                effect: PermanentGroupCounters::Add {
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            }],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
