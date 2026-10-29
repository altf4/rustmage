// Incomplete gated draft: the static replacement and sacrifice ability are implemented; other loyalty abilities are pending.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 5,
        zone_replacement: Some(ZoneReplacementRule::ExileOpposingCreatureDeathPlain),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, once_per_turn: false, sorcery: true,
        cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
            mana_component: false, tap: false, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: Some(-2) },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects { kind: EffectChoiceKind::Sacrifice, players: PlayerSet::Each,
                filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1,
                    nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None },
                amount: 1, optional_skip: None },
            Instruction::CreateTokenIfControllerSacrificed(Card(5239)),
        ],
    }],
    ..CardRules::EMPTY
};
