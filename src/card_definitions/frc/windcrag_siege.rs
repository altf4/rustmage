// Canonical reprint body; named choices qualify one exact acquired ability.
const MODES: &[&str] = &["Mardu", "Jeskai"];
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::EntryMode(0),
        StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledPermanentAttack,
            condition: StatValue::Fixed(1),
        },
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensWithTemporaryKeywords {
            token: Card(5324),
            amount: AmountValue::Fixed(1),
            keywords: Keywords::LIFELINK.union(Keywords::HASTE),
        }],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        entry_choice: Some(EntryChoiceKind::NamedModes(MODES)),
        entry_mode_triggers: &[(0, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
