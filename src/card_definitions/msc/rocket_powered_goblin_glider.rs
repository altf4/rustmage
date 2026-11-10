// Oracle and pinned XMage evidence are recorded in the paired provenance.
const EQUIPPED: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 0,
    },
    keywords: Keywords::FLYING.union(Keywords::HASTE),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCastFromYourGraveyard,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: true,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(EQUIPPED.stats),
            StaticAbilityFragment::AttachmentKeywords(EQUIPPED.keywords),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EQUIPPED),
        mayhem_cost: Some(generic(2)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
