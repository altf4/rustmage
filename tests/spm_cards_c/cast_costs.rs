use super::*;

const WHITE: Mana = [1, 0, 0, 0, 0, 0];

fn discard_for_mayhem(game: &mut Game, player: Player, card: CardId) {
    let source = put(game, player, "Masked Meower", Zone::Battlefield);
    activate(game, source, 0);
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    settle(game);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}

fn mayhem(game: &mut Game, card: CardId, color_option: Option<u16>) {
    act(game, Action::AnnounceMayhem { card });
    if let Some(cost) = color_option {
        act(game, Action::ChooseSpellCost { cost });
    }
    pay(game);
}

fn web_action(card: CardId, colored: Mana, generic: u32) -> Action {
    Action::AnnounceWebSlinging {
        card,
        face: CastFace::Normal,
        cost: ManaCost { colored, generic },
        flash_surcharge: false,
        origin: WebSlingingOrigin::Ordinary,
    }
}

fn choose_cast_clause(game: &mut Game, predicate: impl Fn(CastCostClause) -> bool) {
    let clause = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| !row.paid && predicate(row.clause))
        .unwrap()
        .try_into()
        .unwrap();
    act(game, Action::ChooseCastCost { clause });
}

fn pay_web(game: &mut Game, returned: CardId) {
    act(game, Action::BeginOrderedCosts);
    choose_cast_clause(game, |clause| clause == CastCostClause::Mana);
    pay(game);
    choose_cast_clause(game, |clause| {
        matches!(clause, CastCostClause::WebSlingingReturn(_))
    });
    let object = game.object_ref(returned);
    act(game, Action::PayWebSlingingReturn { object });
}

fn tap(game: &mut Game, card: CardId) {
    game.state.objects[card as usize].tapped = true;
}

#[test]
fn goblinoids_and_islanders_require_an_actual_discard_and_pay_their_printed_mayhem_costs() {
    for (name, generic, power, toughness, haste) in [
        ("Raging Goblinoids", 2, 5, 4, true),
        ("Spider-Islanders", 1, 4, 3, false),
    ] {
        let player = Player::P0;
        let mut game = scenario(player);
        let card = put(&mut game, player, name, Zone::Hand);
        let milled = put(&mut game, player, name, Zone::Graveyard);
        discard_for_mayhem(&mut game, player, card);
        reject(&mut game, Action::AnnounceMayhem { card: milled });
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, generic - 1];
        reject(&mut game, Action::AnnounceMayhem { card });
        game.state.players[player.index()].mana[5] += 1;
        mayhem(&mut game, card, None);
        settle(&mut game);
        let creature = game.object_view(card);
        assert_eq!(creature.zone, Zone::Battlefield, "{name}");
        assert_eq!((creature.power, creature.toughness), (power, toughness));
        assert_eq!(creature.keywords.contains(Keywords::HASTE), haste);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn glider_attaches_only_after_its_graveyard_cast_and_has_a_separate_paid_equip() {
    let player = Player::P0;
    for from_graveyard in [false, true] {
        let mut game = scenario(player);
        let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let glider = put(
            &mut game,
            player,
            "Rocket-Powered Goblin Glider",
            Zone::Hand,
        );
        if from_graveyard {
            discard_for_mayhem(&mut game, player, glider);
            game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
            mayhem(&mut game, glider, None);
            pass_pair(&mut game);
        } else {
            cast(&mut game, glider, None);
            settle(&mut game);
            assert_eq!(game.object_view(glider).attached_to, None);
            game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
            activate(&mut game, glider, 0);
        }
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        target(&mut game, Target::Card(bearer));
        if !from_graveyard {
            pay(&mut game);
        }
        settle(&mut game);
        let equipment = game.object_view(glider);
        let creature = game.object_view(bearer);
        assert_eq!(equipment.attached_to, Some(creature.reference));
        assert_eq!((creature.power, creature.toughness), (4, 2));
        assert!(
            creature
                .keywords
                .contains(Keywords::FLYING.union(Keywords::HASTE))
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn quicksand_spares_its_casters_creatures_only_when_mayhem_was_paid() {
    let player = Player::P0;
    for paid in [false, true] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Sandman's Quicksand", Zone::Hand);
        if paid {
            discard_for_mayhem(&mut game, player, spell);
            game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 3];
            mayhem(&mut game, spell, None);
        } else {
            game.state.players[player.index()].mana = [0, 0, 2, 0, 0, 1];
            cast(&mut game, spell, None);
        }
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(own).zone,
            if paid {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn kaine_can_pay_either_mayhem_color_and_chooses_whether_to_discard_a_land() {
    let player = Player::P0;
    for (cost, color, discard) in [(0, 2, false), (1, 3, true)] {
        let mut game = scenario(player);
        let kaine = put(&mut game, player, "Scarlet Spider, Kaine", Zone::Hand);
        discard_for_mayhem(&mut game, player, kaine);
        let land = put(&mut game, player, "Forest", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        game.state.players[player.index()].mana[color] = 1;
        mayhem(&mut game, kaine, Some(cost));
        pass_pair(&mut game);
        pass_pair(&mut game);
        act(
            &mut game,
            if discard {
                Action::Discard { card: land }
            } else {
                Action::DeclineEffectPayment
            },
        );
        assert_eq!(
            game.counter_count(kaine, CounterKind::PlusOne),
            u64::from(discard)
        );
        assert_eq!(
            game.object_view(land).zone,
            if discard { Zone::Graveyard } else { Zone::Hand }
        );
        assert!(game.object_view(kaine).keywords.contains(Keywords::MENACE));
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn ben_reilly_enters_with_the_returned_mana_value_only_when_web_slinging_was_paid() {
    let player = Player::P0;
    for paid in [false, true] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let ben = put(&mut game, player, "Scarlet Spider, Ben Reilly", Zone::Hand);
        if paid {
            tap(&mut game, creature);
            game.state.players[player.index()].mana = [0, 0, 0, 1, 1, 0];
            act(&mut game, web_action(ben, [0, 0, 0, 1, 1, 0], 0));
            pay_web(&mut game, creature);
        } else {
            game.state.players[player.index()].mana = [0, 0, 0, 1, 1, 1];
            cast(&mut game, ben, None);
        }
        pass_pair(&mut game);
        assert!(
            game.state.stack.is_empty(),
            "counters are an entry replacement"
        );
        assert_eq!(game.object_view(ben).zone, Zone::Battlefield);
        assert_eq!(
            game.counter_count(ben, CounterKind::PlusOne),
            if paid { 4 } else { 0 }
        );
        assert!(game.object_view(ben).keywords.contains(Keywords::TRAMPLE));
        assert_eq!(
            game.object_view(creature).zone,
            if paid { Zone::Hand } else { Zone::Battlefield }
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn web_slinger_can_return_the_stolen_creature_that_funded_the_cast() {
    let player = Player::P0;
    let owner = player.other();
    let mut game = scenario(player);
    let funder = put(&mut game, owner, "Oasis Gardener", Zone::Battlefield);
    let card = put(&mut game, player, "Spider-Man, Web-Slinger", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    let action = web_action(card, WHITE, 0);
    reject(&mut game, action.clone());
    let object = game.object_ref(funder);
    game.change_controller(object, player);
    game.state.objects[funder as usize].summoning_sick = false;
    act(&mut game, action);
    choose(&mut game, |action| {
        matches!(action, Action::ActivateMana { card, production, .. }
            if *card == funder && *production == WHITE)
    });
    pay_web(&mut game, funder);
    settle(&mut game);
    assert_eq!(game.object_view(funder).zone, Zone::Hand);
    assert_eq!(game.object_view(funder).owner, owner);
    assert_eq!(game.hand_count(owner), 1);
    assert_eq!(stats(&game, card), (3, 3));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn silk_creates_the_citizen_and_boosts_only_the_creatures_already_controlled() {
    let player = Player::P0;
    let mut game = scenario(player);
    let returned = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let silk = put(&mut game, player, "Silk, Web Weaver", Zone::Hand);
    tap(&mut game, returned);
    act(&mut game, web_action(silk, [1, 0, 0, 0, 1, 0], 1));
    pay_web(&mut game, returned);
    settle(&mut game);
    cast(&mut game, returned, None);
    settle(&mut game);
    let citizen = game
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token() && object.controller == player)
                .then_some(id as CardId)
        })
        .unwrap();
    let token = game.object_view(citizen);
    assert!(token.has_subtype("Human") && token.has_subtype("Citizen"));
    assert_eq!(token.colors, 1 | 16);
    assert_eq!((token.power, token.toughness), (1, 1));

    game.state.players[player.index()].mana = [1, 0, 0, 0, 1, 3];
    activate(&mut game, silk, 0);
    pay(&mut game);
    settle(&mut game);
    for (id, expected) in [(silk, (5, 7)), (returned, (4, 4)), (citizen, (3, 3))] {
        assert_eq!(stats(&game, id), expected);
        assert!(game.object_view(id).keywords.contains(Keywords::VIGILANCE));
    }
    assert_eq!(stats(&game, enemy), (2, 2));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    game.state.players[player.index()].mana[0] = 1;
    let later = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    cast(&mut game, later, None);
    settle(&mut game);
    assert_eq!(stats(&game, later), (1, 1));
    assert!(
        !game
            .object_view(later)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    cleanup(&mut game);
    assert_eq!(stats(&game, citizen), (1, 1));
    assert!(
        !game
            .object_view(citizen)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
}

#[test]
fn spider_man_india_puts_a_lasting_counter_and_grants_flying_until_cleanup() {
    let player = Player::P0;
    let mut game = scenario(player);
    let returned = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let india = put(&mut game, player, "Spider-Man India", Zone::Hand);
    tap(&mut game, returned);
    act(&mut game, web_action(india, [1, 0, 0, 0, 1, 0], 1));
    pay_web(&mut game, returned);
    settle(&mut game);
    cast(&mut game, returned, None);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(india));
    settle(&mut game);
    assert_eq!(game.counter_count(india, CounterKind::PlusOne), 1);
    assert!(game.object_view(india).keywords.contains(Keywords::FLYING));
    cleanup(&mut game);
    assert_eq!(game.counter_count(india, CounterKind::PlusOne), 1);
    assert!(!game.object_view(india).keywords.contains(Keywords::FLYING));
}

#[test]
fn brooklyn_visionary_web_slings_and_searches_only_for_a_tapped_basic_land() {
    let player = Player::P0;
    let mut game = scenario(player);
    let returned = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let basic = put(&mut game, player, "Plains", Zone::Library);
    let nonbasic = put(&mut game, player, "Savage Mansion", Zone::Library);
    let visionary = put(
        &mut game,
        player,
        "Spider-Man, Brooklyn Visionary",
        Zone::Hand,
    );
    tap(&mut game, returned);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 2];
    act(&mut game, web_action(visionary, [0, 0, 0, 0, 1, 0], 2));
    pay_web(&mut game, returned);
    pass_pair(&mut game);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: basic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(visionary).zone, Zone::Battlefield);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert!(game.object_view(basic).tapped);
    assert_eq!(game.object_view(nonbasic).zone, Zone::Library);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

#[test]
fn spider_sense_counters_an_instant_and_a_trigger_but_cannot_target_a_creature_spell() {
    let player = Player::P0;
    for ability in [false, true] {
        let mut game = scenario(player);
        let returned = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let sense = put(&mut game, player, "Spider-Sense", Zone::Hand);
        let victim = if ability {
            let hunter = put(&mut game, player, "Helpful Hunter", Zone::Hand);
            cast(&mut game, hunter, None);
            reject(&mut game, Action::AnnounceCast { card: sense });
            pass_pair(&mut game);
            Target::Stack(game.state.stack.last().unwrap().id)
        } else {
            let shock = put(&mut game, player, "Shock", Zone::Hand);
            cast(&mut game, shock, Some(Target::Player(player.other())));
            Target::Spell(shock)
        };
        tap(&mut game, returned);
        game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut game, web_action(sense, [0, 1, 0, 0, 0, 0], 0));
        target(&mut game, victim);
        pay_web(&mut game, returned);
        settle(&mut game);
        assert_eq!(game.object_view(sense).zone, Zone::Graveyard);
        assert_eq!(game.object_view(returned).zone, Zone::Hand);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        assert_eq!(game.state.players[player.other().index()].life, 20);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}
