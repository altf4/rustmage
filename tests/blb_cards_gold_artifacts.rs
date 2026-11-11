//! Authored Bloomburrow artifacts and their exact choices and paid effects.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn reach(game: &mut Game, action: &Action) {
    for _ in 0..16 {
        if game.legal_actions().contains(action) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("choice {action:?} did not become legal");
}

fn announce_ability(game: &mut Game, source: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn fountainport_bell_may_search_and_its_paid_sacrifice_draws_the_selected_basic() {
    let player = Player::P0;
    let mut game = scenario(player);
    let basic = put(&mut game, player, "Island", Zone::Library);
    let nonbasic = put(&mut game, player, "Fabled Passage", Zone::Library);
    let bell = put(&mut game, player, "Fountainport Bell", Zone::Hand);
    cast(&mut game, bell, None);
    let accept = Action::ChooseOptional { accept: true };
    reach(&mut game, &accept);

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(
        declined.state.players[player.index()].library.last(),
        Some(&nonbasic)
    );

    act(&mut game, accept);
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    act(&mut game, Action::ChooseSearch { card: basic });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&basic)
    );
    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .any(|view| view.id == basic)
    );

    choose(&mut game, |action| {
        matches!(action, Action::Activate { card, ability: 0, .. }
            | Action::AnnounceActivate { card, ability: 0, .. } if *card == bell)
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(bell).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
}

#[test]
fn heirloom_epic_accepts_four_sick_creatures_and_only_activates_as_a_sorcery() {
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let epic = put(&mut game, player, "Heirloom Epic", Zone::Battlefield);
    let creatures = [(); 4].map(|_| put(&mut game, player, "Bear Cub", Zone::Battlefield));
    for creature in creatures {
        game.state.objects[creature as usize].summoning_sick = true;
    }
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let activate = Action::AnnounceActivate {
        card: epic,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.step = Step::BeginCombat;
    reject(&mut game, activate.clone());
    game.state.step = Step::Main1;
    act(&mut game, activate);
    act(&mut game, Action::BeginCreatureTapPayment);
    let opposing_tap = Action::ChooseCreatureTapPayment {
        object: game.object_ref(opponent),
    };
    reject(&mut game, opposing_tap);
    for creature in creatures {
        let choose_creature = Action::ChooseCreatureTapPayment {
            object: game.object_ref(creature),
        };
        act(&mut game, choose_creature);
    }
    act(&mut game, Action::PayCosts { payment: [0; 6] });
    settle(&mut game);
    assert!(game.object_view(epic).tapped);
    assert!(
        creatures
            .into_iter()
            .all(|creature| game.object_view(creature).tapped)
    );
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn patchwork_banner_chooses_type_before_entry_and_makes_any_one_color() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let bears = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let banner = put(&mut game, player, "Patchwork Banner", Zone::Hand);
    cast(&mut game, banner, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(banner).zone, Zone::Stack);
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Bird").unwrap()),
        },
    );
    assert_eq!(stats(&game, hawk), (2, 2));
    assert_eq!(stats(&game, changeling), (3, 2));
    assert_eq!(stats(&game, opponent), (1, 1));
    assert_eq!(stats(&game, bears), (2, 2));

    let before = game.state.players[player.index()].mana;
    choose(&mut game, |action| {
        matches!(action, Action::ActivateMana { card, production, .. }
            if *card == banner && *production == [0, 0, 0, 1, 0, 0])
    });
    assert_eq!(game.state.players[player.index()].mana[3], before[3] + 1);
    assert!(game.object_view(banner).tapped);
}

#[test]
fn starforged_sword_separately_gifts_a_tapped_fish_and_attaches_to_an_exact_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let flyer = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let spare = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
    let sword = put(&mut game, player, "Starforged Sword", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: sword });
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseGift { beneficiary: None });
    pay(&mut declined);
    settle(&mut declined);
    assert!(declined.controlled_token_ids(opponent).next().is_none());
    assert_eq!(declined.object_view(sword).attached_to, None);

    act(
        &mut game,
        Action::ChooseGift {
            beneficiary: Some(opponent),
        },
    );
    pay(&mut game);
    assert!(game.controlled_token_ids(opponent).next().is_none());
    let target = Action::ChooseTarget {
        target: Target::Card(flyer),
    };
    reach(&mut game, &target);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    act(&mut game, target);
    settle(&mut game);
    let fish = game.controlled_token_ids(opponent).next().unwrap();
    assert!(game.object_view(fish).tapped);
    assert!(game.has_subtype(fish, "Fish"));
    assert_eq!(game.object_view(fish).colors, 2);
    assert_eq!(stats(&game, fish), (1, 1));
    assert_eq!(
        game.object_view(sword).attached_to,
        Some(game.object_ref(flyer))
    );
    assert_eq!(stats(&game, flyer), (4, 4));
    assert!(!game.keywords(flyer).contains(Keywords::FLYING));
    assert!(game.keywords(flyer).contains(Keywords::LIFELINK));

    announce_ability(&mut game, sword, 0);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(spare),
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut game);
    assert!(game.keywords(flyer).contains(Keywords::FLYING));
    assert_eq!(stats(&game, spare), (5, 5));
}

#[test]
fn tangle_tumbler_has_an_independent_tap_token_animation_and_paid_counter_ability() {
    let player = Player::P1;
    let mut game = scenario(player);
    let tumbler = put(&mut game, player, "Tangle Tumbler", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.create_tokens(player, Card::from_catalogue_id(5317), 1, false);
    game.create_tokens(player, Card::from_catalogue_id(5304), 1, false);
    let tokens: Vec<_> = game.controlled_token_ids(player).collect();
    assert_eq!(tokens.len(), 2);
    assert!(!game.is_creature(tumbler));
    announce_ability(&mut game, tumbler, 1);
    for token in tokens.iter().copied() {
        let choice = Action::ChooseAdditionalCost {
            object: game.object_ref(token),
        };
        act(&mut game, choice);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert!(
        tokens
            .into_iter()
            .all(|token| game.object_view(token).tapped)
    );
    assert!(game.is_creature(tumbler));
    assert!(game.keywords(tumbler).contains(Keywords::VIGILANCE));
    assert_eq!(stats(&game, tumbler), (6, 6));
    assert!(!game.object_view(tumbler).tapped);

    announce_ability(&mut game, tumbler, 0);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut game);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    game.finish_cleanup();
    assert!(!game.is_creature(tumbler));
    assert!(game.object_view(tumbler).tapped);
}
