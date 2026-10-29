//! Unlinked tests for the isolated outside-game pool foundation.
use super::*;

fn config(rules: DeckRules) -> Config {
    Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 7,
        starting_player: Player::P0,
        deck_rules: rules,
    }
}
fn pool(mode: OutsideGameMode, a: Vec<Card>, b: Vec<Card>) -> OutsideGameConfig {
    OutsideGameConfig {
        mode,
        pools: [a, b],
    }
}
#[test]
fn outside_constructor_validates_combined_limits_and_preserves_old_constructor() {
    let shock = Card::from_name("Shock").unwrap();
    assert!(Game::new(config(DeckRules::Constructed)).is_ok());
    let mut cfg = config(DeckRules::Constructed);
    cfg.decks[0].extend([shock; 4]);
    assert!(
        Game::new_with_outside(
            cfg.clone(),
            pool(OutsideGameMode::ConstructedSideboard, vec![shock], vec![])
        )
        .is_err()
    );
    cfg.decks[0].pop();
    let g = Game::new_with_outside(
        cfg,
        pool(OutsideGameMode::ConstructedSideboard, vec![shock], vec![]),
    )
    .unwrap();
    assert_eq!(g.observe(Player::P1).sideboard_counts, Some([1, 0]));
    assert!(g.observe(Player::P1).owned_outside.is_none());
    assert!(
        Game::new_with_outside(
            config(DeckRules::Constructed),
            pool(
                OutsideGameMode::ConstructedSideboard,
                vec![Card::Forest; 16],
                vec![]
            )
        )
        .is_err()
    );
    assert!(
        Game::new_with_outside(
            config(DeckRules::Freeform),
            pool(OutsideGameMode::ConstructedSideboard, vec![], vec![])
        )
        .is_err()
    );
    assert!(
        Game::new_with_outside(
            config(DeckRules::Freeform),
            pool(
                OutsideGameMode::CasualCollection,
                vec![Card::Forest; 16],
                vec![]
            )
        )
        .is_ok()
    );
}
#[test]
fn outside_pool_is_hidden_and_owned_by_each_snapshot_and_fork() {
    let shock = Card::from_name("Shock").unwrap();
    let mut g = Game::new_with_outside(
        config(DeckRules::Freeform),
        pool(OutsideGameMode::CasualCollection, vec![shock], vec![]),
    )
    .unwrap();
    let owner = g.observe(Player::P0);
    assert_eq!(owner.owned_outside.unwrap()[0], Some(vec![shock]));
    let opponent = g.observe(Player::P1);
    assert!(opponent.owned_outside.is_none());
    assert!(!serde_json::to_string(&opponent).unwrap().contains("Shock"));
    let snapshot = g.snapshot();
    let fork = g.clone();
    g.state.outside_game.as_mut().unwrap().pools[0].clear();
    assert_eq!(
        fork.state.outside_game.as_ref().unwrap().pools[0],
        vec![shock]
    );
    g.restore(&snapshot).unwrap();
    assert_eq!(g.state.outside_game.as_ref().unwrap().pools[0], vec![shock]);
    assert_eq!(g.snapshot(), snapshot);
}

fn outside_scenario(player: Player, cards: Vec<Card>) -> Game {
    let mut g = scenario(player);
    let mut pools = [vec![], vec![]];
    pools[player.index()] = cards;
    g.state.outside_game = Some(Box::new(OutsideGameState {
        mode: OutsideGameMode::CasualCollection,
        pools,
        choice: None,
        pending_entry: None,
        pool_versions: [0; 2],
        next_choice_id: 0,
        known: [Vec::new(), Vec::new()],
    }));
    g
}
fn cast_extrapolate(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Shock", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name("Extrapolate the Impossible").unwrap();
    act(g, Action::AnnounceCast { card: id });
    pay(g);
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::OutsideGameChoice { .. }) {
            break;
        }
        act(g, Action::Pass);
    }
    id
}
#[test]
fn outside_private_pair_reveals_then_opponent_chooses_transactionally() {
    for player in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let forest = Card::Forest;
        let mut g = outside_scenario(player, vec![shock, bear, forest]);
        cast_extrapolate(&mut g, player);
        let before = g.snapshot();
        assert!(
            g.observe(player)
                .outside_choice
                .as_ref()
                .unwrap()
                .pool
                .is_some()
        );
        assert!(
            g.observe(player.other())
                .outside_choice
                .as_ref()
                .unwrap()
                .pool
                .is_none()
        );
        assert!(
            g.apply(player.other(), Action::ChooseOutsideCard { index: 0 })
                .is_err()
        );
        assert_eq!(g.snapshot(), before);
        for _ in 0..2 {
            act(&mut g, Action::ChooseOutsideCard { index: 0 });
            let partial = g.snapshot();
            assert!(
                g.observe(player.other())
                    .outside_choice
                    .as_ref()
                    .unwrap()
                    .first
                    .is_none()
            );
            assert!(
                g.apply(player, Action::ChooseOutsideCard { index: 0 })
                    .is_err()
            );
            assert_eq!(g.snapshot(), partial);
            act(&mut g, Action::ChooseOutsideCard { index: 1 });
            for viewer in [player, player.other()] {
                assert_eq!(
                    g.observe(viewer).outside_choice.as_ref().unwrap().revealed,
                    Some([shock, bear])
                );
            }
            assert_eq!(g.actor(), Some(player.other()));
            let revealed = g.snapshot();
            assert!(
                g.apply(player, Action::ChooseOutsideRevealed { index: 0 })
                    .is_err()
            );
            assert_eq!(g.snapshot(), revealed);
            act(&mut g, Action::ChooseOutsideRevealed { index: 1 });
            let state = g.state.outside_game.as_ref().unwrap();
            assert_eq!(state.pools[player.index()], vec![shock, forest]);
            assert_eq!(
                g.observe(player.other()).known_outside.unwrap()[player.index()],
                vec![shock]
            );
            let object = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, o)| {
                    (o.card == bear && o.zone == Zone::Hand && o.owner == player)
                        .then_some(g.object_ref(id as CardId))
                })
                .unwrap();
            assert!(g.observe(player.other()).known_hand.contains(&object));
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn outside_decline_cancel_and_duplicate_names_preserve_pool_for_next_spell() {
    for player in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let mut g = outside_scenario(player, vec![shock, shock, bear]);
        cast_extrapolate(&mut g, player);
        act(&mut g, Action::ChooseOutsideCard { index: 0 });
        let snapshot = g.snapshot();
        assert!(
            g.apply(player, Action::ChooseOutsideCard { index: 1 })
                .is_err()
        );
        assert_eq!(g.snapshot(), snapshot);
        act(&mut g, Action::CancelOutsideCard);
        act(&mut g, Action::DeclineOutsideCards);
        assert_eq!(
            g.state.outside_game.as_ref().unwrap().pools[player.index()],
            vec![shock, shock, bear]
        );
        settle(&mut g);
        cast_extrapolate(&mut g, player);
        act(&mut g, Action::ChooseOutsideCard { index: 0 });
        act(&mut g, Action::ChooseOutsideCard { index: 2 });
        act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
        assert_eq!(
            g.state.outside_game.as_ref().unwrap().pools[player.index()],
            vec![shock, bear]
        );
    }
}

#[test]
fn outside_empty_or_duplicate_only_pool_never_offers_an_incomplete_pair() {
    for player in [Player::P0, Player::P1] {
        for cards in [vec![], vec![Card::Forest], vec![Card::Forest, Card::Forest]] {
            let mut g = outside_scenario(player, cards.clone());
            let source = put(&mut g, player, "Shock", Zone::Hand);
            g.state.objects[source as usize].card =
                Card::from_name("Extrapolate the Impossible").unwrap();
            act(&mut g, Action::AnnounceCast { card: source });
            pay(&mut g);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::OutsideGameChoice { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(
                matches!(g.state.pending, Pending::OutsideGameChoice { player: p } if p == player)
            );
            assert!(!g.legal_actions().iter().any(|a| matches!(
                a,
                Action::ChooseOutsideCard { .. } | Action::ChooseOutsideBasic { .. }
            )));
            assert!(
                g.observe(player.other())
                    .outside_choice
                    .as_ref()
                    .unwrap()
                    .pool
                    .is_none()
            );
            act(&mut g, Action::DeclineOutsideCards);
            settle(&mut g);
            assert!(g.state.outside_game.as_ref().unwrap().choice.is_none());
            assert_eq!(
                g.state.outside_game.as_ref().unwrap().pools[player.index()],
                cards
            );
        }
    }
}

#[test]
fn outside_limited_virtual_basics_are_unlimited_and_do_not_deplete_physical_pool() {
    for player in [Player::P0, Player::P1] {
        let mut g = outside_scenario(player, vec![]);
        g.state.outside_game.as_mut().unwrap().mode = OutsideGameMode::LimitedSideboard;
        let island = Card::from_name("Island").unwrap();
        let plains = Card::from_name("Plains").unwrap();
        for _ in 0..2 {
            cast_extrapolate(&mut g, player);
            assert!(
                g.legal_actions()
                    .contains(&Action::ChooseOutsideBasic { card: island })
            );
            let wastes = Card::from_name("Wastes").unwrap();
            assert!(
                !g.legal_actions()
                    .contains(&Action::ChooseOutsideBasic { card: wastes })
            );
            act(&mut g, Action::ChooseOutsideBasic { card: island });
            let snapshot = g.snapshot();
            assert!(
                g.apply(player, Action::ChooseOutsideBasic { card: island })
                    .is_err()
            );
            assert_eq!(g.snapshot(), snapshot);
            act(&mut g, Action::ChooseOutsideBasic { card: plains });
            assert_eq!(
                g.observe(player.other())
                    .outside_choice
                    .as_ref()
                    .unwrap()
                    .revealed,
                Some([island, plains])
            );
            act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
            assert!(g.state.outside_game.as_ref().unwrap().pools[player.index()].is_empty());
            settle(&mut g);
        }
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.card == island && o.owner == player && o.zone == Zone::Hand)
                .count(),
            2
        );
    }
}

#[test]
fn outside_limited_requires_compatible_rules_and_minimum_deck() {
    let empty = pool(OutsideGameMode::LimitedSideboard, vec![], vec![]);
    assert!(Game::new_with_outside(config(DeckRules::Constructed), empty.clone()).is_err());
    assert!(Game::new_with_outside(config(DeckRules::Freeform), empty.clone()).is_ok());
    let mut short = config(DeckRules::Freeform);
    short.decks[0].truncate(39);
    assert!(Game::new_with_outside(short, empty).is_err());
}

#[test]
fn outside_controller_of_player_cannot_see_or_probe_hidden_pool() {
    for player in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let mut g = outside_scenario(player, vec![shock, bear]);
        cast_extrapolate(&mut g, player);
        g.schedule_player_control(player, player.other());
        g.begin_player_control_turn();
        assert_eq!(g.actor(), Some(player.other()));
        let controlled = g.observe(player.other());
        assert!(controlled.outside_choice.as_ref().unwrap().pool.is_none());
        assert!(controlled.owned_outside.is_none());
        assert!(!controlled.actions.iter().any(|a| matches!(
            a,
            Action::ChooseOutsideCard { .. } | Action::ChooseOutsideBasic { .. }
        )));
        assert_eq!(
            g.observe(player).owned_outside.unwrap()[player.index()],
            Some(vec![shock, bear])
        );
        let before = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::DeclineOutsideCards);
            assert_eq!(
                g.state.outside_game.as_ref().unwrap().pools[player.index()],
                vec![shock, bear]
            );
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn outside_revealed_card_stays_known_when_lower_pool_index_is_removed() {
    for player in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let forest = Card::Forest;
        let mut g = outside_scenario(player, vec![shock, bear, forest]);
        cast_extrapolate(&mut g, player);
        act(&mut g, Action::ChooseOutsideCard { index: 1 });
        act(&mut g, Action::ChooseOutsideCard { index: 2 });
        act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
        settle(&mut g);
        for viewer in [player, player.other()] {
            assert_eq!(
                g.observe(viewer).known_outside.unwrap()[player.index()],
                vec![forest]
            );
        }
        cast_extrapolate(&mut g, player);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseOutsideCard { index: 0 });
            act(&mut g, Action::ChooseOutsideCard { index: 1 });
            for viewer in [player, player.other()] {
                let view = g.observe(viewer);
                assert_eq!(view.outside_choice.unwrap().revealed, Some([shock, forest]));
                assert!(view.known_outside.unwrap()[player.index()].contains(&forest));
            }
            act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
            assert_eq!(
                g.state.outside_game.as_ref().unwrap().pools[player.index()],
                vec![forest]
            );
            for viewer in [player, player.other()] {
                assert_eq!(
                    g.observe(viewer).known_outside.unwrap()[player.index()],
                    vec![forest]
                );
            }
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
        }
    }
}

#[test]
fn outside_real_player_control_keeps_pool_private_even_with_known_cards() {
    // Construct a Cosmic Cube schedules the same control effect as Mindslaver;
    // Mindslaver itself is absent from this catalogue.
    for controller in [Player::P0, Player::P1] {
        let owner = controller.other();
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let forest = Card::Forest;
        let mut g = outside_scenario(owner, vec![shock, bear, forest]);
        cast_extrapolate(&mut g, owner);
        act(&mut g, Action::ChooseOutsideCard { index: 0 });
        act(&mut g, Action::ChooseOutsideCard { index: 1 });
        act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
        settle(&mut g);
        assert_eq!(
            g.observe(controller).known_outside.unwrap()[owner.index()],
            vec![bear]
        );

        g.state.active = controller;
        g.state.priority = controller;
        g.state.pending = Pending::Priority;
        g.state.step = Step::Main1;
        let plan = put(
            &mut g,
            controller,
            "Construct a Cosmic Cube",
            Zone::Battlefield,
        );
        // Seed six counters; the seventh must be put by an actual effect so
        // that the threshold event is emitted. add_counter alone is a raw
        // state helper and deliberately does not dispatch that event.
        g.add_counter(plan, CounterKind::Plan, 6);
        let divination = put(&mut g, controller, "Divination", Zone::Hand);
        // The starting pool empties on the way to the controlled player's
        // next turn. Give that player actual mana sources for the later cast.
        let swamps = [
            put(&mut g, owner, "Swamp", Zone::Battlefield),
            put(&mut g, owner, "Swamp", Zone::Battlefield),
        ];
        let before_control = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: divination });
            pay(&mut g);
            for _ in 0..64 {
                g.prepare_triggers();
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(owner),
                }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(owner),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            settle(&mut g);
            assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
            // Progress the actual game through combat, cleanup and the next
            // turn instead of editing the active step or calling cleanup.
            for _ in 0..160 {
                if g.state.active == owner
                    && g.state.step == Step::Main1
                    && g.state.pending == Pending::Priority
                {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                    )
                });
            }
            assert_eq!(g.state.active, owner);
            assert_eq!(g.state.step, Step::Main1);
            assert_eq!(g.decision_controller(owner), controller);
            for &swamp in &swamps {
                let mana = Action::TapMana { card: swamp };
                assert!(g.legal_actions().contains(&mana));
                act(&mut g, mana);
            }
            cast_extrapolate(&mut g, owner);
            assert_eq!(g.actor(), Some(controller));
            let visible = g.observe(controller);
            assert!(visible.owned_outside.is_none());
            assert!(visible.outside_choice.as_ref().unwrap().pool.is_none());
            assert_eq!(visible.known_outside.unwrap()[owner.index()], vec![bear]);
            assert!(visible.actions.contains(&Action::DeclineOutsideCards));
            assert!(!visible.actions.iter().any(|a| matches!(
                a,
                Action::ChooseOutsideCard { .. } | Action::ChooseOutsideBasic { .. }
            )));
            let saved = g.snapshot();
            for _ in 0..2 {
                act(&mut g, Action::DeclineOutsideCards);
                assert_eq!(
                    g.state.outside_game.as_ref().unwrap().pools[owner.index()],
                    vec![bear, forest]
                );
                g.restore(&saved).unwrap();
                assert_eq!(g.snapshot(), saved);
            }
            g.restore(&before_control).unwrap();
            assert_eq!(g.snapshot(), before_control);
        }
    }
}

#[test]
fn outside_card_is_not_drawn_and_remains_its_owners_usable_physical_card() {
    for player in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let mut g = outside_scenario(player, vec![shock, Card::Forest]);
        put(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let observer = put(&mut g, player, "Construct a Cosmic Cube", Zone::Battlefield);
        let return_observer = put(&mut g, player, "Justice, Vance Astrovik", Zone::Battlefield);
        g.state.players[player.index()].cards_drawn_this_turn = 1;
        cast_extrapolate(&mut g, player);
        act(&mut g, Action::ChooseOutsideCard { index: 0 });
        act(&mut g, Action::ChooseOutsideCard { index: 1 });
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.source.is_some_and(|source| source.id == observer))
            );
            assert_eq!(g.counter_count(observer, CounterKind::Plan), 0);
            assert_eq!(g.counter_count(return_observer, CounterKind::PlusOne), 0);
            let (id, card) = g
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, o)| o.card == shock && o.owner == player && o.zone == Zone::Hand)
                .unwrap();
            assert!(!card.is_token() && !card.is_spell_copy());
            let id = id as CardId;
            let reference = g.object_ref(id);
            assert!(g.observe(player.other()).known_hand.contains(&reference));
            let life = g.state.players[player.other().index()].life;
            act(&mut g, Action::AnnounceCast { card: id });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(player.other()),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert!(!g.observe(player.other()).known_hand.contains(&reference));
            settle(&mut g);
            assert_eq!(g.state.players[player.other().index()].life, life - 2);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
        }
    }
}

#[test]
fn outside_spell_copy_uses_controllers_pool_and_original_sees_remaining_cards() {
    for player in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let mut g = outside_scenario(player, vec![shock, bear, Card::Forest]);
        let original = put(&mut g, player, "Shock", Zone::Hand);
        g.state.objects[original as usize].card =
            Card::from_name("Extrapolate the Impossible").unwrap();
        act(&mut g, Action::AnnounceCast { card: original });
        pay(&mut g);
        let sparks = put(&mut g, player, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(original),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::OutsideGameChoice { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            assert!(
                matches!(g.state.pending, Pending::OutsideGameChoice { player: actor } if actor == player)
            );
            act(&mut g, Action::ChooseOutsideCard { index: 0 });
            act(&mut g, Action::ChooseOutsideCard { index: 1 });
            act(&mut g, Action::ChooseOutsideRevealed { index: 0 });
            for _ in 0..64 {
                if matches!(g.state.pending, Pending::OutsideGameChoice { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert_eq!(
                g.state.outside_game.as_ref().unwrap().pools[player.index()],
                vec![bear, Card::Forest]
            );
            act(&mut g, Action::ChooseOutsideCard { index: 0 });
            act(&mut g, Action::ChooseOutsideCard { index: 1 });
            act(&mut g, Action::ChooseOutsideRevealed { index: 1 });
            settle(&mut g);
            assert_eq!(
                g.state.outside_game.as_ref().unwrap().pools[player.index()],
                vec![bear]
            );
            assert!(
                g.state
                    .objects
                    .iter()
                    .any(|o| o.card == shock && o.owner == player && o.zone == Zone::Hand)
            );
            assert!(
                g.state
                    .objects
                    .iter()
                    .any(|o| o.card == Card::Forest && o.owner == player && o.zone == Zone::Hand)
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn outside_hand_admission_replays_each_opponent_choice_and_preserves_public_knowledge() {
    for owner in [Player::P0, Player::P1] {
        for chosen in [0u8, 1] {
            let shock = Card::from_name("Shock").unwrap();
            let bear = Card::from_name("Bear Cub").unwrap();
            let cards = [shock, bear];
            let mut g = outside_scenario(owner, cards.to_vec());
            cast_extrapolate(&mut g, owner);
            act(&mut g, Action::ChooseOutsideCard { index: 0 });
            act(&mut g, Action::ChooseOutsideCard { index: 1 });
            let before = g.snapshot();
            let action = Action::ChooseOutsideRevealed { index: chosen };
            let actor = owner.other();
            let rev = g.revision();
            let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            assert!(raw.apply(owner, action.clone()).is_err());
            assert_eq!(raw.snapshot(), before);
            raw.apply(actor, action.clone()).unwrap();
            indexed.apply_index(actor, rev, index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            let after = raw.snapshot();
            let selected = cards[usize::from(chosen)];
            let unselected = cards[1 - usize::from(chosen)];
            assert_eq!(
                raw.state.outside_game.as_ref().unwrap().pools[owner.index()],
                vec![unselected]
            );
            let (id, object) = raw
                .state
                .objects
                .iter()
                .enumerate()
                .find(|(_, object)| {
                    object.card == selected && object.owner == owner && object.zone == Zone::Hand
                })
                .unwrap();
            assert!(!object.is_token() && !object.is_spell_copy());
            assert_eq!(object.controller, owner);
            let reference = raw.object_ref(id as CardId);
            for viewer in [Player::P0, Player::P1] {
                let view = raw.observe(viewer);
                assert!(view.known_hand.contains(&reference));
                assert_eq!(view.known_outside.unwrap()[owner.index()], vec![unselected]);
                assert!(view.outside_choice.is_none());
            }
            raw.restore(&before).unwrap();
            for viewer in [Player::P0, Player::P1] {
                assert_eq!(
                    raw.observe(viewer).outside_choice.unwrap().revealed,
                    Some(cards)
                );
            }
            act(&mut raw, action);
            assert_eq!(raw.snapshot(), after);
        }
    }
}

#[test]
fn planned_outside_entry_validates_exact_slot_and_replays() {
    for owner in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let mut g = outside_scenario(owner, vec![shock, bear, shock, Card::Forest]);
        cast_extrapolate(&mut g, owner);
        act(&mut g, Action::ChooseOutsideCard { index: 2 });
        act(&mut g, Action::ChooseOutsideCard { index: 1 });
        assert!(g.plan_outside_hand(0));
        // The real provider discovery path leaves an unaffected admission
        // unchanged and its owned continuation survives snapshot replay.
        let before_discovery = g.snapshot();
        assert!(g.prepare_outside_entry_providers());
        assert_eq!(g.snapshot(), before_discovery);
        let planned = g.snapshot();
        assert!(
            g.observe(owner.other())
                .outside_choice
                .as_ref()
                .unwrap()
                .revealed
                .is_some()
        );
        for _ in 0..2 {
            assert!(g.commit_outside_hand());
            // The internal commit only installs the card; the real action
            // resumes the resolving program before exposing an observation.
            g.run_program();
            let outside = g.state.outside_game.as_ref().unwrap();
            assert_eq!(
                outside.pools[owner.index()],
                vec![shock, bear, Card::Forest]
            );
            assert_eq!(outside.pool_versions[owner.index()], 1);
            assert!(outside.pending_entry.is_none());
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Hand && o.owner == owner && o.card == shock)
                    .count(),
                1
            );
            assert_eq!(
                g.observe(owner.other()).known_outside.unwrap()[owner.index()],
                vec![bear]
            );
            g.restore(&planned).unwrap();
        }
        // An identical pair in a new choice has a distinct owned identity.
        assert!(g.start_outside_choice(owner));
        g.choose_outside_card(outside_choices::Selection::Pool(2));
        g.choose_outside_card(outside_choices::Selection::Pool(1));
        let identical = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert_eq!(g.snapshot(), identical);
        assert!(g.plan_outside_hand(0));
        assert!(g.commit_outside_hand());
        assert_eq!(
            g.state.outside_game.as_ref().unwrap().pools[owner.index()],
            vec![shock, bear, Card::Forest]
        );
        g.restore(&planned).unwrap();
        // Declining clears the staged plan; a restarted choice cannot reuse it.
        g.decline_outside_cards();
        assert!(
            g.state
                .outside_game
                .as_ref()
                .unwrap()
                .pending_entry
                .is_none()
        );
        assert!(g.start_outside_choice(owner));
        g.choose_outside_card(outside_choices::Selection::Pool(2));
        g.choose_outside_card(outside_choices::Selection::Pool(1));
        let restarted = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert_eq!(g.snapshot(), restarted);
        g.restore(&planned).unwrap();
        // Overflow is signalled before replacing an existing choice or plan.
        g.state.outside_game.as_mut().unwrap().next_choice_id = u64::MAX;
        let full = g.snapshot();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || g.start_outside_choice(owner)
            ))
            .is_err()
        );
        assert_eq!(g.snapshot(), full);
        g.restore(&planned).unwrap();
        // Replacing the revealed pair cannot reuse an overlapping selected slot.
        assert!(g.start_outside_choice(owner));
        g.choose_outside_card(outside_choices::Selection::Pool(2));
        g.choose_outside_card(outside_choices::Selection::Pool(3));
        let overlapping = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert_eq!(g.snapshot(), overlapping);
        assert!(g.plan_outside_hand(0));
        assert!(g.commit_outside_hand());
        g.restore(&planned).unwrap();
        // Even if the pair IDs match, changed unselected revealed facts reject.
        g.state.outside_game.as_mut().unwrap().pools[owner.index()][1] = Card::Forest;
        let changed_other = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert_eq!(g.snapshot(), changed_other);
        g.restore(&planned).unwrap();
        // A changed pool generation invalidates the original slot even if an
        // equal-named duplicate occupies the same numeric index.
        g.state.outside_game.as_mut().unwrap().pool_versions[owner.index()] += 1;
        let stale = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert_eq!(g.snapshot(), stale);
        g.restore(&planned).unwrap();
        // Changing the slot contents is rejected before removing anything.
        g.state.outside_game.as_mut().unwrap().pools[owner.index()][2] = Card::Forest;
        let changed = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert_eq!(g.snapshot(), changed);
    }
}

#[test]
fn planned_virtual_basic_entry_does_not_consume_outside_pool() {
    for owner in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let mut g = outside_scenario(owner, vec![shock]);
        g.state.outside_game.as_mut().unwrap().mode = OutsideGameMode::LimitedSideboard;
        cast_extrapolate(&mut g, owner);
        act(&mut g, Action::ChooseOutsideCard { index: 0 });
        act(&mut g, Action::ChooseOutsideBasic { card: Card::Forest });
        assert!(g.plan_outside_hand(1));
        let planned = g.snapshot();
        for _ in 0..2 {
            assert!(g.commit_outside_hand());
            let state = g.state.outside_game.as_ref().unwrap();
            assert_eq!(state.pools[owner.index()], vec![shock]);
            assert_eq!(state.pool_versions[owner.index()], 0);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == owner && o.zone == Zone::Hand && o.card == Card::Forest)
                    .count(),
                1
            );
            g.restore(&planned).unwrap();
        }
    }
}

#[test]
fn synthetic_outside_entry_replacement_choices_are_owned_and_transactional() {
    use outside_choices::{OutsideEntryDestination, OutsideEntryReplacement};
    for owner in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let mut g = outside_scenario(owner, vec![shock, bear]);
        cast_extrapolate(&mut g, owner);
        act(&mut g, Action::ChooseOutsideCard { index: 0 });
        act(&mut g, Action::ChooseOutsideCard { index: 1 });
        assert!(g.plan_outside_hand(0));
        let choices = vec![
            OutsideEntryReplacement {
                id: 11,
                destination: OutsideEntryDestination::Prevent,
            },
            OutsideEntryReplacement {
                id: 12,
                destination: OutsideEntryDestination::Exile,
            },
        ];
        let duplicate = vec![choices[0], choices[0]];
        let before_offer = g.snapshot();
        assert!(!g.offer_outside_entry_replacements(duplicate));
        assert_eq!(g.snapshot(), before_offer);
        assert!(g.offer_outside_entry_replacements(choices));
        let waiting = g.snapshot();
        assert!(!g.commit_outside_hand());
        assert!(!g.choose_outside_entry_replacement(99));
        assert_eq!(g.snapshot(), waiting);
        for (effect, prevented) in [(11, true), (12, false)] {
            for _ in 0..2 {
                assert!(g.choose_outside_entry_replacement(effect));
                let after_choice = g.snapshot();
                assert!(
                    !g.offer_outside_entry_replacements(vec![OutsideEntryReplacement {
                        id: effect,
                        destination: OutsideEntryDestination::Hand,
                    }])
                );
                assert_eq!(g.snapshot(), after_choice);
                assert!(g.commit_outside_hand());
                let state = g.state.outside_game.as_ref().unwrap();
                assert!(state.pending_entry.is_none());
                assert_eq!(
                    state.pools[owner.index()],
                    if prevented {
                        vec![shock, bear]
                    } else {
                        vec![bear]
                    }
                );
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.owner == owner && o.card == shock && o.zone == Zone::Exile)
                        .count(),
                    usize::from(!prevented)
                );
                assert!(
                    !g.state
                        .objects
                        .iter()
                        .any(|o| o.owner == owner && o.card == shock && o.zone == Zone::Hand)
                );
                g.restore(&waiting).unwrap();
                assert_eq!(g.snapshot(), waiting);
            }
        }
    }
}

#[test]
fn outside_admission_uses_real_provider_choice_and_replays() {
    let _fixture = crate::cards::test_counter_providers::activate_outside();
    for owner in [Player::P0, Player::P1] {
        let shock = Card::from_name("Shock").unwrap();
        let bear = Card::from_name("Bear Cub").unwrap();
        let mut game = outside_scenario(owner, vec![shock, bear]);
        let provider = put(&mut game, owner, "Healer's Hawk", Zone::Battlefield);
        let ennis = put(
            &mut game,
            owner,
            "Ennis, Debate Moderator",
            Zone::Battlefield,
        );
        let source = game.object_ref(provider);
        cast_extrapolate(&mut game, owner);
        act(&mut game, Action::ChooseOutsideCard { index: 0 });
        act(&mut game, Action::ChooseOutsideCard { index: 1 });
        act(&mut game, Action::ChooseOutsideRevealed { index: 0 });
        assert!(
            matches!(game.state.pending, Pending::OutsideEntryReplacement { player } if player == owner)
        );
        let before = game.snapshot();
        assert!(game.observe(owner).outside_entry_replacement.is_some());
        assert!(
            game.observe(owner.other())
                .outside_entry_replacement
                .is_none()
        );
        let prevent = Action::ChooseOutsideEntryReplacement { source, ability: 0 };
        let exile = Action::ChooseOutsideEntryReplacement { source, ability: 1 };
        assert!(game.legal_actions().contains(&prevent));
        assert!(game.legal_actions().contains(&exile));
        // A player controlling the affected player makes the replacement
        // decision; provider information remains visible to both of them.
        game.state.active = owner;
        game.schedule_player_control(owner, owner.other());
        game.begin_player_control_turn();
        assert!(game.observe(owner).outside_entry_replacement.is_some());
        assert!(game.observe(owner).actions.is_empty());
        assert!(
            game.observe(owner.other())
                .outside_entry_replacement
                .is_some()
        );
        assert!(game.observe(owner.other()).actions.contains(&prevent));
        game.restore(&before).unwrap();
        assert!(game.apply(owner.other(), prevent.clone()).is_err());
        assert!(
            game.apply(
                owner,
                Action::ChooseOutsideEntryReplacement { source, ability: 2 }
            )
            .is_err()
        );
        assert_eq!(game.snapshot(), before);
        let before_objects = game.state.objects.len();
        act(&mut game, prevent.clone());
        assert_eq!(game.state.objects.len(), before_objects);
        assert!(!game.state.cards_exiled_this_turn);
        assert_eq!(
            game.state.outside_game.as_ref().unwrap().pools[owner.index()],
            vec![shock, bear]
        );
        let prevented = game.snapshot();
        for _ in 0..2 {
            game.begin_step(Step::End);
            game.prepare_triggers();
            settle(&mut game);
            assert_eq!(game.counter_count(ennis, CounterKind::PlusOne), 0);
            game.restore(&prevented).unwrap();
        }
        game.restore(&before).unwrap();
        game.state.cards_exiled_this_turn = true;
        act(&mut game, prevent);
        assert!(game.state.cards_exiled_this_turn);
        game.restore(&before).unwrap();
        act(&mut game, exile.clone());
        assert_eq!(
            game.state.outside_game.as_ref().unwrap().pools[owner.index()],
            vec![bear]
        );
        assert!(
            game.state
                .objects
                .iter()
                .any(|o| o.owner == owner && o.card == shock && o.zone == Zone::Exile)
        );
        assert!(game.state.cards_exiled_this_turn);
        let after = game.snapshot();
        game.restore(&before).unwrap();
        act(&mut game, exile);
        assert_eq!(game.snapshot(), after);
        // Ennis observes the actual admission at its controller's end step.
        for _ in 0..2 {
            game.begin_step(Step::End);
            game.prepare_triggers();
            settle(&mut game);
            assert_eq!(game.counter_count(ennis, CounterKind::PlusOne), 1);
            game.restore(&after).unwrap();
        }
    }
}
