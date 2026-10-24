//! Cross-cohort paid resolution, draw triggers and simultaneous counter events.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn walls(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.identity() == "@token/msh-wall")
                .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn msh_fifth_combined_payment_second_draw_and_counter_batches_create_two_walls() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let sue = paid(&mut g, p, "Invisible Woman, Sue Storm");
            let king = paid(&mut g, p, "King T'Challa");
            let kid = paid(&mut g, p, "Kid Loki");
            let archive = paid(&mut g, p, "Hedron Archive");
            let photon = named(&mut g, p, "Monica Rambeau", Zone::Hand);
            checked(&mut g, Action::AnnounceBack { card: photon });
            while g.state.announcing.is_some() {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            let secret = named(&mut g, p, "Brave Brawler", Zone::Hand);
            let spell = named(&mut g, p, "Vision of Love", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            g.prepare_triggers();
            for _ in 0..80 {
                if g.observe(p).effect_choice.is_some() {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::FinishTargets
                                | Action::PayCosts { .. }
                                | Action::ChooseOptional { accept: true }
                        )
                    })
                    .unwrap();
                checked(&mut g, action);
            }
            let choice = g.observe(p).effect_choice.unwrap();
            assert!(matches!(
                choice.kind,
                EffectChoiceKind::SacrificeOrDiscard { nonland: false }
            ));
            assert_eq!(walls(&g, p).len(), 1); // Photon's whole multi-Hero batch, before payment.
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == secret));
            let action = if discard {
                Action::Discard { card: secret }
            } else {
                Action::Sacrifice {
                    object: g.object_ref(archive),
                }
            };
            checked(&mut g, action);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(if discard { secret } else { archive }).zone,
                Zone::Graveyard
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
            assert_eq!(g.counter_count(sue, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(king, CounterKind::PlusOne), 1);
            assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(photon, CounterKind::PlusOne), 0);
            assert_eq!(walls(&g, p).len(), 2); // Kid's independent second-draw placement.
            for id in [sue, king, kid] {
                assert!(g.keywords(id).contains(Keywords::HEXPROOF));
            }
            for id in walls(&g, p) {
                assert_eq!(g.counter_count(id, CounterKind::PlusOne), 0);
                assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
                assert!(g.keywords(id).contains(Keywords::DEFENDER));
            }
        }
    }
}
