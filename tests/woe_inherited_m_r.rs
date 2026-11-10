//! Paid gameplay outcomes for inherited Wilds of Eldraine cards from M through R.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD: Card = Card::from_catalogue_id(5246);
const RAT: Card = Card::from_catalogue_id(5323);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn tokens(game: &Game, controller: Player, recipe: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield
                && object.controller == controller
                && object.is_token()
                && object.card == recipe
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

fn role(game: &Game, controller: Player, recipient: CardId, kind: RoleKind) -> CardId {
    let recipient = game.object_ref(recipient);
    game.state
        .objects
        .iter()
        .position(|object| {
            object.zone == Zone::Battlefield
                && object.controller == controller
                && object.card.role() == Some(kind)
                && object.attached_to == Some(recipient)
        })
        .expect("role on the exact recipient") as CardId
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    game.begin_step(Step::End);
    for _ in 0..12 {
        if game.state.turn != turn {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("turn did not reach cleanup");
}

fn reach_attackers(game: &mut Game) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::Attackers);
}

fn attack(game: &mut Game, card: CardId) {
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

#[test]
fn ride_the_rails_and_desperate_parry_pay_the_instant_face_then_the_exiled_creature() {
    for (name, player, altered, front, keywords) in [
        (
            "Minecart Daredevil",
            Player::P0,
            (5, 4),
            (4, 2),
            Keywords::NONE,
        ),
        (
            "Obyra's Attendants",
            Player::P1,
            (-1, 3),
            (3, 4),
            Keywords::FLYING,
        ),
    ] {
        let mut game = scenario(player.other());
        let spell = put(&mut game, player, name, Zone::Hand);
        let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
        act(&mut game, Action::Pass);
        reject(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::AnnounceAdventure { card: spell });
        act(&mut game, target(giant));
        pay(&mut game);

        let mut stale = game.clone();
        act(&mut stale, Action::Pass);
        cast(&mut stale, bounce, Some(Target::Card(giant)));
        settle(&mut stale);
        assert_eq!(stale.object_view(giant).zone, Zone::Hand);
        assert_eq!(stale.object_view(spell).zone, Zone::Graveyard);

        settle(&mut game);
        assert_eq!(stats(&game, giant), altered);
        assert_eq!(game.object_view(spell).zone, Zone::Exile);
        let exiled = game.object_ref(spell);
        reject(&mut game, Action::AnnounceExile { card: spell });
        finish_turn(&mut game);
        assert_eq!(stats(&game, giant), (3, 3));
        game.begin_step(Step::Main1);
        game.state.players[player.index()].mana = [8; 6];
        act(&mut game, Action::AnnounceExile { card: spell });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        assert_ne!(game.object_ref(spell), exiled);
        assert_eq!(stats(&game, spell), front);
        assert_eq!(game.object_view(spell).keywords, keywords);
    }
}

#[test]
fn free_the_fae_takes_one_of_three_eligible_milled_cards_and_exiles_the_prankster() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Picklock Prankster", Zone::Hand);
    let old = put(&mut game, player, "Shock", Zone::Graveyard);
    let adventurer = put(&mut game, player, "Minecart Daredevil", Zone::Library);
    let faerie = put(&mut game, player, "Obyra's Attendants", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let sorcery = put(&mut game, player, "Divination", Zone::Library);
    let before = game.state.players[player.index()].library.len();

    act(&mut game, Action::AnnounceAdventure { card: source });
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), before - 4);
    assert_eq!(game.state.pending, Pending::MilledSelection { player });
    assert!(game.observe(player.other()).milled_choice.is_none());
    reject(&mut game, Action::FinishMilledSelection);
    for invalid in [adventurer, old] {
        let object = game.object_ref(invalid);
        reject(&mut game, Action::SelectMilled { object });
    }

    let eligible = [faerie, instant, sorcery];
    for selected in eligible {
        let mut branch = game.clone();
        let object = branch.object_ref(selected);
        act(&mut branch, Action::SelectMilled { object });
        settle(&mut branch);
        for card in [faerie, instant, sorcery, adventurer, old] {
            assert_eq!(
                branch.object_view(card).zone,
                if card == selected {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
        }
        assert!(
            branch
                .observe(player.other())
                .known_hand
                .contains(&branch.object_ref(selected))
        );
        assert_eq!(branch.object_view(source).zone, Zone::Exile);
        act(&mut branch, Action::AnnounceExile { card: source });
        pay(&mut branch);
        settle(&mut branch);
        let front = branch.object_view(source);
        assert_eq!(front.zone, Zone::Battlefield);
        assert_eq!((front.power, front.toughness), (1, 3));
        assert!(
            front
                .keywords
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
    }

    let mut empty = scenario(player);
    let source = put(&mut empty, player, "Picklock Prankster", Zone::Hand);
    let before = empty.state.players[player.index()].library.len();
    act(&mut empty, Action::AnnounceAdventure { card: source });
    pay(&mut empty);
    settle(&mut empty);
    assert_eq!(
        empty.state.players[player.index()].library.len(),
        before - 4
    );
    assert_eq!(empty.object_view(source).zone, Zone::Exile);
}

#[test]
fn pest_problem_makes_two_no_block_rats_then_trainee_has_first_strike_on_its_turn() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let trainee = put(&mut game, player, "Ratcatcher Trainee", Zone::Hand);
    let bear = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceAdventure { card: trainee });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(trainee).zone, Zone::Exile);
    let rats = tokens(&game, player, RAT);
    assert_eq!(rats.len(), 2);
    for rat in &rats {
        let rat = game.object_view(*rat);
        assert_eq!((rat.power, rat.toughness, rat.colors), (1, 1, 4));
        assert!(rat.has_subtype("Rat") && rat.keywords.contains(Keywords::CANT_BLOCK));
    }
    reach_attackers(&mut game);
    attack(&mut game, bear);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: rats[0],
            attacker: bear,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    finish_turn(&mut game);

    game.begin_step(Step::Main1);
    game.state.players[player.index()].mana = [8; 6];
    act(&mut game, Action::AnnounceExile { card: trainee });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, trainee), (2, 1));
    assert!(
        game.object_view(trainee)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    finish_turn(&mut game);
    assert!(
        !game
            .object_view(trainee)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
}

#[test]
fn coralsmith_can_pay_three_activations_and_scries_two_on_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let smith = put(&mut game, player, "Merfolk Coralsmith", Zone::Hand);
    let second = put(&mut game, player, "Island", Zone::Library);
    let first = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, smith, None);
    settle(&mut game);
    assert_eq!(stats(&game, smith), (2, 3));
    for power in [3, 4] {
        act(&mut game, activate(smith, 0));
        assert_eq!(game.mana_payment_domain().unwrap().generic, 1);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, smith), (power, 5 - power));
    }
    let mut cleanup = game.clone();
    finish_turn(&mut cleanup);
    assert_eq!(stats(&cleanup, smith), (2, 3));

    act(&mut game, activate(smith, 0));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(smith).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let look = game.observe(player).library_choice.unwrap();
    assert_eq!(
        look.remaining
            .iter()
            .map(|card| card.id)
            .collect::<Vec<_>>(),
        [first, second]
    );
    assert!(game.observe(player.other()).library_choice.is_none());
    for (card, destination) in [
        (first, LibraryDestination::Bottom),
        (second, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    assert_eq!(game.top_library_card(player), Some(second));
}

#[test]
fn merry_bards_resolves_its_optional_payment_before_choosing_a_young_hero() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let bard = put(&mut game, player, "Merry Bards", Zone::Hand);
        let hero = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
        cast(&mut game, bard, None);
        pass_pair(&mut game);
        assert!(game.state.stack.last().unwrap().targets.is_empty());
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::ResolutionMana { player });
        let mut declined = game.clone();
        act(&mut declined, Action::DeclineResolutionMana);
        settle(&mut declined);
        assert!(
            declined
                .observe(player)
                .objects
                .iter()
                .all(|object| !object.is_token())
        );

        let payment = Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 1],
        };
        act(&mut game, payment);
        reject(&mut game, target(enemy));
        act(&mut game, target(hero));
        let mut stale = game.clone();
        act(&mut stale, Action::Pass);
        cast(&mut stale, bounce, Some(Target::Card(hero)));
        settle(&mut stale);
        assert!(
            stale
                .observe(player)
                .objects
                .iter()
                .all(|object| !object.is_token())
        );

        settle(&mut game);
        role(&game, player, hero, RoleKind::YoungHero);
        reach_attackers(&mut game);
        attack(&mut game, hero);
        settle(&mut game);
        assert_eq!(game.counter_count(hero, CounterKind::PlusOne), 1);
    }
}

#[test]
fn mintstrosity_only_creates_food_for_dying_and_the_food_can_be_paid_and_sacrificed() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Mintstrosity", Zone::Hand);
    let kill = put(&mut game, player, "Murder", Zone::Hand);
    let exile = put(&mut game, player, "Final Vengeance", Zone::Hand);
    let offering = put(&mut game, player, "A Tale for the Ages", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 1));
    assert!(tokens(&game, player, FOOD).is_empty());

    let mut banished = game.clone();
    // A paid exile spell with a cost of sacrificing a separate enchantment.
    act(&mut banished, Action::AnnounceCast { card: exile });
    act(&mut banished, target(source));
    let object = banished.object_ref(offering);
    act(&mut banished, Action::ChooseAdditionalCost { object });
    pay(&mut banished);
    settle(&mut banished);
    assert_eq!(banished.object_view(source).zone, Zone::Exile);
    assert!(tokens(&banished, player, FOOD).is_empty());

    cast(&mut game, kill, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let food = tokens(&game, player, FOOD);
    assert_eq!(food.len(), 1);
    act(&mut game, activate(food[0], 0));
    assert_eq!(game.mana_payment_domain().unwrap().generic, 2);
    pay(&mut game);
    assert_ne!(game.object_view(food[0]).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn misleading_motes_lets_the_creatures_owner_place_it_on_top_or_bottom() {
    for player in [Player::P0, Player::P1] {
        let owner = player.other();
        let mut game = scenario(player);
        let source = put(&mut game, player, "Misleading Motes", Zone::Hand);
        let victim = put(&mut game, owner, "Hill Giant", Zone::Battlefield);
        let top = game.top_library_card(owner).unwrap();
        let prior = game.object_ref(victim);
        game.change_controller(prior, player);
        cast(&mut game, source, Some(Target::Card(victim)));
        pass_pair(&mut game);
        assert_eq!(game.state.pending, Pending::TargetLibrary { player: owner });
        assert_eq!(game.observe(owner).target_library.unwrap().target, prior);
        assert!(game.observe(player).target_library.is_none());
        reject(
            &mut game,
            Action::ChooseLibraryDestination {
                object: prior,
                destination: LibraryDestination::SecondFromTop,
            },
        );

        for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
            let mut branch = game.clone();
            act(
                &mut branch,
                Action::ChooseLibraryDestination {
                    object: prior,
                    destination,
                },
            );
            settle(&mut branch);
            assert_eq!(branch.object_view(victim).zone, Zone::Library);
            assert_ne!(branch.object_ref(victim), prior);
            let library = &branch.state.players[owner.index()].library;
            if destination == LibraryDestination::Top {
                assert_eq!(library.last(), Some(&victim));
            } else {
                assert_eq!(library.first(), Some(&victim));
                assert_eq!(library.last(), Some(&top));
            }
            let movement = branch.observe(player).last_target_library_move.unwrap();
            assert_eq!(movement.owner, owner);
            assert_eq!(movement.destination, destination);
        }
    }
}

#[test]
fn plunge_into_winter_and_quick_study_distinguish_optional_targets_and_private_draws() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Plunge into Winter", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let bounce = put(&mut game, player.other(), "Unsummon", Zone::Hand);
    let bottom = put(&mut game, player, "Island", Zone::Library);
    let top = put(&mut game, player, "Shock", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: source });
    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    pay(&mut zero);
    act(&mut game, target(victim));
    pay(&mut game);

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    cast(&mut stale, bounce, Some(Target::Card(victim)));
    settle(&mut stale);
    assert_eq!(stale.top_library_card(player), Some(top));
    assert_eq!(stale.object_view(top).zone, Zone::Library);

    for (branch, tapped) in [(&mut zero, false), (&mut game, true)] {
        pass_pair(branch);
        assert_eq!(branch.object_view(victim).tapped, tapped);
        assert_eq!(
            branch.observe(player).library_choice.unwrap().remaining[0].id,
            top
        );
        assert!(branch.observe(player.other()).library_choice.is_none());
        act(
            branch,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(branch);
        assert_eq!(branch.object_view(bottom).zone, Zone::Hand);
        assert!(
            !branch
                .observe(player.other())
                .known_hand
                .contains(&branch.object_ref(bottom))
        );
    }

    let mut game = scenario(player.other());
    let study = put(&mut game, player, "Quick Study", Zone::Hand);
    let still_hidden = put(&mut game, player, "Forest", Zone::Library);
    let drawn = [
        put(&mut game, player, "Island", Zone::Library),
        put(&mut game, player, "Shock", Zone::Library),
    ];
    act(&mut game, Action::Pass);
    announce(&mut game, study, &[]);
    settle(&mut game);
    for card in drawn {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
        assert!(
            !game
                .observe(player.other())
                .known_hand
                .contains(&game.object_ref(card))
        );
    }
    assert_eq!(game.top_library_card(player), Some(still_hidden));
}
