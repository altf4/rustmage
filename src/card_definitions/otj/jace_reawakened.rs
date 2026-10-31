// Complete Oracle body and pinned XMage behavior; see paired provenance.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
const NO_FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 3,
        cast_after_first_three_turns: true,
        ..CardExtras::EMPTY
    },
    // A delayed trigger, created by the ultimate, is not a battlefield ability.
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Discard,
                    players: PlayerSet::Controller,
                    filter: NO_FILTER,
                    amount: 1,
                    optional_skip: None,
                },
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::HandPlot { maximum: 3 },
                players: PlayerSet::Controller,
                filter: NO_FILTER,
                amount: 1,
                optional_skip: Some(0),
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-6),
            targets: TargetPlan::NONE,
            program: &[Instruction::CopySpellsThisTurn { ability: 0 }],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
