//! Actual A-range Outlaws definitions, including owned choices and their costs.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn tokens_with(game: &Game, player: Player, subtype: &str) -> Vec<CardId> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && o.has_subtype(subtype)
        })
        .map(|o| o.id)
        .collect()
}

#[test]
fn akul_requires_three_other_creatures_and_an_exact_optional_hand_choice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let akul = put(&mut game, player, "Akul the Unrepentant", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let from_hand = put(&mut game, player, "Daring Thunder-Thief", Zone::Hand);
    let action = activation(akul, 0);
    reject(&mut game, action.clone());
    let third = put(&mut game, player, "Ankle Biter", Zone::Battlefield);
    assert!(
        game.keywords(akul)
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
    act(&mut game, action.clone());
    for id in [first, second, third] {
        let object = game.object_ref(id);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(&mut game, Action::FinishAdditionalCostGroup);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    for card in [first, second, third] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    pass_pair(&mut game);
    let land = game.object_ref(land);
    let deployed = game.object_ref(from_hand);
    reject(&mut game, Action::PutHandCreature { object: land });
    assert!(game.observe(player.other()).effect_choice.is_none());
    let mut decline = game.clone();
    act(&mut decline, Action::FinishHandSelection);
    assert_eq!(decline.object_view(from_hand).zone, Zone::Hand);
    act(&mut game, Action::PutHandCreature { object: deployed });
    assert_eq!(game.object_view(from_hand).zone, Zone::Battlefield);
    for _ in 0..3 {
        put(&mut game, player, "Ankle Biter", Zone::Battlefield);
    }
    reject(&mut game, action);
}

#[test]
fn aloe_triggers_on_the_paid_plot_and_the_same_turn_free_cast_is_illegal() {
    let player = Player::P1;
    let mut game = scenario(player);
    let aloe = put(&mut game, player, "Aloe Alchemist", Zone::Hand);
    let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnouncePlot { card: aloe });
    let payment = game.observe(player).plot_payment.unwrap();
    assert_eq!(
        payment.cost,
        ManaCost {
            generic: 1,
            colored: [0, 0, 0, 0, 1, 0]
        }
    );
    assert!(game.observe(player.other()).plot_payment.is_none());
    choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
    choose_target(&mut game, Target::Card(creature));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (6, 5));
    assert!(game.keywords(creature).contains(Keywords::TRAMPLE));
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 0);
    let plot = Action::AnnouncePlotted { card: aloe };
    reject(&mut game, plot.clone());
    game.state.turn += 1;
    act(&mut game, plot);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Plot
    );
    settle(&mut game);
    assert!(game.keywords(aloe).contains(Keywords::TRAMPLE));
}

#[test]
fn annie_flash_returns_only_a_small_owned_permanent_and_tapping_grants_play() {
    let player = Player::P0;
    let mut game = scenario(player);
    let annie = put(&mut game, player, "Annie Flash, the Veteran", Zone::Hand);
    let equipment = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let nonpermanent = put(&mut game, player, "Murder", Zone::Graveyard);
    let costly = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let opposing = put(&mut game, player.other(), "Short Bow", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Library);

    let mut uncast = game.clone();
    uncast.move_object(annie, Zone::Battlefield);
    uncast.prepare_triggers();
    assert!(uncast.state.stack.is_empty());

    game.state.active = player.other();
    game.state.step = Step::Upkeep;
    cast(&mut game, annie, None);
    pass_pair(&mut game);
    for card in [nonpermanent, costly, opposing] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    choose_target(&mut game, Target::Card(equipment));
    settle(&mut game);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert!(game.object_view(equipment).tapped);
    game.tap_objects_as(player, &[annie]);
    game.give_priority();
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert!(
        game.state
            .exile_permissions
            .iter()
            .any(|p| p.object == game.object_ref(land)
                && p.player == player
                && !p.cast_only
                && p.expiry == ExilePlayExpiry::Turn(game.state.turn))
    );
}

#[test]
fn annie_joins_up_deals_five_and_doubles_a_real_legendary_creature_trigger() {
    let player = Player::P1;
    let mut game = scenario(player);
    let joins = put(&mut game, player, "Annie Joins Up", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let annie = put(
        &mut game,
        player,
        "Annie Flash, the Veteran",
        Zone::Battlefield,
    );
    cast(&mut game, joins, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    choose_target(&mut game, Target::Card(victim));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    game.tap_objects_as(player, &[annie]);
    game.give_priority();
    game.prepare_triggers();
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }));
    }
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.state.exile_permissions.len(), 4);
}

#[test]
fn another_round_charges_two_mana_per_x_and_repeats_each_owned_blink_choice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let round = put(&mut game, player, "Another Round", Zone::Hand);
    let ours = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let stolen = put(&mut game, player.other(), "Ankle Biter", Zone::Battlefield);
    let old_ours = game.object_ref(ours);
    let old_stolen = game.object_ref(stolen);
    game.record_control(old_stolen, player, None);
    act(&mut game, Action::AnnounceCast { card: round });
    act(&mut game, Action::ChooseX { value: 1 });
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 5);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseEffectObject { object: old_stolen });
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.object_view(stolen).controller, player.other());
    act(&mut game, Action::ChooseEffectObject { object: old_ours });
    act(&mut game, Action::FinishEffectChoice);
    assert_ne!(game.object_ref(ours), old_ours);
    assert_eq!(game.object_view(round).zone, Zone::Graveyard);
}

#[test]
fn archangel_taxes_player_and_planeswalker_attacks_and_each_blocker_while_attacking() {
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(player);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let archangel = put(&mut game, enemy, "Archangel of Tithes", Zone::Battlefield);
    let walker = put(&mut game, enemy, "Oko, the Ringleader", Zone::Battlefield);
    game.add_counter(walker, CounterKind::Loyalty, 4);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for action in [
        Action::Attack { card: giant },
        Action::AttackPlaneswalker {
            card: giant,
            defender: game.object_ref(walker),
        },
    ] {
        let mut branch = game.clone();
        act(&mut branch, action);
        act(&mut branch, Action::FinishAttackers);
        assert_eq!(branch.observe(player).attack_payment.unwrap().generic, 1);
    }
    assert!(game.keywords(archangel).contains(Keywords::FLYING));
    assert_eq!(stats(&game, archangel), (3, 5));

    let mut game = scenario(player);
    let archangel = put(&mut game, player, "Archangel of Tithes", Zone::Battlefield);
    let blocker = put(&mut game, enemy, "Healer's Hawk", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: archangel });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: archangel,
        },
    );
    act(&mut game, Action::FinishBlockers);
    assert_eq!(game.observe(enemy).block_payment.unwrap().generic, 1);
}

#[test]
fn newt_grants_the_matching_exact_flashback_route_after_combat_damage() {
    let player = Player::P0;
    for saddled in [false, true] {
        let mut game = scenario(player);
        let newt = put(&mut game, player, "Archmage's Newt", Zone::Battlefield);
        let rider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Graveyard);
        let opposing = put(&mut game, player.other(), "Opt", Zone::Graveyard);
        if saddled {
            act(&mut game, activation(newt, 0));
            let object = game.object_ref(rider);
            act(&mut game, Action::ChooseSaddler { object });
            act(&mut game, Action::FinishSaddle);
            settle(&mut game);
        }
        let source = game.damage_source(newt);
        game.apply_damage_batch(&[DamageEvent {
            preventable: true,
            source,
            target: Target::Player(player.other()),
            amount: 2,
            combat: true,
        }]);
        game.give_priority();
        game.prepare_triggers();
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(opposing),
            },
        );
        choose_target(&mut game, Target::Card(spell));
        settle(&mut game);
        let actions = game.legal_actions();
        assert_eq!(
            actions.contains(&Action::AnnounceGrantedZeroFlashback {
                card: spell,
                flash_surcharge: false,
            }),
            saddled
        );
        assert_eq!(
            actions.contains(&Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            }),
            !saddled
        );
    }
}

#[test]
fn arid_archway_requires_the_bounce_and_surveys_only_for_another_desert() {
    let player = Player::P1;
    for return_desert in [false, true] {
        let mut game = scenario(player);
        let land = put(&mut game, player, "Arid Archway", Zone::Hand);
        let desert = put(&mut game, player, "Bristling Backwoods", Zone::Battlefield);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped);
        pass_pair(&mut game);
        let object = game.object_ref(if return_desert { desert } else { land });
        act(&mut game, Action::ChooseEffectObject { object });
        assert_eq!(game.object_view(object.card).zone, Zone::Hand);
        assert_eq!(game.observe(player).library_choice.is_some(), return_desert);
        if return_desert {
            assert!(game.observe(player.other()).library_choice.is_none());
        }
    }
}

#[test]
fn armadillo_samples_live_toughness_and_has_a_separate_opposing_ward_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Armored Armadillo", Zone::Battlefield);
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    act(&mut game, activation(creature, 0));
    pay(&mut game);
    cast(&mut game, growth, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (10, 7));

    act(&mut game, Action::Pass);
    let kill = put(&mut game, player.other(), "Murder", Zone::Hand);
    cast(&mut game, kill, Some(Target::Card(creature)));
    let ward = game.state.stack.last().unwrap();
    assert!(matches!(ward.kind, StackKind::TriggeredAbility { .. }));
    assert_eq!(ward.source.unwrap().id, creature);
    assert_eq!(ward.related_stack.unwrap().id, game.state.stack[0].id);
}

#[test]
fn aegis_exiles_a_creature_then_copies_it_for_the_exact_equip_duration() {
    let player = Player::P1;
    let mut game = scenario(player);
    let aegis = put(&mut game, player, "Assimilation Aegis", Zone::Hand);
    let original = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let wearer = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    cast(&mut game, aegis, None);
    pass_pair(&mut game);
    choose_target(&mut game, Target::Card(original));
    settle(&mut game);
    assert_eq!(game.object_view(original).zone, Zone::Exile);
    act(&mut game, activation(aegis, 0));
    choose_target(&mut game, Target::Card(wearer));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(wearer).card.name(), "Hill Giant");
    game.move_object(aegis, Zone::Graveyard);
    assert_eq!(game.object_view(wearer).card.name(), "Healer's Hawk");
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
}

#[test]
fn knifepoint_makes_one_mercenary_and_grants_first_strike_during_its_own_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "At Knifepoint", Zone::Battlefield);
    let pirate = put(&mut game, player, "Spyglass Siren", Zone::Battlefield);
    assert!(game.keywords(pirate).contains(Keywords::FIRST_STRIKE));
    for _ in 0..2 {
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        settle(&mut game);
    }
    let tokens = tokens_with(&game, player, "Mercenary");
    assert_eq!(tokens.len(), 1);
    let token = tokens[0];
    assert_eq!(stats(&game, token), (1, 1));
    assert_eq!(game.object_view(token).colors, 8);
    assert!(game.keywords(token).contains(Keywords::FIRST_STRIKE));
    reject(&mut game, activation(token, 0));
    game.state.objects[token as usize].summoning_sick = false;
    act(&mut game, activation(token, 0));
    choose_target(&mut game, Target::Card(pirate));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(stats(&game, pirate).0, 2);
    game.state.active = player.other();
    assert!(!game.keywords(pirate).contains(Keywords::FIRST_STRIKE));
}

#[test]
fn aven_plots_a_live_spell_and_its_caster_pays_the_later_exile_tax() {
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(enemy);
    let spell = put(&mut game, enemy, "Hill Giant", Zone::Hand);
    let aven = put(&mut game, player, "Aven Interrupter", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    cast(&mut game, aven, None);
    pass_pair(&mut game);
    choose_target(&mut game, Target::Spell(spell));
    settle(&mut game);
    assert!(
        game.keywords(aven)
            .contains(Keywords::FLASH.union(Keywords::FLYING))
    );
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(
        game.observe(enemy).plotted_cards[0].object,
        game.object_ref(spell)
    );
    assert!(!game.has_plotted_permission(enemy, spell));
    game.state.turn += 1;
    act(&mut game, Action::AnnouncePlotted { card: spell });
    pay(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.method, CastMethod::Plot);
    assert_eq!(cast.payment.into_iter().sum::<u32>(), 2);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
}
