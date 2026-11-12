const DRAW: Instruction = Instruction::Draw(PlayerSet::Controller, 1);
const CHOOSE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::RememberPermanents {
        require_battlefield_source: false,
    },
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 1 | 16,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 1,
    optional_skip: None,
};
const COPY: Instruction = Instruction::CreateCopyOfRememberedPermanent;
const BOUNCE: Instruction = Instruction::ReturnNonlandNontokenPermanents;

// The printed costs are one, two, and three pawprints, with a budget of five.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Choose no modes",
            targets: TargetPlan::NONE,
            program: &[],
        },
        SpellMode {
            text: "Draw 1 card",
            targets: TargetPlan::NONE,
            program: &[DRAW],
        },
        SpellMode {
            text: "Draw 2 cards",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW],
        },
        SpellMode {
            text: "Draw 3 cards",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW, DRAW],
        },
        SpellMode {
            text: "Draw 4 cards",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW, DRAW, DRAW],
        },
        SpellMode {
            text: "Draw 5 cards",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW, DRAW, DRAW, DRAW],
        },
        SpellMode {
            text: "Copy one artifact or creature",
            targets: TargetPlan::NONE,
            program: &[CHOOSE, COPY],
        },
        SpellMode {
            text: "Draw 1 card, then Copy one artifact or creature",
            targets: TargetPlan::NONE,
            program: &[DRAW, CHOOSE, COPY],
        },
        SpellMode {
            text: "Draw 2 cards, then Copy one artifact or creature",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW, CHOOSE, COPY],
        },
        SpellMode {
            text: "Draw 3 cards, then Copy one artifact or creature",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW, DRAW, CHOOSE, COPY],
        },
        SpellMode {
            text: "Copy two artifacts or creatures",
            targets: TargetPlan::NONE,
            program: &[CHOOSE, COPY, CHOOSE, COPY],
        },
        SpellMode {
            text: "Draw 1 card, then Copy two artifacts or creatures",
            targets: TargetPlan::NONE,
            program: &[DRAW, CHOOSE, COPY, CHOOSE, COPY],
        },
        SpellMode {
            text: "Return nonland nontoken permanents to hand",
            targets: TargetPlan::NONE,
            program: &[BOUNCE],
        },
        SpellMode {
            text: "Draw 1 card, then Return nonland nontoken permanents to hand",
            targets: TargetPlan::NONE,
            program: &[DRAW, BOUNCE],
        },
        SpellMode {
            text: "Draw 2 cards, then Return nonland nontoken permanents to hand",
            targets: TargetPlan::NONE,
            program: &[DRAW, DRAW, BOUNCE],
        },
        SpellMode {
            text: "Copy one artifact or creature, then Return nonland nontoken permanents to hand",
            targets: TargetPlan::NONE,
            program: &[CHOOSE, COPY, BOUNCE],
        },
    ]),
    mode_counts: Some(&[0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 2, 3, 1, 2, 3, 2]),
    ..CardRules::EMPTY
};
