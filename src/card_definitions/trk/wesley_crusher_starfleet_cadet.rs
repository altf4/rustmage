// Complete Oracle and pinned WesleyCrusherStarfleetCadet.java reviewed. The
// six nonrepeating pairs enumerate exactly two of the four printed choices.
const COUNTER: TargetRule = TargetRule::SpellKinds {
    kinds: 0,
    excluded: 1,
};
const SMALL: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: 2,
};
const CREATURE: TargetRule = TargetRule::Creature;
const PLAYER: TargetRule = TargetRule::Player {
    opponent_only: false,
};
const UNLESS_TWO: Instruction = Instruction::CounterUnlessPay {
    cost: generic(2),
    subject: CounterSubject::Targets,
    exile: false,
    unpaid_skip: 0,
};
const BOUNCE: Instruction = Instruction::ReturnToHand;
const UNBLOCK: Instruction = Instruction::GrantKeywords {
    subject: Subject::Targets,
    keywords: Keywords::CANT_BE_BLOCKED,
};
const DRAW: Instruction = Instruction::Draw(PlayerSet::Targeted, 2);
const DISCARD: Instruction = Instruction::Discard(PlayerSet::Targeted, 2);
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Counter a noncreature spell unless its controller pays two; return a small creature",
                targets: TargetPlan::Groups(&[TargetSpec::one(COUNTER), TargetSpec::one(SMALL)]),
                program: &[
                    Instruction::SelectGroup(0),
                    UNLESS_TWO,
                    Instruction::SelectGroup(1),
                    BOUNCE,
                ],
            },
            SpellMode {
                text: "Counter a noncreature spell unless its controller pays two; target creature can't be blocked",
                targets: TargetPlan::Groups(&[TargetSpec::one(COUNTER), TargetSpec::one(CREATURE)]),
                program: &[
                    Instruction::SelectGroup(0),
                    UNLESS_TWO,
                    Instruction::SelectGroup(1),
                    UNBLOCK,
                ],
            },
            SpellMode {
                text: "Counter a noncreature spell unless its controller pays two; target player draws two then discards two",
                targets: TargetPlan::Groups(&[TargetSpec::one(COUNTER), TargetSpec::one(PLAYER)]),
                program: &[
                    Instruction::SelectGroup(0),
                    UNLESS_TWO,
                    Instruction::SelectGroup(1),
                    DRAW,
                    DISCARD,
                ],
            },
            SpellMode {
                text: "Return a small creature; target creature can't be blocked",
                targets: TargetPlan::Groups(&[TargetSpec::one(SMALL), TargetSpec::one(CREATURE)]),
                program: &[
                    Instruction::SelectGroup(0),
                    BOUNCE,
                    Instruction::SelectGroup(1),
                    UNBLOCK,
                ],
            },
            SpellMode {
                text: "Return a small creature; target player draws two then discards two",
                targets: TargetPlan::Groups(&[TargetSpec::one(SMALL), TargetSpec::one(PLAYER)]),
                program: &[
                    Instruction::SelectGroup(0),
                    BOUNCE,
                    Instruction::SelectGroup(1),
                    DRAW,
                    DISCARD,
                ],
            },
            SpellMode {
                text: "Target creature can't be blocked; target player draws two then discards two",
                targets: TargetPlan::Groups(&[TargetSpec::one(CREATURE), TargetSpec::one(PLAYER)]),
                program: &[
                    Instruction::SelectGroup(0),
                    UNBLOCK,
                    Instruction::SelectGroup(1),
                    DRAW,
                    DISCARD,
                ],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
