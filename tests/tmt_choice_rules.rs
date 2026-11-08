//! Paid TMNT choices, hidden information, and replay across the merged engine.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn advance(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..72 {
        if done(g) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    panic!("choice did not arrive, pending {:?}", g.state.pending);
}

fn spell(g: &mut Game, player: Player, name: &str, target: Option<Target>) -> CardId {
    let source = put(g, player, name, Zone::Hand);
    cast(g, source, target);
    source
}

fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let mut owned = {
        let parent = g.clone();
        parent.clone()
    };
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    owned.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    assert_eq!(g.snapshot(), owned.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_vec(&g.observe(viewer)).unwrap(),
            serde_json::to_vec(&owned.observe(viewer)).unwrap()
        );
    }
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), owned.snapshot());
}

fn outside_scenario(player: Player, own: &[Card], opponent: &[Card]) -> Game {
    let mut pools = [Vec::new(), Vec::new()];
    pools[player.index()] = own.to_vec();
    pools[player.other().index()] = opponent.to_vec();
    let configured = Game::new_with_outside(
        Config {
            decks: [vec![Card::Forest; 7], vec![Card::Island; 7]],
            seed: 20261023,
            starting_player: player,
            deck_rules: DeckRules::Freeform,
        },
        OutsideGameConfig {
            mode: OutsideGameMode::CasualCollection,
            pools,
        },
    )
    .unwrap();
    let mut g = scenario(player);
    g.state.outside_game = configured.state.outside_game;
    g
}

fn hand_card(g: &Game, player: Player, definition: Card) -> ObjectRef {
    g.observe(player)
        .objects
        .iter()
        .find(|o| o.owner == player && o.zone == Zone::Hand && o.card == definition)
        .unwrap()
        .reference
}

#[test]
fn tmt_manhole_missile_bottom_payment_is_opaque_and_draws_the_previous_top() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let choice = put(&mut g, p, "Shock", Zone::Hand);
        let another = put(&mut g, p, "Opt", Zone::Hand);
        let top = put(&mut g, p, "Island", Zone::Library);
        g.remember_public_hand(g.object_ref(choice));
        g.remember_public_hand(g.object_ref(another));
        spell(&mut g, p, "Manhole Missile", Some(Target::Card(victim)));
        advance(&mut g, |g| g.observe(p).effect_choice.is_some());
        assert!(g.observe(p.other()).effect_choice.is_none());
        let object = g.object_ref(choice);
        checked(&mut g, Action::BottomHandCard { object });
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[top as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].library[0], choice);
        let peer = g.observe(p.other());
        assert!(peer.last_target_library_move.is_none());
        assert!(!peer.known_hand.contains(&g.object_ref(another)));
        assert!(
            !peer
                .objects
                .iter()
                .any(|o| o.id == choice || o.id == another)
        );
    }
}

#[test]
fn tmt_michelangelo_technique_enforces_the_private_top_eight_total_budget() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let large = put(&mut g, p, "Serra Angel", Zone::Library);
        let cheap = put(&mut g, p, "Llanowar Elves", Zone::Library);
        let exceeds = put(&mut g, p, "Bear Cub", Zone::Library);
        let mut bottom = vec![exceeds];
        for _ in 0..5 {
            bottom.push(put(&mut g, p, "Forest", Zone::Library));
        }
        spell(&mut g, p, "Michelangelo's Technique", None);
        advance(&mut g, |g| g.observe(p).library_selection.is_some());
        assert!(g.observe(p.other()).library_selection.is_none());
        assert_eq!(g.observe(p).library_selection.unwrap().remaining.len(), 8);
        act(&mut g, Action::SelectLibrary { card: large });
        reject(&mut g, Action::SelectLibrary { card: exceeds });
        act(&mut g, Action::SelectLibrary { card: cheap });
        assert!(!g.on_battlefield(large) && !g.on_battlefield(cheap));
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert!(g.on_battlefield(large) && g.on_battlefield(cheap));
        let mut placed = g.state.players[p.index()].library[..6].to_vec();
        placed.sort_unstable();
        bottom.sort_unstable();
        assert_eq!(placed, bottom);
    }
}

#[test]
fn tmt_kitsune_technique_mills_the_targets_odd_current_library() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let top = put(&mut g, p.other(), "Island", Zone::Library);
        spell(
            &mut g,
            p,
            "Kitsune's Technique",
            Some(Target::Player(p.other())),
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].library.len(), 4);
        assert_eq!(g.state.players[p.index()].library.len(), 8);
        assert_eq!(g.state.objects[top as usize].zone, Zone::Graveyard);
    }
}

#[test]
fn tmt_michelangelo_improviser_commits_one_creature_and_one_land_together() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = spell(&mut g, p, "Michelangelo, Improviser", None);
        settle(&mut g);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        let extra = put(&mut g, p, "Serra Angel", Zone::Hand);
        let land = put(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[source as usize].summoning_sick = false;
        advance(&mut g, |g| g.state.pending == Pending::Attackers);
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        advance(&mut g, |g| g.observe(p).effect_choice.is_some());
        assert!(g.observe(p.other()).effect_choice.is_none());
        let creature_ref = g.object_ref(creature);
        act(
            &mut g,
            Action::PutHandPermanent {
                object: creature_ref,
            },
        );
        let invalid = g.object_ref(extra);
        reject(&mut g, Action::PutHandPermanent { object: invalid });
        assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
        let land_ref = g.object_ref(land);
        checked(&mut g, Action::PutHandPermanent { object: land_ref });
        assert!(g.on_battlefield(creature) && g.on_battlefield(land));
        assert!(!g.state.objects[creature as usize].tapped);
        assert!(!g.state.attackers.iter().any(|a| a.card == creature));
    }
}

#[test]
fn tmt_wheels_wait_for_both_public_choices_before_moving_cards() {
    for p in [Player::P0, Player::P1] {
        for shuffle in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Opt", Zone::Hand);
            let enemy = put(&mut g, p.other(), "Shock", Zone::Hand);
            let grave = put(&mut g, p, "Island", Zone::Graveyard);
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let name = if shuffle {
                "Turtles in Time"
            } else {
                "Raphael's Technique"
            };
            let source = spell(&mut g, p, name, None);
            advance(&mut g, |g| g.observe(p).optional_wheel_choices.is_some());
            assert_eq!(g.actor(), Some(p));
            act(&mut g, Action::ChooseOptional { accept: true });
            assert_eq!(g.actor(), Some(p.other()));
            let mut expected = [None; 2];
            expected[p.index()] = Some(true);
            assert_eq!(g.observe(p).optional_wheel_choices, Some(expected));
            assert_eq!(g.observe(p.other()).optional_wheel_choices, Some(expected));
            assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[enemy as usize].zone, Zone::Hand);
            checked(&mut g, Action::ChooseOptional { accept: !shuffle });
            settle(&mut g);
            assert_eq!(g.observe(p).players[p.index()].hand_count, 7);
            if shuffle {
                assert_eq!(g.state.objects[source as usize].zone, Zone::Exile);
                assert_eq!(g.state.objects[creature as usize].zone, Zone::Hand);
                assert_eq!(g.state.objects[enemy as usize].zone, Zone::Hand);
                assert_ne!(g.state.objects[grave as usize].zone, Zone::Graveyard);
            } else {
                assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
                assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
                assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
                assert_eq!(g.observe(p).players[p.other().index()].hand_count, 7);
            }
        }
    }
}

#[test]
fn tmt_north_wind_single_outside_wish_keeps_the_new_hand_card_private() {
    for p in [Player::P0, Player::P1] {
        for admit in [false, true] {
            let own = if admit {
                vec![Card::Forest]
            } else {
                Vec::new()
            };
            let mut g = outside_scenario(p, &own, &[Card::Island]);
            spell(&mut g, p, "North Wind Avatar", None);
            advance(&mut g, |g| g.observe(p).outside_choice.is_some());
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.observe(p).outside_choice.unwrap().pool, Some(own));
            assert!(g.observe(p.other()).outside_choice.unwrap().pool.is_none());
            if admit {
                reject(&mut g, Action::ChooseOutsideCard { index: 1 });
                checked(&mut g, Action::ChooseOutsideCard { index: 0 });
                let admitted = hand_card(&g, p, Card::Forest);
                let peer = g.observe(p.other());
                assert!(!peer.known_hand.contains(&admitted));
                assert!(!peer.objects.iter().any(|o| o.reference == admitted));
                assert!(g.observe(p).owned_outside.is_none());
            } else {
                reject(&mut g, Action::ChooseOutsideCard { index: 0 });
                act(&mut g, Action::DeclineOutsideCards);
            }
        }
    }
}

#[test]
fn tmt_turtles_forever_reveals_only_found_cards_then_moves_exact_owned_choices() {
    for p in [Player::P0, Player::P1] {
        let krenko = Card::from_name("Krenko, Mob Boss").unwrap();
        let muldrotha = Card::from_name("Muldrotha, the Gravetide").unwrap();
        let kellan = Card::from_name("Kellan, Planar Trailblazer").unwrap();
        let mut g = outside_scenario(p, &[krenko, muldrotha, kellan], &[muldrotha]);
        let library = put(&mut g, p, krenko.name(), Zone::Library);
        spell(&mut g, p, "Turtles Forever", None);
        advance(&mut g, |g| g.observe(p).legendary_split_search.is_some());
        act(
            &mut g,
            Action::ChooseLegendarySearchZones {
                library: true,
                outside: true,
            },
        );
        act(&mut g, Action::ChooseSearch { card: library });
        reject(&mut g, Action::ChooseOutsideCard { index: 0 });
        act(&mut g, Action::ChooseOutsideCard { index: 1 });
        act(&mut g, Action::ChooseOutsideCard { index: 2 });
        assert!(g.observe(p.other()).legendary_split_search.is_none());
        checked(&mut g, Action::FinishSearch);
        assert_eq!(g.actor(), Some(p.other()));
        let view = g.observe(p.other()).legendary_split_search.unwrap();
        assert!(view.revealed && view.library.is_empty() && view.outside_candidates.is_empty());
        assert_eq!(view.found, [krenko, muldrotha, kellan]);
        assert_eq!(view.required_for_hand, 2);
        reject(&mut g, Action::FinishSearch);
        act(&mut g, Action::ChooseLegendaryFound { index: 0 });
        reject(&mut g, Action::ChooseLegendaryFound { index: 0 });
        act(&mut g, Action::ChooseLegendaryFound { index: 1 });
        checked(&mut g, Action::FinishSearch);
        assert_eq!(g.state.objects[library as usize].zone, Zone::Hand);
        let admitted = hand_card(&g, p, muldrotha);
        assert!(g.observe(p.other()).known_hand.contains(&admitted));
        assert!(
            g.state.players[p.index()]
                .library
                .iter()
                .any(|&id| g.state.objects[id as usize].card == kellan)
        );
        assert_eq!(
            g.observe(p).owned_outside.unwrap()[p.index()],
            Some(vec![krenko])
        );
        assert_eq!(
            g.observe(p.other()).owned_outside.unwrap()[p.other().index()],
            Some(vec![muldrotha])
        );
    }
}

#[test]
fn tmt_mondo_gecko_keeps_earlier_hexproof_colors_after_the_color_changes_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = spell(&mut g, p, "Mondo Gecko", None);
        settle(&mut g);
        for color in [ChoiceColor::Red, ChoiceColor::Blue] {
            let card = put(&mut g, p, "Forest", Zone::Hand);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(card);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            advance(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionColor { .. })
            });
            checked(&mut g, Action::ChooseResolutionColor { color });
            settle(&mut g);
            assert_eq!(g.object_view(source).colors, 1 << color.index());
        }
        let target = Target::Card(source);
        for (name, blocked) in [("Shock", true), ("Unsummon", true), ("Murder", false)] {
            let id = put(&mut g, p.other(), name, Zone::Hand);
            let card = g.state.objects[id as usize].card;
            assert_eq!(
                g.cast_target_matches(
                    TargetRule::Creature,
                    p.other(),
                    g.object_ref(id),
                    card,
                    target
                ),
                !blocked
            );
            assert!(g.cast_target_matches(TargetRule::Creature, p, g.object_ref(id), card, target));
        }
    }
}

#[test]
fn tmt_wingnut_alliance_chooses_a_real_keyword_for_the_exact_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = spell(&mut g, p, "Wingnut, Bat on the Belfry", None);
        settle(&mut g);
        spell(&mut g, p, "Bear Cub", None);
        advance(&mut g, |g| g.observe(p).keyword_choice.is_some());
        let choice = g.observe(p).keyword_choice.unwrap();
        let index = choice
            .choices
            .iter()
            .position(|&k| k == Keywords::MENACE)
            .unwrap();
        checked(
            &mut g,
            Action::ChooseKeyword {
                index: u16::try_from(index).unwrap(),
            },
        );
        assert!(g.keywords(source).contains(Keywords::MENACE));
    }
}
