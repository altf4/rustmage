// The conditional animation is checked during resolution, not activation.
const EQUIPPED: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::AttachmentStats(Stats {
            power: 2,
            toughness: 1,
        }),
        StaticAbilityFragment::AttachmentKeywords(Keywords::FLYING),
    ],
}];
const CREATURE_YOU_CONTROL: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(CREATURE_YOU_CONTROL),
        program: &[Instruction::AttachSource],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 2,
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
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSourceWithAbilityIfNoncreature {
                spec: AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Named(&["Construct", "Hero"]),
                    base: Some((0, 0)),
                    keywords: Keywords::FLYING,
                    duration: AnimationDuration::EndOfTurn,
                },
                kind: crate::engine::SelfGrantedAbilityKind::ControllerArtifactBonus,
            }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 2,
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
            targets: TargetPlan::Single(CREATURE_YOU_CONTROL),
            program: &[Instruction::AttachSource],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    static_groups: EQUIPPED,
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 2,
                toughness: 1,
            },
            keywords: Keywords::FLYING,
        }),
        static_ability_groups: EQUIPPED,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
