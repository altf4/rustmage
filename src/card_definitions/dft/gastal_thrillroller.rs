// Complete Oracle/XMage: temporary entry animation, ordinary Crew and finality on graveyard return.
const KEYS: Keywords = Keywords::HASTE.union(Keywords::TRAMPLE);
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYS)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: KEYS,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::AnimateSource(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 2 }],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Graveyard,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                colored: [0, 0, 0, 1, 0, 0],
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Discard,
                    filter: SacrificeFilter {
                        other: false,
                        ..SacrificeFilter {
                            keywords: Keywords::NONE,
                            source_only: false,
                            kinds: 0,
                            nonland: false,
                            other: true,
                            tokens: None,
                            artifact_enchantment_or_token: false,
                            subtype: None,
                        }
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceToBattlefieldWithCounter {
                tapped: false,
                counter: CounterKind::Finality,
            }],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 2,
        }),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
