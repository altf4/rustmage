const LAND_AURA: AuraSpec = AuraSpec {
    kinds: 2,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 0,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(2))),
    aura: Some(LAND_AURA),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5314),
            amount: 3,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
            power: 1,
            toughness: 1,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
