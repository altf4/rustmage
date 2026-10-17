// Target controller is bound to the combat damage event, independent of later source control.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::DamagedPlayerPermanent { kinds: 16 | 32 },
            min: 1,
            max: 1,
        }),
        program: &[Instruction::May { skip: 1 }, Instruction::Destroy],
    }],
    ..CardRules::EMPTY
};
