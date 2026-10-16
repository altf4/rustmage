use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn settle(g: &mut Game, target: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.state.stack.is_empty() && g.state.pending == Pending::Priority {
            return;
        }
        let p = g.actor().unwrap();
        if g.observe(p).library_choice.is_some() {
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(g.observe(p.other()).actions.is_empty());
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseSearch { .. }))
            .cloned()
            .or_else(|| {
                actions.into_iter().find(|a| match a {
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    } => target.is_none_or(|t| t == *id),
                    Action::ChooseTarget {
                        target: Target::Player(who),
                    } => *who == p.other(),
                    Action::ChooseSearch { .. }
                    | Action::DeclineEffectPayment
                    | Action::FinishSearch
                    | Action::PutTrigger { .. }
                    | Action::Pass
                    | Action::ArrangeLibrary { .. }
                    | Action::Sacrifice { .. }
                    | Action::FinishTargets => true,
                    _ => false,
                })
            })
            .expect("fixture resolution choice");
        replay(g, a);
    }
    panic!("not settled");
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability:a,target:None,..}if *card==id&&*a==ability)).expect("fixture activation");
    replay(g, a);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<ObjectView> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.owner == p
                && o.card.is_token()
                && o.card.has_subtype(subtype)
        })
        .map(|(i, _)| g.object_view(i as CardId))
        .collect()
}
fn announce(g: &mut Game, id: CardId, ability: u16) {
    replay(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn choose_cost(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    replay(g, Action::ChooseAdditionalCost { object });
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn hidden(g: &Game, p: Player) {
    for o in g
        .observe(p.other())
        .objects
        .iter()
        .filter(|o| o.owner == p && o.zone == Zone::Hand)
    {
        assert!(
            g.state.known_hand.contains(&o.reference),
            "private draw leaked"
        );
    }
}

#[test]
fn complete_permanent_bodies_cast_with_exact_stats_keywords_entries_and_legend_flags() {
    for p in [Player::P0, Player::P1] {
        for (name, pt, keyword, legend) in [
            ("Biomechan Engineer", (2, 2), Keywords::NONE, false),
            ("Lunatic Pandora", (0, 0), Keywords::NONE, true),
            ("Popular Egotist", (3, 2), Keywords::NONE, false),
            (
                "Reckless Lackey",
                (1, 2),
                Keywords::FIRST_STRIKE.union(Keywords::HASTE),
                false,
            ),
            ("Redrock Sentinel", (2, 4), Keywords::DEFENDER, false),
            ("The Sackville-Bagginses", (2, 2), Keywords::NONE, true),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None);
            settle(&mut g, None);
            let v = g.object_view(id);
            assert_eq!(v.zone, Zone::Battlefield);
            assert_eq!((v.power, v.toughness), pt);
            assert!(v.keywords.contains(keyword));
            assert_eq!(v.card.is_legendary(), legend);
            if name == "Biomechan Engineer" {
                assert_eq!(tokens(&g, p, "Lander").len(), 1);
            }
            if name == "Redrock Sentinel" {
                assert!(v.card.is_artifact());
            }
            if name == "The Sackville-Bagginses" {
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
                assert!(tokens(&g, p, "Treasure").is_empty());
            }
        }
    }
}
#[test]
fn biomechan_and_lackey_draw_then_create_full_tokens_after_payment_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for (name, amount, subtype, pt) in [
            ("Biomechan Engineer", 2, "Robot", (2, 2)),
            ("Reckless Lackey", 1, "Treasure", (0, 0)),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None);
            settle(&mut g, None);
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            activate(&mut g, id, 0);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before);
            assert!(tokens(&g, p, subtype).is_empty());
            assert_eq!(
                g.object_view(id).zone,
                if name == "Reckless Lackey" {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if name == "Biomechan Engineer" {
                g.move_object(id, Zone::Hand);
            }
            settle(&mut g, None);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                before + amount
            );
            hidden(&g, p);
            let t = tokens(&g, p, subtype);
            assert_eq!(t.len(), 1);
            assert_eq!((t[0].power, t[0].toughness), pt);
            assert!(t[0].card.is_artifact());
            assert_eq!(t[0].card.colors(), 0);
            assert!(!t[0].tapped);
            if subtype == "Robot" {
                assert!(t[0].summoning_sick);
                assert!(t[0].card.is_creature());
            } else {
                let mana=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==t[0].id&&production[3]==1)).unwrap();
                let before = g.state.players[p.index()].mana[3];
                replay(&mut g, mana);
                assert_eq!(g.state.players[p.index()].mana[3], before + 1);
                assert_eq!(g.object_view(t[0].id).zone, Zone::Outside);
            }
        }
    }
}
#[test]
fn seize_the_spoils_commits_exact_private_discard_cost_before_draw_and_treasure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = named(&mut g, p, "Seize the Spoils", Zone::Hand);
        let hand = named(&mut g, p, "Bear Cub", Zone::Hand);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        assert!(g.observe(p.other()).additional_cost.is_none());
        for id in [spell, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        choose_cost(&mut g, hand);
        assert_eq!(g.object_view(hand).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        pay(&mut g);
        assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert!(tokens(&g, p, "Treasure").is_empty());
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        settle(&mut g, None);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(tokens(&g, p, "Treasure").len(), 1);
        hidden(&g, p);
    }
}
#[test]
fn redrock_requires_owned_land_and_untapped_nonsick_source_then_pays_before_drawing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Redrock Sentinel", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activation.clone());
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let body = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].summoning_sick = true;
        reject(&mut g, activation.clone());
        g.state.objects[source as usize].summoning_sick = false;
        replay(&mut g, activation);
        for id in [foreign, body, source] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        choose_cost(&mut g, land);
        assert!(!g.object_view(source).tapped);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(tokens(&g, p, "Treasure").is_empty());
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(tokens(&g, p, "Treasure").len(), 1);
        hidden(&g, p);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn pandora_surveil_is_owned_private_and_destroy_sacrifices_before_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Lunatic Pandora", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g, None);
        activate(&mut g, source, 0);
        pass_pair(&mut g);
        let choice = g.observe(p).library_choice.unwrap();
        assert_eq!(choice.remaining.len(), 1);
        assert!(g.observe(p.other()).library_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        let card = choice.remaining[0].id;
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Graveyard,
            },
        );
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
        assert!(g.object_view(source).tapped);
        g.begin_turn();
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        announce(&mut g, source, 1);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        settle(&mut g, None);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}
#[test]
fn popular_egotist_cost_triggers_drain_above_indestructible_and_tap_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Popular Egotist", Zone::Battlefield);
        let resource = named(&mut g, p, "Goblin Oriflamme", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = named(&mut g, p, "Nutrient Block", Zone::Battlefield);
        announce(&mut g, source, 0);
        for id in [source, foreign, artifact] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        choose_cost(&mut g, resource);
        pay(&mut g);
        assert_eq!(g.object_view(resource).zone, Zone::Graveyard);
        assert!(!g.object_view(source).tapped);
        assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        assert_eq!(g.state.stack.len(), 2);
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
        assert!(!g.object_view(source).tapped);
        settle(&mut g, None);
        assert!(g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.object_view(source).tapped);
        let removal = named(&mut g, p.other(), "Murder", Zone::Hand);
        g.state.priority = p.other();
        cast(&mut g, removal, Some(Target::Card(source)));
        settle(&mut g, None);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        // Drive real cleanup to expire the temporary keyword without untapping.
        for _ in 0..120 {
            if !g.keywords(source).contains(Keywords::INDESTRUCTIBLE) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::FinishAttackers
                            | Action::FinishBlockers
                            | Action::Discard { .. }
                    )
                })
                .unwrap();
            replay(&mut g, a);
        }
        assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
        assert!(g.object_view(source).tapped);
    }
}
#[test]
fn sackville_entry_can_decline_or_pay_once_and_token_sacrifice_is_a_separate_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let resource = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "The Sackville-Bagginses", Zone::Hand);
        cast(&mut g, source, None);
        pass_pair(&mut g);
        pass_pair(&mut g);
        for id in [source, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::Sacrifice { object });
        }
        let saved = g.snapshot();
        replay(&mut g, Action::DeclineEffectPayment);
        assert_eq!(g.object_view(resource).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(tokens(&g, p, "Treasure").is_empty());
        g.restore(&saved).unwrap();
        let object = g.object_ref(resource);
        replay(&mut g, Action::Sacrifice { object });
        assert_eq!(g.object_view(resource).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(tokens(&g, p, "Treasure").len(), 1);
        assert!(g.state.stack.is_empty());
        hidden(&g, p);
        let token = tokens(&g, p, "Treasure")[0].id;
        let mana = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==token))
            .unwrap();
        replay(&mut g, mana);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn old_egotist_resolution_cannot_tap_or_protect_a_returned_incarnation() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = named(&mut g, p, "Popular Egotist", Zone::Battlefield);
    let body = named(&mut g, p, "Bear Cub", Zone::Battlefield);
    announce(&mut g, source, 0);
    choose_cost(&mut g, body);
    pay(&mut g);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    let old = g.object_ref(source);
    g.move_object(source, Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(old, g.object_ref(source));
    settle(&mut g, None);
    assert!(!g.object_view(source).tapped);
    assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 19);
}
#[test]
fn sackville_token_event_uses_predeparture_traits_and_can_outlive_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "The Sackville-Bagginses", Zone::Battlefield);
        let factory = Card::catalogue()
            .find(|c| c.is_token() && c.has_subtype("Treasure"))
            .unwrap();
        g.create_tokens(p, factory, 1, false);
        let token = tokens(&g, p, "Treasure")[0].id;
        g.sacrifice_batch(&[(p, source), (p, token)]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        settle(&mut g, None);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn token_creation_capacity_failure_rolls_back_the_earlier_draw_in_same_resolution() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = named(&mut g, p, "Biomechan Engineer", Zone::Battlefield);
    let factory = Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype("Treasure"))
        .unwrap();
    g.create_tokens(p, factory, 1, false);
    let id = tokens(&g, p, "Treasure")[0].id;
    g.move_object(id, Zone::Graveyard);
    g.state_based_actions();
    assert!(g.state.free_tokens.contains(&id));
    g.state.objects[id as usize].generation = u64::MAX;
    activate(&mut g, source, 0);
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let r = g.revision();
    let actor = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), r);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert!(tokens(&g, p, "Robot").is_empty());
    }
}
#[test]
fn both_new_legends_use_real_owned_keep_choices_and_legend_departure_is_not_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for name in ["Lunatic Pandora", "The Sackville-Bagginses"] {
            let mut g = setup(p);
            named(&mut g, p, "Popular Egotist", Zone::Battlefield);
            let old = named(&mut g, p, name, Zone::Battlefield);
            let new = named(&mut g, p, name, Zone::Hand);
            let foreign = named(&mut g, p.other(), name, Zone::Battlefield);
            cast(&mut g, new, None);
            pass_pair(&mut g);
            assert_eq!(g.actor(), Some(p));
            assert_eq!(g.observe(p).decision, "legend_rule");
            let wrong = g.object_ref(foreign);
            reject(&mut g, Action::KeepLegend { object: wrong });
            let object = g.object_ref(new);
            replay(&mut g, Action::KeepLegend { object });
            assert_eq!(g.object_view(old).zone, Zone::Graveyard);
            assert_eq!(g.object_view(new).zone, Zone::Battlefield);
            settle(&mut g, None);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
#[test]
fn biomechan_entry_lander_has_real_paid_sacrifice_search_and_tapped_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Biomechan Engineer", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g, None);
        let id = tokens(&g, p, "Lander")[0].id;
        let library = g.state.players[p.index()].library.len();
        activate(&mut g, id, 0);
        assert_eq!(g.object_view(id).zone, Zone::Outside);
        pass_pair(&mut g);
        assert!(g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        settle(&mut g, None);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(
            g.state.objects.iter().any(|o| o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.is_land()
                && o.tapped)
        );
    }
}
#[test]
fn countered_seize_keeps_paid_discard_but_never_draws_or_creates_treasure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = named(&mut g, p, "Seize the Spoils", Zone::Hand);
        let cost = named(&mut g, p, "Bear Cub", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        choose_cost(&mut g, cost);
        pay(&mut g);
        replay(&mut g, Action::Pass);
        let counter = named(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, counter, Some(Target::Spell(spell)));
        settle(&mut g, None);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert!(tokens(&g, p, "Treasure").is_empty());
    }
}
