//! Optional graveyard exile pays one exact, owned reflexive event.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPECTER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileGraveyardCreaturesReflexive { ability: 1 },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 1,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 0,
                optional_skip: None,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowCreaturesScaled {
                filter: CreatureFilter {
                    relation: -1,
                    other: true,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                excluding_subtype: None,
                amount: AmountValue::ReflexiveSubjectCount,
                power: -1,
                toughness: -1,
            }],
        },
    ],
    ..CardRules::EMPTY
};

#[test]
fn decline_or_exile_one_or_more_with_independent_exact_reflexive_counts() {
    let _scope = fixture_rules::activate(&[(Card::HillGiant, &SPECTER)]);
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Hill Giant", Zone::Hand);
        let friend = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let foe = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        let a = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let b = put(&mut g, player, "Bear Cub", Zone::Graveyard);
        let noncreature = put(&mut g, player, "Forest", Zone::Graveyard);
        let opponent = put(&mut g, player.other(), "Bear Cub", Zone::Graveyard);
        cast(&mut g, source, None);
        for _ in 0..4 {
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::EffectChoice { player });
        let select = |object| Action::SelectResolutionExile { object };
        let first = select(g.object_ref(a));
        let second = select(g.object_ref(b));
        for forbidden in [noncreature, opponent].map(|id| select(g.object_ref(id))) {
            reject(&mut g, forbidden);
        }
        let checkpoint = g.snapshot();
        let mut decline = g.clone();
        act(&mut decline, Action::FinishEffectChoice);
        assert!(decline.state.stack.is_empty());
        assert_eq!(decline.object_view(a).zone, Zone::Graveyard);
        assert_eq!(decline.object_view(foe).power, 3);

        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|action| *action == first)
            .unwrap();
        let mut indexed = g.clone();
        indexed.apply_index(player, revision, index).unwrap();
        act(&mut g, first.clone());
        assert_eq!(g.snapshot(), indexed.snapshot());
        reject(&mut g, first);
        act(&mut g, second.clone());
        act(&mut g, Action::FinishEffectChoice);
        assert_eq!(g.state.stack.len(), 1);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(g.object_view(foe).power, 3);
        // The new ability owns its quantity. A later movement cannot reduce it.
        g.move_object(b, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.object_view(source).power, 3);
        assert_eq!(g.object_view(friend).power, 1);
        assert_eq!(g.object_view(foe).toughness, 1);

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, second);
        act(&mut indexed, Action::FinishEffectChoice);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(foe).toughness, 2);
        assert_eq!(indexed.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
    }
}
