//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("chapter or copy choice did not arrive");
}
fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && g.is_creature(id as CardId)
                && g.has_subtype(id as CardId, "Army"))
            .then_some(id as CardId)
        })
        .collect()
}
fn cast_tidings(g: &mut Game, p: Player, zone: Zone, flash: bool) -> CardId {
    let spell = put(g, p, "Tidings of War", zone);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, if flash { 3 } else { 0 }];
    match zone {
        Zone::Hand => act(g, Action::AnnounceCast { card: spell }),
        Zone::Exile => {
            g.grant_exile_play_permission(p, g.object_ref(spell), false);
            act(g, Action::AnnounceExile { card: spell });
        }
        _ if flash => act(g, Action::AnnounceFlashback { card: spell }),
        _ => {
            g.grant_graveyard_permission(p, g.object_ref(spell), GraveyardCastKind::Cast);
            act(g, Action::AnnounceCast { card: spell });
        }
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    spell
}

#[test]
fn tidings_paid_cast_origins_create_or_grow_only_own_army() {
    {
        let p = Player::P0;
        for (zone, flash, expected) in [
            (Zone::Hand, false, 1),
            (Zone::Graveyard, true, 3),
            (Zone::Graveyard, false, 3),
            (Zone::Exile, false, 1),
        ] {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "@token/goblin_army", Zone::Battlefield);
            g.state.objects[enemy as usize].counters.plus_one = 2;
            let spell = cast_tidings(&mut g, p, zone, flash);
            settle(&mut g);
            let own = armies(&g, p);
            assert_eq!(own.len(), 1);
            assert_eq!(stats(&g, own[0]), (expected, expected));
            assert!(g.has_subtype(own[0], "Goblin"));
            assert_eq!(g.object_view(own[0]).colors, 4, "black token");
            assert_eq!(stats(&g, enemy), (2, 2));
            assert_eq!(
                g.object_view(spell).zone,
                if flash { Zone::Exile } else { Zone::Graveyard }
            );
            // A second ordinary cast grows the same Army instead of creating another.
            cast_tidings(&mut g, p, Zone::Hand, false);
            settle(&mut g);
            assert_eq!(armies(&g, p), own);
            assert_eq!(stats(&g, own[0]), (expected + 1, expected + 1));
        }
    }
}

#[test]
fn tidings_uncast_flashback_copy_amasses_one_original_amasses_three() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let spell = cast_tidings(&mut g, p, Zone::Graveyard, true);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(artifact);
        act(&mut g, Action::Sacrifice { object });
        let copy = g.state.stack.iter().find(|s| s.is_copy).unwrap();
        assert_eq!(copy.cast.unwrap().origin, Zone::Graveyard);
        let copy_id = copy.id;
        until(&mut g, |g| !g.state.stack.iter().any(|s| s.id == copy_id));
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert_eq!(stats(&g, armies(&g, p)[0]), (1, 1));
        settle(&mut g);
        assert_eq!(stats(&g, armies(&g, p)[0]), (4, 4));
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
    }
}

fn first_chapter(g: &mut Game, p: Player, nonland: bool) -> CardId {
    let discard = put(
        g,
        p,
        if nonland { "Shock" } else { "Forest" },
        Zone::Library,
    );
    let saga = put(g, p, "The Mountain-king's Return", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
    cast(g, saga, None);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.counter_count(saga, CounterKind::Lore), 1);
    assert_eq!(g.object_view(discard).zone, Zone::Hand);
    assert!(g.observe(p.other()).objects.iter().all(|o| o.id != discard));
    act(g, Action::Discard { card: discard });
    settle(g);
    let tokens: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.is_token())
        .map(|(id, _)| id as CardId)
        .collect();
    assert_eq!(tokens.len(), usize::from(nonland));
    if nonland {
        let token = g.object_view(tokens[0]);
        assert_eq!((token.power, token.toughness, token.colors), (1, 1, 1));
        assert!(g.has_subtype(tokens[0], "Human") && g.has_subtype(tokens[0], "Soldier"));
    }
    saga
}
fn chapter_target(g: &mut Game, target: CardId) {
    g.begin_step(Step::Main1);
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        })
    });
}

#[test]
fn mountain_return_paid_chapters_recruit_return_mana_three_and_optional_any_creature() {
    {
        let p = Player::P0;
        for decline in [false, true] {
            let mut g = scenario(p);
            let saga = first_chapter(&mut g, p, !decline);
            let returned = put(&mut g, p, "Thrashing Brontodon", Zone::Graveyard);
            let enemy_grave = put(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
            let too_large = put(&mut g, p, "Serra Angel", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Graveyard);
            chapter_target(&mut g, returned);
            for id in [enemy_grave, too_large, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(returned),
                },
            );
            finish_targets(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(returned).zone, Zone::Battlefield);
            assert_eq!(g.object_view(returned).controller, p);
            assert!(!g.object_view(returned).tapped);
            assert_eq!(g.counter_count(saga, CounterKind::Lore), 2);
            let enemy = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            chapter_target(&mut g, enemy);
            if !decline {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            finish_targets(&mut g);
            settle(&mut g);
            assert_eq!(
                g.counter_count(enemy, CounterKind::PlusOne),
                u64::from(!decline)
            );
            assert_eq!(g.counter_count(returned, CounterKind::PlusOne), 0);
            assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        }
    }
}
