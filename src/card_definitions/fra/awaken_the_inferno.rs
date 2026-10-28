// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 65,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::Damage(6),
            Instruction::SelectGroup(1),
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    ),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: true,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: true,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
