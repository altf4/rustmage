const OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: OWN,
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const GRANT: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: OWN,
    source_only: false,
    own_turn: false,
    keywords: Keywords::DEATHTOUCH,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::KeywordGrant(0, GRANT),
        StaticAbilityFragment::StaticModifier(0, BONUS),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::YourMainPhase,
                event: &TriggerCondition::SelfEnters,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CombatAndMainAfterCurrentMain(1),
                Instruction::ScheduleTurnTrigger {
                    event: TurnCallbackEvent::Attack,
                    ability: 1,
                    once: true,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::DelayedTargetEvent,
            targets: TargetPlan::NONE,
            program: &[Instruction::UntapPlayerCreatures(PlayerSet::Controller)],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        static_modifiers: &[BONUS],
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
