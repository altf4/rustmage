const COPY_TARGET: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1 | 16,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const SPELL_TARGET: TargetSpec = TargetSpec::one(TargetRule::Spell);
const COPY: Instruction = Instruction::CreateCopyToken {
    subject: Subject::Targets,
    amount: 1,
};
const DRAW: Instruction = Instruction::Draw(PlayerSet::Controller, 2);
const DISCARD: Instruction = Instruction::Discard(PlayerSet::Controller, 1);
const fn mode(
    text: &'static str,
    targets: TargetPlan,
    program: &'static [Instruction],
) -> SpellMode {
    SpellMode {
        text,
        targets,
        program,
    }
}
const fn counter_cost(generic: u32) -> ManaCost {
    ManaCost {
        colored: [0, 1, 0, 0, 0, 0],
        generic,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        mode(
            "Counter a spell",
            TargetPlan::Single(SPELL_TARGET),
            &[Instruction::Counter],
        ),
        mode(
            "Copy an artifact or creature you control",
            TargetPlan::Single(COPY_TARGET),
            &[COPY],
        ),
        mode(
            "Draw two cards, then discard a card",
            TargetPlan::NONE,
            &[DRAW, DISCARD],
        ),
        mode(
            "Counter a spell and copy a permanent",
            TargetPlan::Groups(&[SPELL_TARGET, COPY_TARGET]),
            &[Instruction::Counter, Instruction::SelectGroup(1), COPY],
        ),
        mode(
            "Counter a spell, then draw and discard",
            TargetPlan::Single(SPELL_TARGET),
            &[Instruction::Counter, DRAW, DISCARD],
        ),
        mode(
            "Copy a permanent, then draw and discard",
            TargetPlan::Single(COPY_TARGET),
            &[COPY, DRAW, DISCARD],
        ),
        mode(
            "Counter a spell, copy a permanent, then draw and discard",
            TargetPlan::Groups(&[SPELL_TARGET, COPY_TARGET]),
            &[
                Instruction::Counter,
                Instruction::SelectGroup(1),
                COPY,
                DRAW,
                DISCARD,
            ],
        ),
    ]),
    mode_costs: Some(&[
        counter_cost(1),
        generic(3),
        generic(2),
        counter_cost(4),
        counter_cost(3),
        generic(5),
        counter_cost(6),
    ]),
    ..CardRules::EMPTY
};
