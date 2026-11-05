mod support;
use rustmage as _;
use rustmage_core::{
    PermanentType,
    cards::{Keywords, TargetPredicate, TargetRule},
    engine::*,
};
use support::*;

#[test]
fn cast_discount_excludes_the_exact_departing_spell_in_both_counted_zones() {
    let p = Player::P0;
    for origin in [Zone::Graveyard, Zone::Exile] {
        let mut g = scenario(p);
        let swarm = put(&mut g, p, "Huskburster Swarm", origin);
        let action = if origin == Zone::Graveyard {
            let muldrotha = put(&mut g, p, "Muldrotha, the Gravetide", Zone::Battlefield);
            Action::PlayGraveyardPermanent {
                card: swarm,
                permission: g.object_ref(muldrotha),
                permanent_type: PermanentType::Creature,
            }
        } else {
            g.grant_exile_play_permission(p, g.object_ref(swarm), false);
            Action::AnnounceExile { card: swarm }
        };
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 6];
        reject(&mut g, action.clone());

        let mut discounted = g.clone();
        // Another copy with the same name and origin still contributes.
        put(&mut discounted, p, "Huskburster Swarm", origin);
        act(&mut discounted, action.clone());
        pay(&mut discounted);
        assert_eq!(discounted.state.players[p.index()].mana, [0; 6]);
        assert_eq!(discounted.object_view(swarm).zone, Zone::Stack);

        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 7];
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.object_view(swarm).zone, Zone::Battlefield);
    }
}

#[test]
fn cast_target_preflight_projects_the_departure_for_a_live_power_filter() {
    let p = Player::P0;
    let mut game = scenario(p);
    let drake = put(&mut game, p, "Enigma Drake", Zone::Battlefield);
    let flashback = put(&mut game, p, "Think Twice", Zone::Graveyard);
    assert_eq!(game.object_view(drake).power, 1);
    let source = game.object_ref(flashback);
    let target = Target::Card(drake);
    let rule = TargetRule::Filtered {
        base: &TargetRule::Creature,
        predicate: &TargetPredicate::PowerAtMost(0),
    };

    let definition = game.object_view(flashback).card;
    assert!(!game.activation_target_matches(rule, p, source, target));
    assert!(game.cast_target_matches(rule, p, source, definition, target));
    assert!(game.visit_cast_targets(rule, p, source, definition, |candidate| candidate == target));
    assert_eq!(game.object_view(drake).power, 1);
}

#[test]
fn harmonize_uses_the_power_after_the_spell_leaves_its_graveyard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let drake = put(&mut game, player, "Enigma Drake", Zone::Battlefield);
    let spell = put(&mut game, player, "Wild Ride", Zone::Graveyard);
    for _ in 0..3 {
        put(&mut game, player, "Opt", Zone::Graveyard);
    }
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    let announce = Action::AnnounceHarmonize { card: spell };
    assert_eq!(game.object_view(drake).power, 4);
    reject(&mut game, announce.clone());

    put(&mut game, player, "Opt", Zone::Graveyard);
    act(&mut game, announce);
    assert_eq!(game.object_view(drake).power, 4);
    let object = Some(game.object_ref(drake));
    act(&mut game, Action::ChooseHarmonizer { object });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(drake),
        },
    );
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    settle(&mut game);
    assert!(game.object_view(drake).tapped);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}

#[test]
fn graveyard_aura_discount_uses_its_targets_post_departure_keywords() {
    let player = Player::P0;
    let mut game = scenario(player);
    let muldrotha = put(
        &mut game,
        player,
        "Muldrotha, the Gravetide",
        Zone::Battlefield,
    );
    let theorix = put(
        &mut game,
        player,
        "Theorix Metamage // Omit Variables",
        Zone::Battlefield,
    );
    let trap = put(&mut game, player, "Swampsnare Trap", Zone::Graveyard);
    for _ in 0..6 {
        put(&mut game, player, "Forest", Zone::Graveyard);
    }
    let announce = Action::PlayGraveyardPermanent {
        card: trap,
        permission: game.object_ref(muldrotha),
        permanent_type: PermanentType::Enchantment,
    };
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 1];
    assert!(
        game.object_view(theorix)
            .keywords
            .contains(Keywords::FLYING)
    );
    reject(&mut game, announce.clone());

    put(&mut game, player, "Forest", Zone::Graveyard);
    act(&mut game, announce);
    assert!(
        game.object_view(theorix)
            .keywords
            .contains(Keywords::FLYING)
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(theorix),
        },
    );
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(trap).zone, Zone::Stack);
}

#[test]
fn a_flashback_cast_counts_its_own_graveyard_departure_for_a_history_discount() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Wilt in the Heat", Zone::Graveyard);
    let mage = put(&mut game, player, "Stingcaster Mage", Zone::Hand);
    cast(&mut game, mage, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(spell),
        },
    );
    settle(&mut game);
    assert!(!game.state.players[player.index()].card_left_graveyard_this_turn);
    game.state.players[player.index()].mana = [1, 0, 0, 1, 0, 0];
    act(
        &mut game,
        Action::AnnounceGrantedFlashback {
            card: spell,
            flash_surcharge: false,
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(mage),
        },
    );
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}
