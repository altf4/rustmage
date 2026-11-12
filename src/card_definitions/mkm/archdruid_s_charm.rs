const OPPOSING_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Search your library for a creature or land card, reveal it, and put it onto the battlefield tapped if it's a land or into your hand otherwise. Then shuffle.",
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::Kinds(1 | 2),
                maximum: 1,
                destination: SearchDestination::BattlefieldIfLandElseHand,
                tapped: true,
                reveal: true,
            })],
        },
        SpellMode {
            text: "Put a +1/+1 counter on target creature you control. It deals damage equal to its power to target creature you don't control.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::creature_you_control()),
                TargetSpec::one(OPPOSING_CREATURE),
            ]),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::CreaturePowerDamageAny {
                    source: 0,
                    recipient: 1,
                },
            ],
        },
        SpellMode {
            text: "Exile target artifact or enchantment.",
            targets: one(object(16 | 32)),
            program: &[Instruction::Exile],
        },
    ]),
    ..CardRules::EMPTY
};
