//! LCI creature, tap-cost, and public library choices exercised through play.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn material(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance_to(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishBlockers
                    | Action::PayCosts { .. }
                    | Action::ChooseCastCost { .. }
            )
        });
    }
    panic!("expected the next card choice");
}

fn choose_materials(game: &mut Game, required: &[CardId], invalid: CardId) {
    advance_to(game, |game| {
        matches!(game.state.pending, Pending::AdditionalCost { .. })
    });
    let wrong = material(game, invalid);
    reject(game, wrong);
    for (index, &card) in required.iter().enumerate() {
        let choice = material(game, card);
        act(game, choice.clone());
        if index == 0 {
            reject(game, choice);
        }
    }
    advance_to(game, |game| game.state.announcing.is_none());
}

#[test]
fn ghalta_deploys_any_number_of_the_controllers_private_creature_cards_together() {
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 3, 5];
    let ghalta = named(&mut game, p, "Ghalta, Stampede Tyrant", Zone::Hand);
    let first = put(&mut game, p, "Grizzly Bears", Zone::Hand);
    let second = put(&mut game, p, "Bear Cub", Zone::Hand);
    let artifact = put(&mut game, p, "Swiftfoot Boots", Zone::Hand);
    let foreign = put(&mut game, p.other(), "Grizzly Bears", Zone::Hand);
    cast(&mut game, ghalta, None);
    advance_to(&mut game, |game| game.observe(p).hand_choice.is_some());
    assert!(game.keywords(ghalta).contains(Keywords::TRAMPLE));
    assert!(game.observe(p.other()).hand_choice.is_none());

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseHandCard { object: None });
    assert_eq!(declined.object_view(first).zone, Zone::Hand);

    let action = |game: &Game, card| Action::ChooseHandCard {
        object: Some(game.object_ref(card)),
    };
    let invalid_artifact = action(&game, artifact);
    let invalid_foreign = action(&game, foreign);
    reject(&mut game, invalid_artifact);
    reject(&mut game, invalid_foreign);
    for card in [first, second] {
        let choice = action(&game, card);
        act(&mut game, choice);
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    act(&mut game, Action::ChooseHandCard { object: None });
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
    assert_eq!(game.object_view(foreign).zone, Zone::Hand);
}

#[test]
fn gishath_reveals_as_many_cards_as_player_combat_damage_and_deploys_selected_dinosaurs() {
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [1, 0, 0, 1, 1, 5];
    let outside = put(&mut game, p, "Belligerent Yearling", Zone::Library);
    let mut revealed = Vec::new();
    for name in [
        "Forest",
        "Island",
        "Shock",
        "Bear Cub",
        "Forest",
        "Hulking Raptor",
        "Belligerent Yearling",
    ] {
        revealed.push(put(&mut game, p, name, Zone::Library));
    }
    let gishath = named(&mut game, p, "Gishath, Sun's Avatar", Zone::Hand);
    cast(&mut game, gishath, None);
    settle(&mut game);
    assert!(
        game.keywords(gishath).contains(
            Keywords::HASTE
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE)
        )
    );
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: gishath });
    act(&mut game, Action::FinishAttackers);
    assert!(!game.object_view(gishath).tapped);
    advance_to(&mut game, |game| {
        game.observe(p.other()).library_selection.is_some()
    });
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 7
    );
    let selection = game.observe(p.other()).library_selection.unwrap();
    assert!(selection.public);
    assert_eq!(selection.remaining.len(), 7);
    reject(&mut game, Action::SelectLibrary { card: outside });
    reject(&mut game, Action::SelectLibrary { card: revealed[3] });
    for &card in &revealed[5..] {
        act(&mut game, Action::SelectLibrary { card });
    }
    act(&mut game, Action::FinishLibrarySelection);
    assert!(
        revealed[5..]
            .iter()
            .all(|&card| game.object_view(card).zone == Zone::Battlefield)
    );
    assert!(
        revealed[..5]
            .iter()
            .all(|card| game.state.players[p.index()].library[..5].contains(card))
    );
}

#[test]
fn glorifier_sacrifices_another_eligible_permanent_then_puts_a_counter_on_each_target() {
    let p = Player::P0;
    let mut game = scenario(p);
    let card = named(&mut game, p, "Glorifier of Suffering", Zone::Hand);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let friendly = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, card, None);
    advance_to(&mut game, |game| game.observe(p).effect_choice.is_some());
    let self_payment = Action::Sacrifice {
        object: game.object_ref(card),
    };
    reject(&mut game, self_payment);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert_eq!(declined.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(declined.counter_count(friendly, CounterKind::PlusOne), 0);

    let payment = Action::Sacrifice {
        object: game.object_ref(artifact),
    };
    act(&mut game, payment);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    act(&mut game, target(friendly));
    reject(&mut game, target(friendly));
    act(&mut game, target(opposing));
    settle(&mut game);
    for card in [friendly, opposing] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
}

#[test]
fn goldfury_strider_can_tap_itself_and_a_summoning_sick_creature_at_sorcery_speed() {
    let p = Player::P1;
    let mut game = scenario(p);
    let strider = named(&mut game, p, "Goldfury Strider", Zone::Battlefield);
    let friend = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    game.state.objects[strider as usize].summoning_sick = true;
    game.state.objects[friend as usize].summoning_sick = true;
    assert!(game.keywords(strider).contains(Keywords::TRAMPLE));
    game.state.step = Step::Upkeep;
    reject(&mut game, activate(strider));
    game.state.step = Step::Main1;
    act(&mut game, activate(strider));
    act(&mut game, target(enemy));
    choose_materials(&mut game, &[strider, friend], land);
    settle(&mut game);
    assert!(game.object_view(strider).tapped && game.object_view(friend).tapped);
    assert_eq!(stats(&game, enemy), (4, 2));
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(stats(&game, enemy), (2, 2));
}

#[test]
fn guardian_of_the_great_door_pays_two_white_and_four_distinct_eligible_taps() {
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
    let spell = named(&mut game, p, "Guardian of the Great Door", Zone::Hand);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.objects[creature as usize].summoning_sick = true;
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    reject(&mut game, Action::AnnounceCast { card: spell });
    let second_land = put(&mut game, p, "Island", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    choose_materials(
        &mut game,
        &[creature, artifact, land, second_land],
        opposing,
    );
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    for card in [creature, artifact, land, second_land] {
        assert!(game.object_view(card).tapped);
    }
    assert!(game.keywords(spell).contains(Keywords::FLYING));
    assert_eq!(stats(&game, spell), (4, 4));
}

#[test]
fn hotfoot_gnome_taps_on_its_entry_turn_and_gives_haste_only_to_another_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let gnome = named(&mut game, p, "Hotfoot Gnome", Zone::Hand);
    let creature = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, gnome, None);
    settle(&mut game);
    assert!(game.keywords(gnome).contains(Keywords::HASTE));
    act(&mut game, activate(gnome));
    reject(&mut game, target(gnome));
    act(&mut game, target(creature));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert!(game.object_view(gnome).tapped);
    assert!(game.keywords(creature).contains(Keywords::HASTE));
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert!(!game.keywords(creature).contains(Keywords::HASTE));
}

#[test]
fn in_the_presence_of_ages_reveals_four_publicly_and_mills_unselected_cards() {
    let p = Player::P1;
    let mut game = scenario(p);
    let first = put(&mut game, p, "Bear Cub", Zone::Library);
    let second = put(&mut game, p, "Grizzly Bears", Zone::Library);
    let land = put(&mut game, p, "Forest", Zone::Library);
    let ineligible = put(&mut game, p, "Shock", Zone::Library);
    let spell = named(&mut game, p, "In the Presence of Ages", Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let selection = game.observe(p.other()).library_selection.unwrap();
    assert!(selection.public);
    assert_eq!(selection.remaining.len(), 4);
    reject(&mut game, Action::SelectLibrary { card: ineligible });
    act(&mut game, Action::SelectLibrary { card: first });
    reject(&mut game, Action::SelectLibrary { card: second });
    act(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ineligible).zone, Zone::Graveyard);
}

#[test]
fn itzquinth_pays_first_then_selects_a_dinosaur_and_a_distinct_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let dinosaur = put(&mut game, p, "Hulking Raptor", Zone::Battlefield);
    let non_dinosaur = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let spell = named(&mut game, p, "Itzquinth, Firstborn of Gishath", Zone::Hand);
    cast(&mut game, spell, None);
    advance_to(&mut game, |game| {
        game.state.pending == (Pending::ResolutionMana { player: p })
    });
    assert!(game.keywords(spell).contains(Keywords::HASTE));
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(victim).zone, Zone::Battlefield);

    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    reject(&mut game, target(non_dinosaur));
    act(&mut game, target(dinosaur));
    reject(&mut game, target(dinosaur));
    act(&mut game, target(victim));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.object_view(dinosaur).damage, 0);
}

#[test]
fn kitesail_larcenist_turns_each_players_other_target_into_a_treasure_and_has_ward() {
    let p = Player::P1;
    let opponent = p.other();
    let mut game = scenario(p);
    let ours = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let larcenist = named(&mut game, p, "Kitesail Larcenist", Zone::Hand);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, larcenist, None);
    pass_pair(&mut game);
    assert!(game.keywords(larcenist).contains(Keywords::FLYING));
    reject(&mut game, target(larcenist));
    reject(&mut game, target(theirs));
    act(&mut game, target(ours));
    act(&mut game, target(theirs));
    settle(&mut game);
    for card in [ours, theirs] {
        assert_eq!(game.object_view(card).kinds, 16);
        assert!(game.has_subtype(card, "Treasure"));
    }
    assert!(
        !game
            .legal_actions()
            .contains(&Action::TapMana { card: ours })
    );
    let mut mana = game.clone();
    let production = [0, 0, 0, 1, 0, 0];
    let before = mana.state.players[p.index()].mana[3];
    let grantor = mana.object_ref(larcenist);
    act(
        &mut mana,
        Action::TapGrantedMana {
            card: ours,
            grantor,
            production,
        },
    );
    assert_eq!(mana.object_view(ours).zone, Zone::Graveyard);
    assert_eq!(mana.state.players[p.index()].mana[3], before + 1);

    game.state.players[opponent.index()].mana = production;
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(larcenist)));
    if !game.state.stack.is_empty() {
        pass_pair(&mut game);
        if game
            .legal_actions()
            .contains(&Action::DeclineResolutionMana)
        {
            act(&mut game, Action::DeclineResolutionMana);
        }
    }
    settle(&mut game);
    assert_eq!(game.object_view(larcenist).damage, 0);
    game.move_object(larcenist, Zone::Hand);
    assert!(game.has_subtype(ours, "Elf"));
    assert!(!game.has_subtype(theirs, "Treasure"));
}

#[test]
fn kutzils_flanker_resolves_each_of_its_three_entry_modes() {
    let p = Player::P1;
    let mut game = scenario(p);
    let departed = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.move_object(departed, Zone::Hand);
    let late = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Shock", Zone::Graveyard);
    let spell = named(&mut game, p, "Kutzil's Flanker", Zone::Hand);
    game.state.active = p.other();
    game.state.step = Step::Upkeep;
    cast(&mut game, spell, None);
    pass_pair(&mut game);

    let mut counters = game.clone();
    act(&mut counters, Action::ChooseMode { mode: 0 });
    counters.move_object(late, Zone::Exile);
    settle(&mut counters);
    assert_eq!(counters.counter_count(spell, CounterKind::PlusOne), 2);

    let mut life = game.clone();
    act(&mut life, Action::ChooseMode { mode: 1 });
    pass_pair(&mut life);
    assert_eq!(life.state.players[p.index()].life, STARTING_LIFE + 2);
    assert_eq!(life.observe(p).library_choice.unwrap().remaining.len(), 2);
    assert!(life.observe(p.other()).library_choice.is_none());

    act(&mut game, Action::ChooseMode { mode: 2 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Exile);
}

#[test]
fn magmatic_galleon_deals_five_to_an_opposing_creature_and_crews_for_two() {
    let p = Player::P0;
    let mut game = scenario(p);
    let galleon = named(&mut game, p, "Magmatic Galleon", Zone::Hand);
    let crew = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let target_creature = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 0, 0, 2, 0, 3];
    cast(&mut game, galleon, None);
    pass_pair(&mut game);
    reject(&mut game, target(crew));
    act(&mut game, target(target_creature));
    settle(&mut game);
    assert_eq!(game.object_view(target_creature).zone, Zone::Graveyard);
    let treasures = |game: &Game| {
        game.state
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield && object.card == Card(5302))
            .count()
    };
    assert_eq!(treasures(&game), 1);
    assert_eq!(game.object_view(galleon).kinds, 16);
    act(&mut game, activate(galleon));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    reject(&mut game, Action::FinishCrew);
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.object_view(crew).tapped);
    assert_eq!(game.object_view(galleon).kinds, 1 | 16);
    assert_eq!(stats(&game, galleon), (5, 5));
}
