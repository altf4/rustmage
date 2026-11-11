const EQUIPMENT: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 0,
        toughness: 0,
    },
    keywords: Keywords::VIGILANCE,
};
const GRANT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::AttachmentKeywords(
        Keywords::VIGILANCE,
    )],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GRANT,
    extras: &CardExtras {
        equipment: Some(EQUIPMENT),
        static_ability_groups: GRANT,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentToGraveyard {
            kinds: 1 | 16,
            relation: CastRelation::Any,
            exclude_source_kinds: 1 | 16,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Attached,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
