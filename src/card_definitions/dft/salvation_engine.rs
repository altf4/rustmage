// Direct typed behavior from complete pinned Oracle and XMage sources.
const BOOST: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        other: true,
        artifact: true,
        ..CreatureFilter {
            relation: 0,
            other: false,
            subtype: None,
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        }
    },
    stats: Stats {
        power: 2,
        toughness: 2,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(0, BOOST)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec {
            min: 0,
            max: 1,
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        }),
        program: &[Instruction::ReturnToBattlefieldUnderControl],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 6 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 10,
        }),
        static_modifiers: &[BOOST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
