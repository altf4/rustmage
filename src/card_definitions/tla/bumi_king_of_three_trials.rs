// Direct typed behavior from the paired Oracle and pinned XMage sources.
const PLAYER: TargetSpec = TargetSpec::one(TargetRule::Player {
    opponent_only: false,
});
const LAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const COUNTERS: Instruction = Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 3,
};
const SCRY: Instruction = Instruction::ScryTargetPlayer(3);
const ANIMATE: Instruction = Instruction::AnimateEarthbendTargets;
const LAND_COUNTERS: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::PlusOne,
    amount: 3,
};
const REGISTER: Instruction = Instruction::RegisterEarthbendReturn;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ModalUpToStat {
            maximum: StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Lesson"),
                basic: false,
                other: false,
                cards_only: true,
            }),
            counts: &[0, 1, 1, 1, 2, 2, 2, 3],
            modes: &[
                SpellMode {
                    text: "Choose no modes",
                    targets: TargetPlan::NONE,
                    program: &[],
                },
                SpellMode {
                    text: "Put three +1/+1 counters on Bumi",
                    targets: TargetPlan::NONE,
                    program: &[COUNTERS],
                },
                SpellMode {
                    text: "Target player scries 3",
                    targets: TargetPlan::Single(PLAYER),
                    program: &[SCRY],
                },
                SpellMode {
                    text: "Earthbend 3",
                    targets: TargetPlan::Single(LAND),
                    program: &[ANIMATE, LAND_COUNTERS, REGISTER],
                },
                SpellMode {
                    text: "Put counters on Bumi and scry",
                    targets: TargetPlan::Single(PLAYER),
                    program: &[COUNTERS, SCRY],
                },
                SpellMode {
                    text: "Put counters on Bumi and earthbend",
                    targets: TargetPlan::Single(LAND),
                    program: &[COUNTERS, ANIMATE, LAND_COUNTERS, REGISTER],
                },
                SpellMode {
                    text: "Scry and earthbend",
                    targets: TargetPlan::Groups(&[PLAYER, LAND]),
                    program: &[
                        SCRY,
                        Instruction::SelectGroup(1),
                        ANIMATE,
                        LAND_COUNTERS,
                        REGISTER,
                    ],
                },
                SpellMode {
                    text: "Put counters on Bumi, scry, and earthbend",
                    targets: TargetPlan::Groups(&[PLAYER, LAND]),
                    program: &[
                        COUNTERS,
                        SCRY,
                        Instruction::SelectGroup(1),
                        ANIMATE,
                        LAND_COUNTERS,
                        REGISTER,
                    ],
                },
            ],
        }],
    }],
    ..CardRules::EMPTY
};
