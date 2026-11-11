// The 16 legal multiplicities of the printed pawprint modes.
const CHOOSE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::RememberPermanents {
        require_battlefield_source: false,
    },
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        kinds: 1,
        keywords: Keywords::NONE,
        source_only: false,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 1,
    optional_skip: None,
};
const BOOST: [Instruction; 3] = [
    CHOOSE,
    Instruction::PutRememberedPermanentCounters {
        kind: CounterKind::PlusOne,
        amount: 1,
    },
    Instruction::GrantRememberedCreatures(Keywords::VIGILANCE.union(Keywords::TRAMPLE)),
];
const DESTROY: Instruction = Instruction::DestroyAllOfChosenPermanentType(&[
    rustmage_core::PermanentType::Artifact,
    rustmage_core::PermanentType::Enchantment,
]);
const DRAW: Instruction = Instruction::DrawScaled(
    PlayerSet::Controller,
    AmountValue::GreatestCreaturePower(CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    }),
);

const fn program<const N: usize>(
    boosts: usize,
    destructions: usize,
    draw: bool,
) -> [Instruction; N] {
    assert!(N == boosts * BOOST.len() + destructions + draw as usize);
    let mut program = [Instruction::Skip(0); N];
    let mut cursor = 0;
    while cursor < boosts * BOOST.len() {
        program[cursor] = BOOST[cursor % BOOST.len()];
        cursor += 1;
    }
    while cursor < boosts * BOOST.len() + destructions {
        program[cursor] = DESTROY;
        cursor += 1;
    }
    if draw {
        program[cursor] = DRAW;
    }
    program
}

const fn mode(text: &'static str, program: &'static [Instruction]) -> SpellMode {
    SpellMode {
        text,
        targets: TargetPlan::NONE,
        program,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        mode("Choose no modes.", &[]),
        mode("Counter and keywords.", &program::<3>(1, 0, false)),
        mode("Counter and keywords twice.", &program::<6>(2, 0, false)),
        mode(
            "Counter and keywords three times.",
            &program::<9>(3, 0, false),
        ),
        mode(
            "Counter and keywords four times.",
            &program::<12>(4, 0, false),
        ),
        mode(
            "Counter and keywords five times.",
            &program::<15>(5, 0, false),
        ),
        mode(
            "Choose a permanent type and destroy all of that type.",
            &[DESTROY],
        ),
        mode(
            "Counter and keywords, then destroy.",
            &program::<4>(1, 1, false),
        ),
        mode(
            "Counter and keywords twice, then destroy.",
            &program::<7>(2, 1, false),
        ),
        mode(
            "Counter and keywords three times, then destroy.",
            &program::<10>(3, 1, false),
        ),
        mode("Choose and destroy twice.", &[DESTROY, DESTROY]),
        mode(
            "Counter and keywords, then choose and destroy twice.",
            &program::<5>(1, 2, false),
        ),
        mode("Draw equal to greatest power.", &[DRAW]),
        mode(
            "Counter and keywords, then draw.",
            &program::<4>(1, 0, true),
        ),
        mode(
            "Counter and keywords twice, then draw.",
            &program::<7>(2, 0, true),
        ),
        mode("Choose and destroy, then draw.", &[DESTROY, DRAW]),
    ]),
    mode_counts: Some(&[0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 2, 3, 1, 2, 3, 2]),
    ..CardRules::EMPTY
};
