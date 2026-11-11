// Complete typed behavior from the pinned Oracle and XMage implementation.
const PLAYER: TargetRule = TargetRule::Player {
    opponent_only: false,
};
const RABBIT: Instruction = Instruction::ChangeTargetCreatureForm(AnimationSpec {
    artifact: false,
    creature_types: AnimationCreatureTypes::Named(&["Rabbit"]),
    base: Some((0, 1)),
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
});
const WHITE: Instruction = Instruction::SetColors {
    subject: Subject::Targets,
    colors: ColorValue::Fixed(1),
    duration: AnimationDuration::EndOfTurn,
};
const DRAW: Instruction = Instruction::Draw(PlayerSet::Targeted, 2);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Turn target creature into a white 0/1 Rabbit",
            targets: one(TargetRule::Creature),
            program: &[RABBIT, WHITE],
        },
        SpellMode {
            text: "Target player draws two cards",
            targets: one(PLAYER),
            program: &[DRAW],
        },
        SpellMode {
            text: "Turn target creature into a white 0/1 Rabbit and draw two cards",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Creature),
                TargetSpec::one(PLAYER),
            ]),
            program: &[RABBIT, WHITE, Instruction::SelectGroup(1), DRAW],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(3), generic(4)]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
