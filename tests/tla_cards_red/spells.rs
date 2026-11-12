use crate::{reach, support::*};
use rustmage_core::{cards::*, engine::*};

fn choose_targets(game: &mut Game, targets: &[Target]) {
    for &target in targets {
        act(game, Action::ChooseTarget { target });
    }
}

fn modal(game: &mut Game, spell: CardId, mode: u16, targets: &[Target]) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    choose_targets(game, targets);
    pay(game);
}

#[test]
fn bumi_bash_counts_lands_at_resolution_and_distinguishes_land_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    let bear = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let basic = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let nonbasic = put(
        &mut game,
        player.other(),
        "Restless Cottage",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Bumi Bash", Zone::Hand);
    let mut damage = game.clone();
    modal(&mut damage, spell, 0, &[Target::Card(bear)]);
    put(&mut damage, player, "Island", Zone::Battlefield);
    settle(&mut damage);
    assert_eq!(damage.object_view(bear).zone, Zone::Graveyard);

    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 1 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(basic),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(nonbasic),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(nonbasic).zone, Zone::Graveyard);
}

#[test]
fn how_to_start_a_riot_resolves_each_independent_target_group() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bearer = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "How to Start a Riot", Zone::Hand);
    announce(
        &mut game,
        spell,
        &[Target::Card(bearer), Target::Player(player)],
    );
    let mut lost = game.clone();
    lost.move_object(bearer, Zone::Exile);
    settle(&mut lost);
    assert_eq!(stats(&lost, first), (4, 2));
    settle(&mut game);
    assert!(game.keywords(bearer).contains(Keywords::MENACE));
    assert_eq!(stats(&game, bearer), (2, 2));
    assert_eq!(stats(&game, first), (4, 2));
    assert_eq!(stats(&game, second), (5, 3));
}

#[test]
fn iroh_demonstrates_the_opposing_batch_and_the_single_creature_modes() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let second = put(
        &mut game,
        player.other(),
        "Llanowar Elves",
        Zone::Battlefield,
    );
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Iroh's Demonstration", Zone::Hand);
    let mut single = game.clone();
    modal(&mut single, spell, 1, &[Target::Card(own)]);
    settle(&mut single);
    assert_eq!(single.object_view(own).zone, Zone::Graveyard);
    assert_eq!(single.state.objects[first as usize].damage, 0);
    modal(&mut game, spell, 0, &[]);
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.state.objects[first as usize].damage, 1);
    assert_eq!(game.state.objects[own as usize].damage, 0);
}

#[test]
fn jets_brainwashing_applies_the_kicked_control_branch_and_always_creates_a_clue() {
    let player = Player::P1;
    for kicked in [false, true] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        game.state.objects[target as usize].tapped = true;
        let spell = put(&mut game, player, "Jet's Brainwashing", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseKicker { pay: kicked });
        choose_targets(&mut game, &[Target::Card(target)]);
        pay(&mut game);
        settle(&mut game);
        assert!(game.keywords(target).contains(Keywords::CANT_BLOCK));
        assert_eq!(game.keywords(target).contains(Keywords::HASTE), kicked);
        let creature = game.object_view(target);
        assert_eq!(
            creature.controller,
            if kicked { player } else { player.other() }
        );
        assert_eq!(creature.tapped, !kicked);
        assert_eq!(count_tokens(&game, player, "Clue"), 1);
        game.state.step = Step::End;
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.object_view(target).controller, player.other());
        assert!(!game.keywords(target).contains(Keywords::CANT_BLOCK));
    }
}

#[test]
fn price_of_freedom_leaves_the_optional_basic_search_with_the_targets_controller() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let artifact = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let own_land = put(&mut game, player, "Forest", Zone::Battlefield);
    let opposing_land = put(&mut game, opponent, "Mountain", Zone::Battlefield);
    let basic = put(&mut game, opponent, "Island", Zone::Library);
    let spell = put(&mut game, player, "Price of Freedom", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    assert!(game.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(opposing_land)
    }));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own_land),
        },
    );
    choose_targets(&mut game, &[Target::Card(artifact)]);
    pay(&mut game);
    let accept = Action::ChooseOptional { accept: true };
    reach(&mut game, |g| g.legal_actions().contains(&accept));
    assert_eq!(game.actor(), Some(opponent));
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(
        decline.state.players[player.index()].cards_drawn_this_turn,
        1
    );
    act(&mut game, accept);
    assert!(game.observe(player).search_choice.is_none());
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(basic).controller, opponent);
    assert!(game.object_view(basic).tapped);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

fn pay_redirect(game: &mut Game, player: Player, target: CardId, life: bool) {
    for _ in 0..20 {
        if game.state.pending == Pending::Priority && game.state.stack.len() == 2 {
            return;
        }
        let additional = game.observe(player).cast_costs.and_then(|costs| {
            costs
                .clauses
                .iter()
                .position(|row| row.clause == CastCostClause::Additional && !row.paid)
        });
        choose(game, |action| match action {
            Action::ChooseSpellCost { cost } => *cost == u16::from(!life),
            Action::ChooseTarget {
                target: Target::Spell(id),
            } => *id == target,
            Action::ChooseCastCost { clause } => {
                additional.is_none_or(|index| usize::from(*clause) == index)
            }
            Action::ChooseAdditionalLife => life,
            Action::BeginOrderedCosts | Action::FinishTargets | Action::PayCosts { .. } => true,
            _ => false,
        });
    }
    panic!("redirect payment did not finish");
}

#[test]
fn redirect_lightning_alternates_between_life_and_mana_and_changes_the_target() {
    let player = Player::P0;
    for life in [true, false] {
        let mut game = scenario(player);
        let shot = put(&mut game, player.other(), "Lightning Strike", Zone::Hand);
        game.state.priority = player.other();
        cast(&mut game, shot, Some(Target::Player(player)));
        if game.actor() != Some(player) {
            act(&mut game, Action::Pass);
        }
        let spell = put(&mut game, player, "Redirect Lightning", Zone::Hand);
        let action = Action::AnnounceCast { card: spell };
        game.state.players[player.index()].mana = [0, 0, 0, if life { 1 } else { 3 }, 0, 0];
        game.state.players[player.index()].life = if life { 20 } else { 4 };
        let initial = game.state.players[player.index()].life;
        act(&mut game, action);
        pay_redirect(&mut game, player, shot, life);
        assert_eq!(
            game.state.players[player.index()].life,
            initial - if life { 5 } else { 0 }
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        act(
            &mut game,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(player.other()),
            },
        );
        act(&mut game, Action::FinishStackTargets);
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 17);
    }
}

#[test]
fn solstice_revelations_flashback_uses_the_strict_mountain_bound() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Solstice Revelations", Zone::Graveyard);
    let found = put(&mut game, player, "Shock", Zone::Library);
    let top = put(&mut game, player, "Forest", Zone::Library);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    let mut equal = game.clone();
    act(&mut equal, Action::AnnounceFlashback { card: spell });
    pay(&mut equal);
    act(&mut equal, Action::Pass);
    act(&mut equal, Action::Pass);
    let blocked = equal.object_ref(found);
    reject(
        &mut equal,
        Action::CastExiledDuringResolution {
            object: blocked,
            face: CastFace::Normal,
        },
    );
    if equal
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut equal, Action::FinishResolutionCasts);
    }
    settle(&mut equal);
    assert_eq!(equal.object_view(found).zone, Zone::Hand);
    assert_eq!(equal.object_view(top).zone, Zone::Exile);

    act(&mut game, Action::AnnounceFlashback { card: spell });
    pay(&mut game);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let object = game.object_ref(found);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    choose_targets(&mut game, &[Target::Player(player.other())]);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.object_view(found).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn the_last_agni_kai_retains_red_with_or_without_a_successful_fight() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fighter = put(&mut game, player, "Shivan Dragon", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "The Last Agni Kai", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 5, 0, 0];
    announce(
        &mut game,
        spell,
        &[Target::Card(fighter), Target::Card(opponent)],
    );
    let mut one_target = game.clone();
    one_target.move_object(fighter, Zone::Exile);
    settle(&mut one_target);
    assert_eq!(one_target.object_view(opponent).zone, Zone::Battlefield);
    assert_eq!(one_target.state.players[player.index()].mana[3], 3);
    settle(&mut game);
    assert_eq!(game.object_view(opponent).zone, Zone::Graveyard);
    assert_eq!(game.state.objects[fighter as usize].damage, 2);
    assert_eq!(game.state.players[player.index()].mana[3], 6);

    for (branch, red) in [(&mut game, 6), (&mut one_target, 3)] {
        branch.state.players[player.index()].mana[1] = 2;
        act(branch, Action::Pass);
        act(branch, Action::Pass);
        assert_eq!(
            branch.state.players[player.index()].mana,
            [0, 0, 0, red, 0, 0]
        );
        branch.state.step = Step::End;
        act(branch, Action::Pass);
        act(branch, Action::Pass);
        assert_eq!(branch.state.players[player.index()].mana, [0; 6]);
    }
}

fn count_tokens(game: &Game, player: Player, subtype: &str) -> usize {
    (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype(subtype)
        })
        .count()
}
