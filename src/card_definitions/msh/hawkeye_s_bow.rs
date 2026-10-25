// Complete Oracle body; the tapped event creature is the independent damage actor.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 0,
            }),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 0,
            },
            keywords: Keywords::REACH,
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedCreatureBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[Instruction::EventObjectDamagePlayers(
            PlayerSet::Opponent,
            1,
        )],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 1,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
