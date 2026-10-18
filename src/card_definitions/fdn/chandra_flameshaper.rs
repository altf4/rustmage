const fn loyalty(
    change: i32,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: Some(change),
        },
        targets,
        program,
        sorcery: true,
        once_per_turn: false,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        loyalty(
            2,
            TargetPlan::NONE,
            &[
                Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 3, 0, 0])),
                Instruction::ExileTopChoosePlay {
                    amount: 3,
                    next_turn: false,
                },
            ],
        ),
        loyalty(
            1,
            TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            &[Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Targets,
                amount: 1,
                exceptions: CopyExceptions::HASTE_AND_SACRIFICE,
            }],
        ),
        loyalty(
            -4,
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 65,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 8,
            }),
            &[Instruction::DamageDivided { total: 8 }],
        ),
    ],
    extras: &CardExtras {
        starting_loyalty: 6,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
