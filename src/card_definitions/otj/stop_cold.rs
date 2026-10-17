// Flash Aura that can enchant an artifact or creature; only the controller's untap step is prohibited.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(17))),
    keywords: Keywords::FLASH,
    aura: Some(AuraSpec {
        kinds: 17,
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
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::TapAttached],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep),
        ],
    }],
    ..CardRules::EMPTY
};
