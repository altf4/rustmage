// Oracle and official Star Trek release notes; pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(17))),
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
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: generic(3),
                skip: 1,
            },
            Instruction::TapAttached,
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::AttachedUntap(UntapRestriction::ControllerStep),
            StaticAbilityFragment::AttachedCannotBeBeamedUp,
        ],
    }],
    ..CardRules::EMPTY
};
