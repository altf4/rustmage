// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(
            0,
            CombatRestriction {
                subject: Subject::Source,
                attack: false,
                block: true,
                unless: None,
            },
        )],
    }],
    extras: &CardExtras {
        combat_restriction: Some(CombatRestriction {
            subject: Subject::Source,
            attack: false,
            block: true,
            unless: None,
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        once_per_turn: false,
        sorcery: false,
        targets: TargetPlan::NONE,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 12,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::ReturnSourceToBattlefield { tapped: false }],
    }],
    ..CardRules::EMPTY
};
