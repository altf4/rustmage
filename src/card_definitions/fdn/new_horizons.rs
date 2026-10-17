pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(2))),
    aura: Some(AuraSpec {
        kinds: 2,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentMana(
            ManaProduction::Choice {
                mask: 31,
                amount: 2,
            },
        )],
    }],
    ..CardRules::EMPTY
};
