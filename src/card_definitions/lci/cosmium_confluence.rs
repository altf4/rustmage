const SEARCH: Instruction = Instruction::Search(SearchSpec {
    filter: SearchFilter::Subtype("Cave"),
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
});
const CHOOSE_CAVE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::RememberPermanents {
        require_battlefield_source: false,
    },
    players: PlayerSet::Controller,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 2,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: Some("Cave"),
    },
    amount: 1,
    optional_skip: None,
};
const COUNTERS: Instruction = Instruction::PutRememberedPermanentCounters {
    kind: CounterKind::PlusOne,
    amount: 3,
};
const ANIMATE: Instruction = Instruction::AnimateRememberedPermanents(AnimationSpec {
    artifact: false,
    creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
    base: Some((0, 0)),
    keywords: Keywords::HASTE,
    duration: AnimationDuration::Permanent,
});
const ENCHANTMENT: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 32,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const DESTROY: Instruction = Instruction::Destroy;
const NEXT_TARGET: Instruction = Instruction::SelectGroup(1);
const LAST_TARGET: Instruction = Instruction::SelectGroup(2);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Find three Caves",
            targets: TargetPlan::NONE,
            program: &[SEARCH, SEARCH, SEARCH],
        },
        SpellMode {
            text: "Find two Caves and animate a Cave",
            targets: TargetPlan::NONE,
            program: &[SEARCH, SEARCH, CHOOSE_CAVE, COUNTERS, ANIMATE],
        },
        SpellMode {
            text: "Find two Caves and destroy an enchantment",
            targets: TargetPlan::Single(ENCHANTMENT),
            program: &[SEARCH, SEARCH, DESTROY],
        },
        SpellMode {
            text: "Find a Cave and animate two Caves",
            targets: TargetPlan::NONE,
            program: &[
                SEARCH,
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
            ],
        },
        SpellMode {
            text: "Find a Cave, animate a Cave, and destroy an enchantment",
            targets: TargetPlan::Single(ENCHANTMENT),
            program: &[SEARCH, CHOOSE_CAVE, COUNTERS, ANIMATE, DESTROY],
        },
        SpellMode {
            text: "Find a Cave and destroy two enchantments",
            targets: TargetPlan::Groups(&[ENCHANTMENT, ENCHANTMENT]),
            program: &[SEARCH, DESTROY, NEXT_TARGET, DESTROY],
        },
        SpellMode {
            text: "Animate a Cave three times",
            targets: TargetPlan::NONE,
            program: &[
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
            ],
        },
        SpellMode {
            text: "Animate a Cave twice and destroy an enchantment",
            targets: TargetPlan::Single(ENCHANTMENT),
            program: &[
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
                DESTROY,
            ],
        },
        SpellMode {
            text: "Animate a Cave and destroy two enchantments",
            targets: TargetPlan::Groups(&[ENCHANTMENT, ENCHANTMENT]),
            program: &[
                CHOOSE_CAVE,
                COUNTERS,
                ANIMATE,
                DESTROY,
                NEXT_TARGET,
                DESTROY,
            ],
        },
        SpellMode {
            text: "Destroy three enchantments",
            targets: TargetPlan::Groups(&[ENCHANTMENT, ENCHANTMENT, ENCHANTMENT]),
            program: &[DESTROY, NEXT_TARGET, DESTROY, LAST_TARGET, DESTROY],
        },
    ]),
    mode_counts: Some(&[3; 10]),
    ..CardRules::EMPTY
};
