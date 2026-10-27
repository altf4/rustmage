// Direct typed behavior from pinned Oracle and XMage sources.
const CONDITION: StatValue = StatValue::AtLeast {
    value: &StatValue::LinkedExileCardTypes,
    minimum: 4,
};
const MODIFIER: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: CONDITION,
    stats: Stats {
        power: 4,
        toughness: 4,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: CONDITION,
                keywords: Keywords::TRAMPLE,
            },
            StaticAbilityFragment::ScalingModifier(0, MODIFIER),
        ],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[MODIFIER],
        ..CardExtras::EMPTY
    },
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
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ExileGraveTargetsLinked],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
