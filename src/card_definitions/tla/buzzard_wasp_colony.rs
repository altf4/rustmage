// Direct typed behavior from the pinned Oracle and XMage sources.
use crate::engine::Zone;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1 | 16,
                        nonland: false,
                        other: false,
                        tokens: None,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                subject: DepartureSubject::Other,
                destinations: &[Zone::Graveyard],
                counters: DepartureCounters::Any,
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutEventCounters {
                subject: Subject::Source,
            }],
        },
    ],
    ..CardRules::EMPTY
};
