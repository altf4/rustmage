// Incomplete gated draft: the static replacement, +2 and -2 are implemented; the -3 is pending.
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
            discard: false, life: 0, remove: None, loyalty: Some(2) },
        targets: TargetPlan::Single(TargetSpec { rule: TargetRule::Object {
            zone: TargetZone::Battlefield, kinds: 1, relation: -1, other: false,
            nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX,
        }, min: 0, max: 1 }),
        program: &[Instruction::TargetStatsUntilNextTurn { power: -4, toughness: -1 }],
    }, ActivatedAbility {
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
