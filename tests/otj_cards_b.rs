//! Actual B-range Outlaws definitions and interacting rules.
#[path = "support/otj_e_l.rs"]
mod scenarios;
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*, rng::Rng};
use scenarios::{MERCENARY, TREASURE, announce_activation, choose_targets, resolve, tokens};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(
    game: &mut Game,
    card: CardId,
    ability: u16,
    targets: &[Target],
    sacrifices: &[CardId],
) {
    announce_activation(game, card, ability);
    choose_targets(game, targets);
    for &sacrifice in sacrifices {
        let object = game.object_ref(sacrifice);
        act(game, Action::ChooseAdditionalCost { object });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(game, Action::FinishAdditionalCostGroup);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn badlands_revival_has_independent_optional_graveyard_roles() {
    let player = Player::P0;
    for same in [false, true] {
        let mut game = scenario(player);
        let revival = put(&mut game, player, "Badlands Revival", Zone::Hand);
        let creature = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let permanent = put(&mut game, player, "Forest", Zone::Graveyard);
        let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
        act(&mut game, Action::AnnounceCast { card: revival });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(opposing),
            },
        );
        choose_targets(
            &mut game,
            &[
                Target::Card(creature),
                Target::Card(if same { creature } else { permanent }),
            ],
        );
        pay(&mut game);
        resolve(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(
            game.object_view(permanent).zone,
            if same { Zone::Graveyard } else { Zone::Hand }
        );
    }
}

#[test]
fn blood_hustler_and_bandits_haul_trigger_once_each_from_paid_activations() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let haul = put(&mut game, player, "Bandit's Haul", Zone::Battlefield);
    let hustler = put(&mut game, player, "Blood Hustler", Zone::Battlefield);
    for _ in 0..2 {
        activate(&mut game, hustler, 0, &[Target::Player(opponent)], &[]);
        resolve(&mut game);
    }
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.counter_count(hustler, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(haul, CounterKind::Loot), 1);
    let mut mana = game.clone();
    choose(&mut mana, |action| {
        matches!(action, Action::ActivateMana { card, production, .. }
        if *card == haul && *production == [0,0,0,1,0,0])
    });
    assert!(mana.object_view(haul).tapped);
    game.add_counter(haul, CounterKind::Loot, 1);
    let drawn = game.state.players[player.index()].cards_drawn_this_turn;
    activate(&mut game, haul, 1, &[], &[]);
    resolve(&mut game);
    assert_eq!(game.counter_count(haul, CounterKind::Loot), 0);
    assert_eq!(
        game.state.players[player.index()].cards_drawn_this_turn,
        drawn + 1
    );
    assert!(game.object_view(haul).tapped);
}

#[test]
fn baron_makes_one_lifelink_vampire_rogue_and_can_sacrifice_another_artifact() {
    let player = Player::P0;
    let mut game = scenario(player);
    let baron = put(
        &mut game,
        player,
        "Baron Bertram Graywater",
        Zone::Battlefield,
    );
    game.create_tokens(player, TREASURE, 2, false);
    game.advance_entries();
    resolve(&mut game);
    let vampire = Card::from_catalogue_id(5613);
    let created = tokens(&game, player, vampire);
    assert_eq!(created.len(), 1);
    let token = game.object_view(created[0]);
    assert!(token.has_subtype("Vampire") && token.has_subtype("Rogue"));
    assert_eq!((token.colors, token.power, token.toughness), (4, 1, 1));
    assert!(token.keywords.contains(Keywords::LIFELINK));
    let treasure = tokens(&game, player, TREASURE)[0];
    activate(&mut game, baron, 0, &[], &[treasure]);
    resolve(&mut game);
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn betrayal_uses_one_legal_controlled_creature_and_two_distinct_other_targets() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Betrayal at the Vault", Zone::Hand);
    let dealer = put(&mut game, player, "Daring Thunder-Thief", Zone::Battlefield);
    let one = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let two = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(dealer),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(dealer),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(one),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(one),
        },
    );
    choose_targets(&mut game, &[Target::Card(two)]);
    pay(&mut game);
    let mut invalid_dealer = game.clone();
    invalid_dealer.record_control(game.object_ref(dealer), player.other(), None);
    resolve(&mut invalid_dealer);
    assert!(invalid_dealer.on_battlefield(one) && invalid_dealer.on_battlefield(two));
    resolve(&mut game);
    assert_eq!(game.object_view(one).zone, Zone::Graveyard);
    assert_eq!(game.object_view(two).zone, Zone::Graveyard);
    assert!(game.on_battlefield(dealer));
}

#[test]
fn binding_negotiation_reveals_every_card_and_leaves_the_choice_owned() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Binding Negotiation", Zone::Hand);
    let land = put(&mut game, other, "Forest", Zone::Hand);
    let nonland = put(&mut game, other, "Hill Giant", Zone::Hand);
    let exiled = put(&mut game, other, "Healer's Hawk", Zone::Exile);
    cast(&mut game, spell, Some(Target::Player(other)));
    pass_pair(&mut game);
    assert!(
        game.observe(player)
            .known_hand
            .contains(&game.object_ref(land))
    );
    let mut discard = game.clone();
    let object = game.object_ref(nonland);
    act(
        &mut discard,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    assert_eq!(discard.object_view(nonland).zone, Zone::Graveyard);
    assert_eq!(discard.object_view(exiled).zone, Zone::Exile);
    act(&mut game, Action::ChooseHandCard { object: None });
    let object = game.object_ref(exiled);
    scenarios::apply_raw_and_index(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(exiled).zone, Zone::Graveyard);
}

#[test]
fn desecrator_adds_a_counter_for_either_sacrifice_and_treasure_only_for_an_outlaw() {
    let player = Player::P0;
    for (name, outlaw) in [("Spyglass Siren", true), ("Hill Giant", false)] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Boneyard Desecrator", Zone::Battlefield);
        let victim = put(&mut game, player, name, Zone::Battlefield);
        assert!(game.keywords(source).contains(Keywords::MENACE));
        activate(&mut game, source, 0, &[], &[victim]);
        resolve(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(tokens(&game, player, TREASURE).len(), usize::from(outlaw));
    }
}

#[test]
fn boom_box_sacrifices_while_paying_and_destroys_all_three_chosen_roles() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Boom Box", Zone::Battlefield);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    activate(
        &mut game,
        source,
        0,
        &[
            Target::Card(artifact),
            Target::Card(creature),
            Target::Card(land),
        ],
        &[],
    );
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert!(game.on_battlefield(artifact));
    resolve(&mut game);
    assert!(
        [artifact, creature, land]
            .iter()
            .all(|&card| game.object_view(card).zone == Zone::Graveyard)
    );
}

#[test]
fn breeches_resolves_a_separate_coin_trigger_using_the_second_spells_x() {
    let player = Player::P0;
    for win in [false, true] {
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Breeches, the Blastmaker",
            Zone::Battlefield,
        );
        let first = put(&mut game, player, "Hill Giant", Zone::Hand);
        let x_spell = put(&mut game, player, "Exsanguinate", Zone::Hand);
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        cast(&mut game, first, None);
        assert_eq!(game.state.stack.len(), 1);
        resolve(&mut game);
        scenarios::cast_x(&mut game, x_spell, 3, &[]);
        assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, source);
        pass_pair(&mut game);
        let object = game.object_ref(artifact);
        game.state.rng = (1..1000)
            .map(Rng)
            .find(|state| {
                let mut state = *state;
                (state.below(2) == 1) == win
            })
            .unwrap();
        act(&mut game, Action::Sacrifice { object });
        if !win {
            choose_targets(&mut game, &[Target::Player(player.other())]);
        }
        pass_pair(&mut game);
        if win {
            let copy = game.state.stack.last().unwrap();
            assert!(copy.is_copy);
            assert_eq!(copy.x, Some(3));
        } else {
            assert_eq!(game.state.players[player.other().index()].life, 15);
        }
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 2);
    }
}

#[test]
fn brimstone_roundup_counts_nonhand_casts_and_offers_its_printed_plot() {
    let player = Player::P1;
    let mut game = scenario(player);
    let roundup = put(&mut game, player, "Brimstone Roundup", Zone::Hand);
    let mut plot = game.clone();
    act(&mut plot, Action::AnnouncePlot { card: roundup });
    assert_eq!(
        plot.observe(player).plot_payment.unwrap().cost,
        ManaCost {
            generic: 2,
            colored: [0, 0, 0, 1, 0, 0],
        }
    );
    cast(&mut game, roundup, None);
    resolve(&mut game);
    let outside = put(&mut game, player, "Hill Giant", Zone::Exile);
    let exact = game.object_ref(outside);
    game.grant_exile_play_permission(player, exact, false);
    act(&mut game, Action::AnnounceExile { card: outside });
    pay(&mut game);
    resolve(&mut game);
    assert_eq!(tokens(&game, player, MERCENARY).len(), 1);
    let third = put(&mut game, player, "Ankle Biter", Zone::Hand);
    cast(&mut game, third, None);
    resolve(&mut game);
    assert_eq!(tokens(&game, player, MERCENARY).len(), 1);
}

#[test]
fn bristlepack_keeps_defender_when_its_own_power_qualifies_to_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Bristlepack Sentry", Zone::Battlefield);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    let action = Action::Attack { card: source };
    reject(&mut game, action.clone());
    game.add_counter(source, CounterKind::PlusOne, 1);
    assert!(game.keywords(source).contains(Keywords::DEFENDER));
    act(&mut game, action);
    act(&mut game, Action::FinishAttackers);
    assert!(game.object_view(source).tapped);
}

#[test]
fn bristly_bill_targets_any_creature_and_doubles_only_own_creatures_plus_one_counters() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Bristly Bill, Spine Sower",
        Zone::Battlefield,
    );
    let other = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    choose_targets(&mut game, &[Target::Card(other)]);
    resolve(&mut game);
    game.add_counter(source, CounterKind::PlusOne, 2);
    game.add_counter(source, CounterKind::Charge, 1);
    game.add_counter(artifact, CounterKind::PlusOne, 1);
    activate(&mut game, source, 0, &[], &[]);
    resolve(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 4);
    assert_eq!(game.counter_count(source, CounterKind::Charge), 1);
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 1);
}

#[test]
fn bruse_exiles_the_exact_top_card_for_an_ox_or_a_later_paid_spell() {
    let player = Player::P0;
    for name in ["Forest", "Hill Giant"] {
        let mut game = scenario(player);
        let bruse = put(&mut game, player, "Bruse Tarl, Roving Rancher", Zone::Hand);
        let top = put(&mut game, player, name, Zone::Library);
        cast(&mut game, bruse, None);
        resolve(&mut game);
        assert_eq!(game.object_view(top).zone, Zone::Exile);
        let ox = tokens(&game, player, Card::from_catalogue_id(5479));
        if name == "Forest" {
            assert_eq!(ox.len(), 1);
            assert!(game.keywords(ox[0]).contains(Keywords::DOUBLE_STRIKE));
            assert!(
                !game
                    .legal_actions()
                    .contains(&Action::PlayLand { card: top })
            );
        } else {
            assert!(ox.is_empty());
            assert!(game.state.exile_permissions.iter().any(|permission| {
                permission.object == game.object_ref(top)
                    && permission.cast_only
                    && permission.expiry == ExilePlayExpiry::EndOfNextTurn
            }));
        }
    }
}

#[test]
fn bucolic_ranchs_colored_mana_is_mount_only_and_the_lookup_is_private() {
    let player = Player::P1;
    let mut game = scenario(player);
    let ranch = put(&mut game, player, "Bucolic Ranch", Zone::Battlefield);
    let mount = put(&mut game, player, "Trained Arynx", Zone::Library);
    activate(&mut game, ranch, 2, &[], &[]);
    pass_pair(&mut game);
    assert!(game.observe(player).library_selection.is_some());
    assert!(game.observe(player.other()).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: mount });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(mount).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(mount))
    );

    let mut mana = scenario(player);
    let ranch = put(&mut mana, player, "Bucolic Ranch", Zone::Battlefield);
    put(&mut mana, player, "Plains", Zone::Battlefield);
    let mount = put(&mut mana, player, "Trained Arynx", Zone::Hand);
    let other = put(&mut mana, player, "Ankle Biter", Zone::Hand);
    mana.state.players[player.index()].mana = [0; 6];
    assert!(mana.legal_actions().iter().any(
        |a| matches!(a, Action::ActivateMana { card, production, .. }
        if *card == ranch && *production == [0,0,0,0,0,1])
    ));
    assert!(
        mana.legal_actions()
            .contains(&Action::AnnounceCast { card: mount })
    );
    reject(&mut mana, Action::AnnounceCast { card: other });
}
