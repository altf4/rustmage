const TUTOR: Instruction = Instruction::SearchRequired(SearchSpec {
    filter: SearchFilter::Any,
    maximum: 1,
    destination: SearchDestination::Top,
    tapped: false,
    reveal: false,
});
const DRAW: Instruction = Instruction::Draw(PlayerSet::Targeted, 3);
const LOSE_LIFE: Instruction = Instruction::LoseLife(PlayerSet::Targeted, 3);
const PLAYER: TargetPlan = one(TargetRule::Player {
    opponent_only: false,
});
const fn life_mode(generic: u32) -> ManaCost {
    ManaCost {
        colored: [0, 0, 2, 0, 0, 0],
        generic,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Search for a card and put it on top",
            targets: TargetPlan::NONE,
            program: &[TUTOR],
        },
        SpellMode {
            text: "Target player draws three and loses 3 life",
            targets: PLAYER,
            program: &[DRAW, LOSE_LIFE],
        },
        SpellMode {
            text: "Search for a card + Target player draws three and loses 3 life",
            targets: PLAYER,
            program: &[TUTOR, DRAW, LOSE_LIFE],
        },
    ]),
    mode_costs: Some(&[generic(2), life_mode(0), life_mode(2)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
