const SACRIFICE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::Sacrifice,
    players: PlayerSet::Each,
    filter: SacrificeFilter {
        kinds: 1,
        other: false,
        keywords: Keywords::NONE,
        source_only: false,
        nonland: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 1,
    optional_skip: None,
};
const DRAW: Instruction = Instruction::DrawScaled(
    PlayerSet::Controller,
    AmountValue::Stat(StatValue::ControlledCreaturesDiedThisTurn),
);
const LIFE_LOSS: Instruction = Instruction::LoseLifeScaled(
    PlayerSet::Opponent,
    AmountValue::Stat(StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 1,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    })),
);

const fn mode(text: &'static str, program: &'static [Instruction]) -> SpellMode {
    SpellMode {
        text,
        targets: TargetPlan::NONE,
        program,
    }
}

// One pawprint for each sacrifice instruction, two for each draw, three for life loss.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        mode("Choose no modes.", &[]),
        mode("Each player sacrifices a creature.", &[SACRIFICE]),
        mode(
            "Each player sacrifices a creature twice.",
            &[SACRIFICE, SACRIFICE],
        ),
        mode(
            "Each player sacrifices a creature three times.",
            &[SACRIFICE, SACRIFICE, SACRIFICE],
        ),
        mode(
            "Each player sacrifices a creature four times.",
            &[SACRIFICE, SACRIFICE, SACRIFICE, SACRIFICE],
        ),
        mode(
            "Each player sacrifices a creature five times.",
            &[SACRIFICE, SACRIFICE, SACRIFICE, SACRIFICE, SACRIFICE],
        ),
        mode("Draw for your creatures that died this turn.", &[DRAW]),
        mode(
            "Sacrifice once, then draw for your creatures that died this turn.",
            &[SACRIFICE, DRAW],
        ),
        mode(
            "Sacrifice twice, then draw for your creatures that died this turn.",
            &[SACRIFICE, SACRIFICE, DRAW],
        ),
        mode(
            "Sacrifice three times, then draw for your creatures that died this turn.",
            &[SACRIFICE, SACRIFICE, SACRIFICE, DRAW],
        ),
        mode(
            "Draw twice for your creatures that died this turn.",
            &[DRAW, DRAW],
        ),
        mode(
            "Sacrifice once, then draw twice for your creatures that died this turn.",
            &[SACRIFICE, DRAW, DRAW],
        ),
        mode(
            "Each opponent loses life for your creature cards in graveyard.",
            &[LIFE_LOSS],
        ),
        mode(
            "Sacrifice once, then each opponent loses life for your creature cards in graveyard.",
            &[SACRIFICE, LIFE_LOSS],
        ),
        mode(
            "Sacrifice twice, then each opponent loses life for your creature cards in graveyard.",
            &[SACRIFICE, SACRIFICE, LIFE_LOSS],
        ),
        mode(
            "Draw for your creatures that died this turn, then each opponent loses life for your creature cards in graveyard.",
            &[DRAW, LIFE_LOSS],
        ),
    ]),
    mode_counts: Some(&[0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 2, 3, 1, 2, 3, 2]),
    ..CardRules::EMPTY
};
