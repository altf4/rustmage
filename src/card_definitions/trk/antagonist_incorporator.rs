// Pinned Oracle and full XMage class: each controlled artifact creature owns
// the combat-damage trigger, including this source and artifact creature tokens.
const PLAYER_HIT: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Source,
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledArtifactCreatureTrigger { ability: 0 }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&PLAYER_HIT),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: generic(1),
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(6007),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
