// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::DamagedPlayerObject {
                zone: TargetZone::Graveyard,
                kinds: 1,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ReturnToBattlefieldAttacking(
            PlayerSet::DamagedPlayer,
        )],
    }],
    keywords: Keywords::FIRST_STRIKE,
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 3,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
