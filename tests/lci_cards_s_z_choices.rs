//! Discover, private opponent decisions, and printed zone changes.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(game: &mut Game, player: Player, name: &str) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), Zone::Hand)
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..16 {
        if ready(game) {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario stopped in {:?}", game.state.pending);
}

fn discover(game: &mut Game, player: Player) {
    advance(game, |g| {
        g.state.pending == Pending::ResolutionCasts { player }
    });
}

#[test]
fn walk_with_the_ancestors_returns_before_discover_and_allows_no_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Island", Zone::Graveyard);
    let instant = put(&mut game, player, "Opt", Zone::Graveyard);
    let enemy = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let found = put(&mut game, player, "Galvanic Giant", Zone::Library);
    let expensive = put(&mut game, player, "Gigantosaurus", Zone::Library);
    let spell = candidate(&mut game, player, "Walk with the Ancestors");
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(instant));
    reject(&mut game, target(enemy));

    let mut none = game.clone();
    finish_targets(&mut none);
    pay(&mut none);
    discover(&mut none, player);
    assert_eq!(none.object_view(land).zone, Zone::Graveyard);
    assert_eq!(
        none.observe(player).cascade.unwrap().discovered,
        Some(none.object_ref(found))
    );
    act(&mut none, Action::FinishResolutionCasts);
    assert_eq!(none.object_view(found).zone, Zone::Hand);

    act(&mut game, target(land));
    finish_targets(&mut game);
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(land, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.top_library_card(player), Some(expensive));
    assert_eq!(stale.object_view(found).zone, Zone::Library);

    discover(&mut game, player);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    let event = game.observe(player.other()).cascade.unwrap();
    assert_eq!(
        event.exiled,
        [expensive, found].map(|id| game.object_ref(id))
    );
    assert_eq!(event.discovered, Some(game.object_ref(found)));
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
}

#[test]
fn zoyowas_justice_uses_the_exact_permanents_mana_value_and_owner() {
    let player = Player::P1;
    let owner = player.other();
    for name in ["Goldvein Pick", "Bear Cub"] {
        let mut game = scenario(player);
        game.state.players[owner.index()].library.clear();
        let source = put(&mut game, owner, name, Zone::Battlefield);
        let land = put(&mut game, owner, "Forest", Zone::Battlefield);
        game.record_control(game.object_ref(source), player, None);
        let old = game.object_ref(source);
        let token = game.state.objects.len() as CardId;
        game.create_tokens(player, Card::from_catalogue_id(5302), 1, false);
        let spell = candidate(&mut game, player, "Zoyowa's Justice");
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, target(land));
        reject(&mut game, target(token));
        act(&mut game, target(source));
        finish_targets(&mut game);
        pay(&mut game);
        discover(&mut game, owner);
        assert_eq!(game.object_ref(source).generation, old.generation + 2);
        assert_eq!(game.object_view(source).zone, Zone::Exile);
        assert_eq!(
            game.observe(player).cascade.unwrap().discovered,
            Some(game.object_ref(source))
        );
        assert!(
            game.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: game.object_ref(source),
                    face: CastFace::Normal,
                })
        );
        act(&mut game, Action::FinishResolutionCasts);
        assert_eq!(game.object_view(source).zone, Zone::Hand);
        assert_eq!(game.object_view(source).owner, owner);
    }
}

#[test]
fn skullcap_snail_leaves_the_exile_choice_private_and_mandatory() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let snail = candidate(&mut game, player, "Skullcap Snail");
    let mut empty = game.clone();
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let other = put(&mut game, opponent, "Opt", Zone::Hand);
    let exile_from = Action::ChooseTarget {
        target: Target::Player(opponent),
    };
    for branch in [&mut game, &mut empty] {
        cast(branch, snail, None);
        advance(branch, |g| g.legal_actions().contains(&exile_from));
        reject(
            branch,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
        act(branch, exile_from.clone());
        finish_targets(branch);
    }
    settle(&mut empty);
    assert_eq!(empty.hand_count(opponent), 0);
    assert!(empty.observe(opponent).hand_choice.is_none());

    advance(&mut game, |g| g.observe(opponent).hand_choice.is_some());
    assert_eq!(stats(&game, snail), (1, 1));
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).hand_choice.is_none());
    assert!(!game.observe(opponent).hand_choice.unwrap().optional);
    reject(&mut game, Action::ChooseHandCard { object: None });
    let land_ref = game.object_ref(land);
    act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(land_ref),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(other).zone, Zone::Hand);
    assert!(game.observe(player).known_hand.is_empty());
}

#[test]
fn volatile_fault_pays_its_cost_and_rewards_either_opponent_search_decision() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let fault = candidate(&mut game, player, "Volatile Fault");
    let basic = put(&mut game, opponent, "Island", Zone::Library);
    let enemy_basic = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Crystal Grotto", Zone::Battlefield);
    let own = put(&mut game, player, "Crystal Grotto", Zone::Battlefield);
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, Action::PlayLand { card: fault });
    assert!(game.has_subtype(fault, "Cave"));
    assert!(!game.object_view(fault).tapped);
    game.state.step = Step::Upkeep;
    let colorless = [0, 0, 0, 0, 0, 1];
    act(
        &mut game,
        Action::ActivateMana {
            card: fault,
            ability: 0,
            payment: [0; 6],
            production: colorless,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(game.state.players[player.index()].mana, colorless);
    game.untap_objects(&[fault], untap_rules::UntapCause::Effect);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: fault,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, target(enemy_basic));
    reject(&mut game, target(own));
    act(&mut game, target(enemy));
    finish_targets(&mut game);
    pay(&mut game);
    assert_eq!(game.object_view(fault).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let accept = Action::ChooseOptional { accept: true };
    advance(&mut game, |g| g.legal_actions().contains(&accept));
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    let library = game.state.players[opponent.index()].library.clone();
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.state.players[opponent.index()].library, library);

    act(&mut game, accept);
    act(&mut game, Action::ChooseSearch { card: basic });
    if matches!(game.state.pending, Pending::EffectSearch { .. }) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(basic).zone, Zone::Battlefield);
    assert_eq!(game.object_view(basic).controller, opponent);
    for branch in [game, decline] {
        let treasures = branch
            .observe(player)
            .objects
            .into_iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && o.is_token()
                    && o.has_subtype("Treasure")
            })
            .count();
        assert_eq!(treasures, 1);
    }
}
