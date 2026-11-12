const FACE_DOWN_CREATURE: TargetPlan = one(TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::FaceDown,
});
const TURN_UP: Instruction = Instruction::TurnTargetFaceUp;
const SELECT_PILE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::ExileFaceUpDisguiseCreatures,
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 1,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 0,
    optional_skip: None,
};
const CLOAK: Instruction = Instruction::CloakShuffledRememberedExile;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Turn target face-down creature face up.",
            targets: FACE_DOWN_CREATURE,
            program: &[TURN_UP],
        },
        SpellMode {
            text: "Shuffle and cloak any number of your face-up creatures with disguise.",
            targets: TargetPlan::NONE,
            program: &[SELECT_PILE, CLOAK],
        },
        SpellMode {
            text: "Turn the target face up, then shuffle and cloak eligible creatures.",
            targets: FACE_DOWN_CREATURE,
            program: &[TURN_UP, SELECT_PILE, CLOAK],
        },
    ]),
    mode_counts: Some(&[1, 1, 2]),
    ..CardRules::EMPTY
};
