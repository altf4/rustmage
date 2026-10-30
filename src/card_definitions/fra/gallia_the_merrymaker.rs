// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::HASTE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::CounterKeywordOther(
                CounterKind::PlusOne,
                Keywords::HASTE,
            )],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::EnteredBattlefieldThisTurn,
        }),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
