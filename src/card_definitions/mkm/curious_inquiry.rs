// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 1,
    },
    keywords: Keywords::NONE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: BONUS,
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5296),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(BONUS.stats),
            StaticAbilityFragment::AttachmentTrigger(0),
        ],
    }],
    ..CardRules::EMPTY
};
