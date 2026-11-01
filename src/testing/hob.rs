//! Small card-feature cases, without the implementation investigations.
use super::*;

#[test]
fn notary_creates_two_nonlegendary_copies_and_counts_halflings_for_mana() {
    let mut g = game();
    let source = put(&mut g, Player::P0, "The Notary Hobbits", Zone::Hand);
    cast(&mut g, source, &[]);
    resolve(&mut g);
    let copies: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.is_token()
                && o.card == Card::from_name("The Notary Hobbits").unwrap()
        })
        .collect();
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|o| !o.is_legendary()));
    assert!(copies.iter().all(|o| (o.power, o.toughness) == (1, 1)));
    // Arrange a ready source for its separate mana ability.
    g.state.objects[source as usize].summoning_sick = false;
    g.state.players[0].mana = [0; 6];
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana {card, production: [0,0,0,0,0,3], ..} if *card==source),
    );
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 0, 3]);
    assert!(g.object_view(source).tapped);
}

#[test]
fn elrond_draws_once_for_creature_activations_and_blinks_until_the_end_step() {
    let mut g = game();
    let elrond = put(&mut g, Player::P0, "Elrond, Moon-Reader", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 5];
    act(
        &mut g,
        Action::AnnounceActivate {
            card: elrond,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    act(&mut g, Action::FinishTargets);
    pay(&mut g);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Exile);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    let elf = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    act(&mut g, Action::TapMana { card: elf });
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    g.begin_step(Step::End);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
}

#[test]
fn bejeweled_warg_combat_trigger_can_put_a_counter_or_create_a_treasure() {
    for mode in 0..2 {
        let mut g = game();
        let warg = put(&mut g, Player::P0, "Bejeweled Warg", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, warg, &[]);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        resolve(&mut g);
        assert_eq!(stats(&g, warg), (3, 2));
        g.state.objects[warg as usize].summoning_sick = false;
        attack(&mut g, warg);
        reach(&mut g, &Action::FinishBlockers);
        act(&mut g, Action::FinishBlockers);
        reach(&mut g, &Action::ChooseMode { mode });
        assert_eq!(g.state.players[1].life, 17);
        act(&mut g, Action::ChooseMode { mode });
        if mode == 0 {
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(warg),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
        }
        resolve(&mut g);
        if mode == 0 {
            assert_eq!(g.counter_count(warg, CounterKind::PlusOne), 1);
            assert_eq!(stats(&g, warg), (4, 3));
        } else {
            let treasures: Vec<_> = g
                .observe(Player::P0)
                .objects
                .into_iter()
                .filter(|o| {
                    o.controller == Player::P0
                        && o.zone == Zone::Battlefield
                        && o.has_subtype("Treasure")
                })
                .collect();
            assert_eq!(treasures.len(), 1);
            assert_eq!(treasures[0].kinds, 16);
            assert!(!treasures[0].tapped);
        }
        covered(&format!("card/376/combat_mode/{mode}"));
    }
}
