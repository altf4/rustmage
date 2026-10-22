// Direct typed behavior from complete pinned Oracle and XMage sources.
const EXILE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::ExileFromHand, players: PlayerSet::Targeted,
    filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 0, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None },
    amount: 2, optional_skip: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode { text: "Target opponent exiles two cards from their hand.", targets: one(TargetRule::Player { opponent_only: true }), program: &[EXILE] },
        SpellMode { text: "Remove all counters from target creature.", targets: one(TargetRule::Creature), program: &[Instruction::RemoveAllCounters] },
        SpellMode { text: "Choose both.", targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player { opponent_only: true }), TargetSpec::one(TargetRule::Creature),
        ]), program: &[Instruction::SelectGroup(0), EXILE, Instruction::SelectGroup(1), Instruction::RemoveAllCounters] },
    ]),
    ..CardRules::EMPTY
};
