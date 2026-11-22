use super::*;

fn crime(game: &mut Game, player: Player) {
    let shock = hand(game, player, "Shock");
    cast(game, shock, Some(Target::Player(player.other())));
}

#[test]
fn magda_and_raven_observe_the_first_crime_and_live_bonuses_update() {
    let player = Player::P0;
    let mut game = scenario(player);
    let magda = board(&mut game, player, "Magda, the Hoardmaster");
    let raven = board(&mut game, player, "Raven of Fell Omens");
    let brigand = board(&mut game, player, "Nimble Brigand");
    let vigilante = board(&mut game, player, "Omenport Vigilante");
    let muscle = board(&mut game, player, "Overzealous Muscle");
    assert!(keywords(&game, raven).contains(Keywords::FLYING));
    assert!(!keywords(&game, brigand).contains(Keywords::CANT_BE_BLOCKED));
    assert!(!keywords(&game, vigilante).contains(Keywords::DOUBLE_STRIKE));

    for _ in 0..2 {
        crime(&mut game, player);
        quiet(&mut game);
    }
    assert_eq!(life(&game, player), 21);
    assert_eq!(life(&game, player.other()), 15);
    assert!(keywords(&game, brigand).contains(Keywords::CANT_BE_BLOCKED));
    assert!(keywords(&game, vigilante).contains(Keywords::DOUBLE_STRIKE));
    assert!(keywords(&game, muscle).contains(Keywords::INDESTRUCTIBLE));

    let treasure = tokens(&game, player, TREASURE);
    assert_eq!(treasure.len(), 1);
    assert!(game.object_view(treasure[0]).tapped);
    let treasure_card = Card::from_catalogue_id(TREASURE);
    let second = spawn(&mut game, player, treasure_card, Zone::Battlefield);
    let third = spawn(&mut game, player, treasure_card, Zone::Battlefield);
    let fourth = spawn(&mut game, player, treasure_card, Zone::Battlefield);
    announce_activation(&mut game, magda, 0);
    reject(&mut game, Action::FinishAdditionalCostGroup);
    finish_additional_cost(&mut game, &[treasure[0], second, third]);
    assert_eq!(tokens(&game, player, TREASURE), vec![fourth]);
    quiet(&mut game);
    let dragon = tokens(&game, player, SCORPION_DRAGON)[0];
    assert_eq!(stats(&game, dragon), (4, 4));
    assert!(keywords(&game, dragon).contains(Keywords::FLYING.union(Keywords::HASTE)));

    reach(&mut game, |g| {
        g.state.active == player.other() && g.state.step == Step::Main1
    });
    assert!(!keywords(&game, brigand).contains(Keywords::CANT_BE_BLOCKED));
    assert!(!keywords(&game, vigilante).contains(Keywords::DOUBLE_STRIKE));
    assert!(!keywords(&game, muscle).contains(Keywords::INDESTRUCTIBLE));
    restock_mana(&mut game, player);
    act(&mut game, Action::Pass);
    crime(&mut game, player);
    quiet(&mut game);
    assert_eq!(life(&game, player), 22);
    assert!(keywords(&game, brigand).contains(Keywords::CANT_BE_BLOCKED));
    assert!(keywords(&game, vigilante).contains(Keywords::DOUBLE_STRIKE));
    assert!(!keywords(&game, muscle).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn magebane_uses_the_actual_casters_current_noncreature_count() {
    let player = Player::P0;
    let mut game = scenario(player);
    board(&mut game, player.other(), "Magebane Lizard");
    let razzle = board(&mut game, player, "Razzle-Dazzler");
    let first = hand(&mut game, player, "Grizzly Bears");
    cast(&mut game, first, None);
    quiet(&mut game);
    assert_eq!(life(&game, player), 20);
    assert_eq!(counters(&game, razzle), 0);

    crime(&mut game, player);
    reach(&mut game, |g| {
        g.state.pending == Pending::Priority && g.actor() == Some(player)
    });
    crime(&mut game, player);
    quiet(&mut game);
    assert_eq!(life(&game, player), 16);
    assert_eq!(life(&game, player.other()), 16);
    assert_eq!(counters(&game, razzle), 1);
    assert!(keywords(&game, razzle).contains(Keywords::CANT_BE_BLOCKED));
}

#[test]
fn marchesa_repeats_her_private_selection_and_apothecary_chooses_on_resolution() {
    let player = Player::P1;
    let mut game = scenario(player);
    board(&mut game, player, "Marchesa, Dealer of Death");
    let apothecary = board(&mut game, player, "Rattleback Apothecary");
    let recipient = board(&mut game, player, "Grizzly Bears");
    assert!(keywords(&game, apothecary).contains(Keywords::DEATHTOUCH));

    for (chosen_keyword, index) in [(Keywords::MENACE, 0), (Keywords::LIFELINK, 1)] {
        let buried = top(&mut game, player, "Island");
        let picked = top(&mut game, player, "Swamp");
        crime(&mut game, player);
        let target = Action::ChooseTarget {
            target: Target::Card(recipient),
        };
        until_action(&mut game, &target);
        act(&mut game, target);
        for _ in 0..24 {
            if game.state.pending == Pending::Priority && game.state.stack.is_empty() {
                break;
            }
            let keyword = Action::ChooseKeyword { index };
            if has(&game, &keyword) {
                act(&mut game, keyword);
            } else if has(&game, &Action::SelectLibrary { card: picked }) {
                assert!(game.observe(player.other()).library_selection.is_none());
                choose_library(&mut game, picked);
            } else {
                choose(&mut game, |action| {
                    matches!(
                        action,
                        Action::Pass | Action::PutTrigger { .. } | Action::PayResolutionMana { .. }
                    )
                });
            }
        }
        assert_eq!(zone(&game, picked), Zone::Hand);
        assert_eq!(zone(&game, buried), Zone::Graveyard);
        assert!(keywords(&game, recipient).contains(chosen_keyword));
    }
}

#[test]
fn marauding_sphinx_surveils_once_and_ward_requires_a_real_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let sphinx = board(&mut game, player, "Marauding Sphinx");
    assert!(keywords(&game, sphinx).contains(Keywords::FLYING.union(Keywords::VIGILANCE)));
    let buried = top(&mut game, player, "Forest");
    let kept = top(&mut game, player, "Island");
    crime(&mut game, player);
    reach(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectLibrary { .. })
    });
    arrange(&mut game, kept, LibraryDestination::Top);
    arrange(&mut game, buried, LibraryDestination::Graveyard);
    quiet(&mut game);
    crime(&mut game, player);
    quiet(&mut game);
    assert_eq!(zone(&game, kept), Zone::Library);
    assert_eq!(zone(&game, buried), Zone::Graveyard);

    act(&mut game, Action::Pass);
    let shock = hand(&mut game, player.other(), "Shock");
    cast(&mut game, shock, Some(Target::Card(sphinx)));
    until_action(&mut game, &Action::DeclineResolutionMana);
    let mut paid = game.clone();
    choose(&mut paid, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    quiet(&mut paid);
    assert_eq!(paid.object_view(sphinx).damage, 2);
    act(&mut game, Action::DeclineResolutionMana);
    quiet(&mut game);
    assert_eq!(game.object_view(sphinx).damage, 0);
    assert_eq!(zone(&game, shock), Zone::Graveyard);
}
