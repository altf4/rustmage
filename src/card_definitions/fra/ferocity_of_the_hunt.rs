// Direct typed port; the return tracks the exact card produced by the death event.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 0,
            },
            keywords: Keywords::DEATHTOUCH,
        },
    }),
    keywords: Keywords::FLASH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedCreatureDies,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ReturnEventCardToBattlefieldTapped],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 0,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::DEATHTOUCH),
        ],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
