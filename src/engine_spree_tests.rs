//! Spree subsets reuse owned modal announcements and summed additional costs.
use super::*;

#[test]
fn all_spree_subsets_pay_the_sum_and_bind_independent_target_groups_both_seats() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3u16 {
            let mut g = setup(p);
            let q = p.other();
            let bear = named(&mut g, q, "Bear Cub", Zone::Battlefield);
            let artifact = named(&mut g, q, "Gilded Lotus", Zone::Battlefield);
            let id = named(&mut g, p, "Explosive Derailment", Zone::Hand);
            let targets = match mode {
                0 => vec![Target::Card(bear)],
                1 => vec![Target::Card(artifact)],
                _ => vec![Target::Card(bear), Target::Card(artifact)],
            };
            cast(&mut g, id, Some(mode), &targets);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.mana_value(), 1);
            assert_eq!(item.targets.len(), if mode == 2 { 2 } else { 1 });
            assert_eq!(
                mana_sum(item.cast.unwrap().payment),
                if mode == 2 { 5 } else { 3 }
            );
            settle(&mut g);
            assert_eq!(g.on_battlefield(bear), mode == 1);
            assert_eq!(g.on_battlefield(artifact), mode == 0);
        }
    }
}

#[test]
fn phantom_targetless_counter_and_combined_subsets_are_complete_both_seats() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3u16 {
            let mut g = setup(p);
            let q = p.other();
            let spell = named(&mut g, p, "Opt", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            let target = Target::Spell(spell);
            let id = named(&mut g, p, "Phantom Interference", Zone::Hand);
            let targets = if mode == 0 { vec![] } else { vec![target] };
            cast(&mut g, id, Some(mode), &targets);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.mana_value(), 2);
            assert_eq!(
                mana_sum(item.cast.unwrap().payment),
                [5, 3, 6][mode as usize]
            );
            g.state.players[p.index()].mana = [0; 6];
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(i, o)| {
                    o.controller == p && g.on_battlefield(*i as CardId) && o.card.is_token()
                })
                .map(|(i, _)| i as CardId)
                .collect();
            assert_eq!(tokens.len(), usize::from(mode != 1));
            for token in tokens {
                assert_eq!(
                    g.stats(token),
                    Stats {
                        power: 2,
                        toughness: 2
                    }
                );
                assert!(g.keywords(token).contains(Keywords::FLYING));
            }
            assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
            let _ = q;
        }
    }
}

#[test]
fn free_base_spree_still_requires_nonempty_distinct_subsets_and_additional_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let id = named(&mut g, p, "Explosive Derailment", Zone::Exile);
        let object = g.object_ref(id);
        let turn = g.state.turn;
        g.state
            .plot
            .get_or_insert_with(Default::default)
            .cards
            .push(PlottedCard {
                object,
                owner: p,
                plotted_turn: turn - 1,
            });
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnouncePlotted { card: id });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
        checked(&mut g, Action::AnnouncePlotted { card: id });
        assert_eq!(g.observe(p).mode_choices.len(), 3);
        assert!(g.observe(p.other()).mode_choices.is_empty());
        reject(&mut g, Action::ChooseMode { mode: 3 });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        reject(&mut g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        checked(&mut g, Action::FinishTargets);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let request = g.observe(p).payment_request.unwrap();
        assert_eq!(request.required, [0; 6]);
        assert_eq!(request.generic, 4);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.state.stack.last().unwrap().mana_value(), 1);
        settle(&mut g);
        assert!(!g.on_battlefield(target));
        assert!(!g.on_battlefield(artifact));
    }
}
