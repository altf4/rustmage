//! Foreign-owned casts: actor permissions, physical ownership and owned branches.
use super::tests::{scenario, spawn};
use super::*;

fn setup(actor: Player) -> Game {
    let mut g = scenario();
    g.state.active = actor;
    g.state.priority = actor;
    g.state.players[actor.index()].mana = [30; 6];
    g
}

fn act(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let before = g.snapshot();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .unwrap_or_else(|| {
            panic!(
                "illegal fixture action {action:?} at {:?}; legal {:?}",
                g.state.pending,
                g.legal_actions()
            )
        });
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for p in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(child.observe(p)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}

fn settle(g: &mut Game) {
    for _ in 0..200 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::FinishSearch))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}

fn finish_announcement(g: &mut Game) {
    for _ in 0..100 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(g, action);
    }
    panic!("announcement did not finish");
}

fn grant(g: &mut Game, actor: Player, name: &str) -> CardId {
    let id = spawn(
        g,
        actor.other(),
        Card::from_name(name).unwrap(),
        Zone::Exile,
    );
    g.state.exile_permissions.push(ExilePlayPermission {
        object: g.object_ref(id),
        player: actor,
        expires_turn: g.state.turn,
    });
    g.state.exile_permissions.sort_by_key(|p| p.object.card);
    id
}

fn korvold_grants(actor: Player) -> (Game, [CardId; 3]) {
    korvold_grants_for(
        actor,
        [
            "Grabby Giant // That's Mine",
            "Bloomvine Regent // Claim Territory",
            "Midgar, City of Mako // Reactor Raid",
        ],
    )
}

fn korvold_grants_for(actor: Player, names: [&str; 3]) -> (Game, [CardId; 3]) {
    let mut g = setup(actor);
    let cards = names.map(|name| {
        spawn(
            &mut g,
            actor.other(),
            Card::from_name(name).unwrap(),
            Zone::Library,
        )
    });
    // The opponent naturally draws once on each intervening turn before III.
    for _ in 0..2 {
        spawn(&mut g, actor.other(), Card::Forest, Zone::Library);
    }
    let saga = spawn(
        &mut g,
        actor,
        Card::from_name("Korvold and the Noble Thief").unwrap(),
        Zone::Hand,
    );
    act(&mut g, Action::AnnounceCast { card: saga });
    settle(&mut g);
    assert_eq!(g.state.objects[saga as usize].counters.lore, 1);
    let mut second = false;
    for _ in 0..1500 {
        if g.state.objects[saga as usize].counters.lore == 2 {
            second = true;
        }
        if cards.iter().all(|&id| g.can_play_exiled(actor, id)) {
            break;
        }
        let actions = g.legal_actions();
        let action = actions
            .iter()
            .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(&mut g, action);
    }
    assert!(second);
    assert_eq!(g.state.objects[saga as usize].zone, Zone::Graveyard);
    assert!(cards.iter().all(|&id| g.can_play_exiled(actor, id)));
    assert!(
        cards
            .iter()
            .all(|&id| !g.can_play_exiled(actor.other(), id))
    );
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.controller == actor
                && o.card
                    .token_info()
                    .is_some_and(|t| t.name == "Treasure Token"))
            .count(),
        2
    );
    g.state.players[actor.index()].mana = [30; 6];
    (g, cards)
}

#[test]
fn real_paid_korvold_all_three_chapters_grant_foreign_routes_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let (g, [giant, bloom, land]) = korvold_grants(actor);
        for action in [
            Action::AnnounceExile { card: giant },
            Action::AnnounceAdventure { card: giant },
            Action::AnnounceExile { card: bloom },
            Action::AnnounceOmen { card: bloom },
            Action::PlayLand { card: land },
            Action::AnnounceAdventure { card: land },
        ] {
            let mut child = g.clone();
            act(&mut child, action);
            settle(&mut child);
        }
    }
}

#[test]
fn paid_land_grants_use_foreign_land_casters_quota_and_offturn_owner_grants_expire() {
    for actor in [Player::P0, Player::P1] {
        let (base, [beans, midgar, ishgard]) = korvold_grants_for(
            actor,
            [
                "Beanstalk Wurm // Plant Beans",
                "Midgar, City of Mako // Reactor Raid",
                "Ishgard, the Holy See // Faith & Grief",
            ],
        );
        for (land, spare) in [(midgar, ishgard), (ishgard, midgar)] {
            let mut g = base.clone();
            let owner = actor.other();
            spawn(
                &mut g,
                owner,
                Card::from_name("Whirlwing Stormbrood // Dynamic Soar").unwrap(),
                Zone::Battlefield,
            );
            let owners_beans = spawn(
                &mut g,
                owner,
                Card::from_name("Beanstalk Wurm // Plant Beans").unwrap(),
                Zone::Hand,
            );
            g.state.players[owner.index()].mana = [30; 6];
            act(&mut g, Action::Pass);
            act(&mut g, Action::AnnounceAdventure { card: owners_beans });
            settle(&mut g);
            assert_eq!(g.state.active, actor);
            assert_eq!(g.state.players[owner.index()].extra_land_plays_remaining, 1);
            act(&mut g, Action::AnnounceAdventure { card: beans });
            settle(&mut g);
            assert_eq!(g.state.players[actor.index()].extra_land_plays_remaining, 1);
            let basic = spawn(&mut g, actor, Card::Forest, Zone::Hand);
            act(&mut g, Action::PlayLand { card: basic });
            act(&mut g, Action::PlayLand { card: land });
            assert_eq!(g.state.objects[land as usize].owner, owner);
            assert_eq!(g.state.objects[land as usize].controller, actor);
            assert_eq!(g.state.players[actor.index()].extra_land_plays_remaining, 0);
            assert_eq!(g.state.players[owner.index()].extra_land_plays_remaining, 1);
            let before = g.snapshot();
            assert_eq!(
                g.apply(actor, Action::PlayLand { card: spare }),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), before);
            let turn = g.state.turn;
            for _ in 0..200 {
                if g.state.turn > turn
                    && g.state.active == owner
                    && g.state.step == Step::Main1
                    && g.state.pending == Pending::Priority
                {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap();
                act(&mut g, action);
            }
            assert_eq!(g.state.active, owner);
            assert_eq!(g.state.step, Step::Main1);
            assert_eq!(g.state.players[owner.index()].extra_land_plays_remaining, 0);
            assert_eq!(g.state.players[actor.index()].extra_land_plays_remaining, 0);
            assert!(
                g.state.objects[land as usize].tapped,
                "the owner's untap cannot untap a foreign-controlled land"
            );
            assert!(!g.can_play_exiled(owner, spare));
            let first = spawn(&mut g, owner, Card::Forest, Zone::Hand);
            let second = spawn(&mut g, owner, Card::Forest, Zone::Hand);
            act(&mut g, Action::PlayLand { card: first });
            let before = g.snapshot();
            assert_eq!(
                g.apply(owner, Action::PlayLand { card: second }),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), before);
            for _ in 0..200 {
                if g.state.active == actor
                    && g.state.step == Step::Main1
                    && g.state.pending == Pending::Priority
                {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| !matches!(a, Action::Concede))
                    .unwrap();
                act(&mut g, action);
            }
            assert!(!g.state.objects[land as usize].tapped);
            let mana = g.legal_actions().into_iter().find(|a| matches!(a, Action::TapMana { card } | Action::AnnounceMana { card, .. } if *card == land)).expect("caster can use foreign land");
            act(&mut g, mana);
            settle(&mut g);
            assert!(g.state.players[actor.index()].mana.iter().sum::<u32>() > 0);
            assert_eq!(g.state.players[owner.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn korvold_foreign_adventure_right_outlives_generic_deadline_and_stays_normal_only() {
    for actor in [Player::P0, Player::P1] {
        let (mut g, [giant, _, _]) = korvold_grants(actor);
        let turn = g.state.turn;
        act(&mut g, Action::AnnounceAdventure { card: giant });
        settle(&mut g);
        assert!(g.can_play_adventure(actor, giant));
        assert!(!g.can_play_adventure(actor.other(), giant));
        assert!(
            !g.state
                .exile_permissions
                .iter()
                .any(|p| p.object.card == giant)
        );
        for _ in 0..300 {
            if g.state.turn > turn
                && g.state.active == actor
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            act(&mut g, action);
        }
        assert!(g.state.turn > turn);
        assert!(g.can_play_adventure(actor, giant));
        let before = g.snapshot();
        assert_eq!(
            g.apply(actor, Action::AnnounceAdventure { card: giant }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        g.state.players[actor.index()].mana = [30; 6];
        act(&mut g, Action::AnnounceExile { card: giant });
        settle(&mut g);
        assert_eq!(g.state.objects[giant as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[giant as usize].controller, actor);
        assert_eq!(g.state.objects[giant as usize].owner, actor.other());
    }
}

#[test]
fn paid_foreign_omen_searches_caster_library_but_shuffles_source_into_owner_library() {
    for actor in [Player::P0, Player::P1] {
        let (mut g, [_, bloom, _]) = korvold_grants(actor);
        let owner = actor.other();
        let caster_len = g.state.players[actor.index()].library.len();
        let owner_len = g.state.players[owner.index()].library.len();
        let mut expected_rng = g.state.rng;
        let mut expected_caster = g.state.players[actor.index()].library.clone();
        let mut expected_owner = g.state.players[owner.index()].library.clone();
        act(&mut g, Action::AnnounceOmen { card: bloom });
        for _ in 0..100 {
            if matches!(g.state.pending, Pending::EffectSearch { .. }) {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            act(&mut g, action);
        }
        let mut selected = Vec::new();
        for _ in 0..2 {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::ChooseSearch { .. }))
                .unwrap();
            if let Action::ChooseSearch { card } = action {
                selected.push(card);
            }
            act(&mut g, action);
        }
        act(&mut g, Action::FinishSearch);
        let own = g.observe(actor).search_distribution.unwrap();
        let other = g.observe(owner).search_distribution.unwrap();
        assert_eq!(
            serde_json::to_value(&own).unwrap(),
            serde_json::to_value(&other).unwrap()
        );
        assert_eq!(own.candidates.len(), 2);
        assert!(own.candidates.iter().all(|o| selected.contains(&o.id)));
        act(
            &mut g,
            Action::ChooseSearchBattlefield { card: selected[0] },
        );
        settle(&mut g);
        assert_eq!(g.state.players[actor.index()].library.len(), caster_len - 2);
        assert_eq!(g.state.players[owner.index()].library.len(), owner_len + 1);
        assert_eq!(g.state.objects[bloom as usize].zone, Zone::Library);
        assert_eq!(g.state.objects[bloom as usize].owner, owner);
        assert_eq!(g.state.objects[selected[0] as usize].controller, actor);
        assert_eq!(g.state.objects[selected[1] as usize].zone, Zone::Hand);
        assert!(
            g.observe(owner)
                .objects
                .iter()
                .any(|o| o.id == selected[1] && o.zone == Zone::Hand)
        );
        assert!(!g.can_play_adventure(actor, bloom));
        expected_caster.retain(|id| !selected.contains(id));
        expected_rng.shuffle(&mut expected_caster);
        expected_owner.push(bloom);
        expected_rng.shuffle(&mut expected_owner);
        assert_eq!(g.state.rng, expected_rng);
        assert_eq!(g.state.players[actor.index()].library, expected_caster);
        assert_eq!(g.state.players[owner.index()].library, expected_owner);
    }
}

#[test]
fn paid_foreign_inset_counters_route_to_owner_graveyard_without_exile_rights_or_shuffle() {
    for actor in [Player::P0, Player::P1] {
        let (base, [giant, bloom, _]) = korvold_grants(actor);
        for (id, action) in [
            (giant, Action::AnnounceAdventure { card: giant }),
            (bloom, Action::AnnounceOmen { card: bloom }),
        ] {
            let mut g = base.clone();
            let owner = actor.other();
            let libraries = [
                g.state.players[0].library.clone(),
                g.state.players[1].library.clone(),
            ];
            act(&mut g, action);
            finish_announcement(&mut g);
            let counter = spawn(
                &mut g,
                owner,
                Card::from_name("Negate").unwrap(),
                Zone::Hand,
            );
            g.state.players[owner.index()].mana = [30; 6];
            act(&mut g, Action::Pass);
            act(&mut g, Action::AnnounceCast { card: counter });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Spell(id),
                },
            );
            finish_announcement(&mut g);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.objects[id as usize].owner, owner);
            assert!(!g.can_play_exiled(actor, id));
            assert!(!g.can_play_exiled(owner, id));
            assert_eq!(g.state.players[0].library, libraries[0]);
            assert_eq!(g.state.players[1].library, libraries[1]);
        }
    }
}

#[test]
fn stale_or_other_player_grant_cannot_mask_independent_valid_permission() {
    for actor in [Player::P0, Player::P1] {
        let mut g = setup(actor);
        let id = grant(&mut g, actor, "Grabby Giant // That's Mine");
        let reference = g.object_ref(id);
        let mut stale = reference;
        stale.generation += 1;
        g.state.exile_permissions.insert(
            0,
            ExilePlayPermission {
                object: stale,
                player: actor,
                expires_turn: g.state.turn,
            },
        );
        g.state.exile_permissions.insert(
            1,
            ExilePlayPermission {
                object: reference,
                player: actor.other(),
                expires_turn: g.state.turn,
            },
        );
        assert!(g.can_play_exiled(actor, id));
        assert!(g.can_play_exiled(actor.other(), id));
        act(&mut g, Action::AnnounceAdventure { card: id });
        settle(&mut g);
        assert!(g.can_play_adventure(actor, id));
        assert!(!g.can_play_exiled(actor.other(), id));
    }
}

#[test]
fn korvold_foreign_cost_reduction_counts_casters_controlled_power_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let (mut g, [spell, _, _]) = korvold_grants_for(
            actor,
            [
                "Ghalta, Primal Hunger",
                "Corrupted Conviction",
                "Sun-Dappled Celebrant",
            ],
        );
        g.state.players[actor.index()].mana = [0, 0, 0, 0, 2, 0];
        let helpers: Vec<_> = (0..5)
            .map(|_| {
                spawn(
                    &mut g,
                    actor.other(),
                    Card::from_name("Bear Cub").unwrap(),
                    Zone::Battlefield,
                )
            })
            .collect();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceExile { card: spell })
        );
        for &id in &helpers {
            g.state.objects[id as usize].controller = actor;
        }
        act(&mut g, Action::AnnounceExile { card: spell });
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[spell as usize].controller, actor);
        assert_eq!(g.state.objects[spell as usize].owner, actor.other());
        assert_eq!(g.state.players[actor.index()].mana, [0; 6]);
    }
}

#[test]
fn korvold_foreign_additional_cost_sacrifices_casters_resource_not_owners_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let (mut g, [_, spell, _]) = korvold_grants_for(
            actor,
            [
                "Ghalta, Primal Hunger",
                "Corrupted Conviction",
                "Sun-Dappled Celebrant",
            ],
        );
        let cost = spawn(
            &mut g,
            actor.other(),
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[cost as usize].controller = actor;
        let decoy = spawn(
            &mut g,
            actor,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        g.state.objects[decoy as usize].controller = actor.other();
        g.state.players[actor.index()].mana = [0, 0, 1, 0, 0, 0];
        let draws = g.state.players[actor.index()].cards_drawn_this_turn;
        act(&mut g, Action::AnnounceExile { card: spell });
        let before = g.snapshot();
        assert_eq!(
            g.apply(
                actor,
                Action::ChooseAdditionalCost {
                    object: g.object_ref(decoy)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        let object = g.object_ref(cost);
        act(&mut g, Action::ChooseAdditionalCost { object });
        settle(&mut g);
        assert_eq!(g.state.objects[cost as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[cost as usize].owner, actor.other());
        assert_eq!(g.state.objects[decoy as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[actor.index()].cards_drawn_this_turn,
            draws + 2
        );
        assert_eq!(g.state.players[actor.index()].mana, [0; 6]);
    }
}

#[test]
fn korvold_foreign_convoke_uses_casters_controlled_creatures_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let (mut g, [_, _, spell]) = korvold_grants_for(
            actor,
            [
                "Ghalta, Primal Hunger",
                "Corrupted Conviction",
                "Sun-Dappled Celebrant",
            ],
        );
        g.state.players[actor.index()].mana = [0; 6];
        let white = Card::catalogue()
            .find(|c| {
                c.is_supported()
                    && c.is_creature()
                    && c.colors() == 1
                    && c.triggers().is_empty()
                    && !c.is_legendary()
            })
            .unwrap();
        let helpers: Vec<_> = (0..6)
            .map(|_| spawn(&mut g, actor.other(), white, Zone::Battlefield))
            .collect();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceExile { card: spell })
        );
        for &id in &helpers {
            g.state.objects[id as usize].controller = actor;
        }
        act(&mut g, Action::AnnounceExile { card: spell });
        act(&mut g, Action::BeginConvoke);
        for (i, &id) in helpers.iter().enumerate() {
            let object = g.object_ref(id);
            act(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution: if i < 2 {
                        ConvokeContribution::White
                    } else {
                        ConvokeContribution::Generic
                    },
                },
            );
        }
        settle(&mut g);
        assert!(
            helpers
                .iter()
                .all(|&id| g.state.objects[id as usize].tapped)
        );
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[spell as usize].controller, actor);
        assert_eq!(g.state.objects[spell as usize].owner, actor.other());
        assert_eq!(g.state.players[actor.index()].mana, [0; 6]);
    }
}

#[test]
fn paid_foreign_bloomvine_enters_for_caster_and_owner_is_immutable_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let mut g = setup(actor);
        let id = grant(&mut g, actor, "Bloomvine Regent // Claim Territory");
        act(&mut g, Action::AnnounceExile { card: id });
        assert_eq!(g.state.objects[id as usize].controller, actor);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[id as usize].owner, actor.other());
        assert_eq!(g.state.objects[id as usize].controller, actor);
        assert_eq!(g.state.players[actor.index()].life, 23);
        assert_eq!(g.state.players[actor.other().index()].life, 20);
        assert!(g.state.players[actor.index()].mana.iter().sum::<u32>() < 180);
    }
}

#[test]
fn paid_foreign_land_uses_caster_allowance_and_keeps_physical_owner_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let mut g = setup(actor);
        let id = grant(&mut g, actor, "Midgar, City of Mako // Reactor Raid");
        act(&mut g, Action::PlayLand { card: id });
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[id as usize].owner, actor.other());
        assert_eq!(g.state.objects[id as usize].controller, actor);
        assert!(g.state.objects[id as usize].tapped);
        assert!(g.state.land_played);
    }
}

#[test]
fn paid_foreign_etb_trigger_keeps_caster_after_paid_source_destruction_both_seats() {
    for actor in [Player::P0, Player::P1] {
        let mut g = setup(actor);
        let id = grant(&mut g, actor, "Bloomvine Regent // Claim Territory");
        act(&mut g, Action::AnnounceExile { card: id });
        for _ in 0..100 {
            if g.state.pending == Pending::Priority
                && g.state
                    .stack
                    .iter()
                    .any(|s| s.card == id && matches!(s.kind, StackKind::TriggeredAbility { .. }))
            {
                break;
            }
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            act(&mut g, action);
        }
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.card == id && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        );
        assert_eq!(g.state.players[actor.index()].life, 20);
        let murder = spawn(
            &mut g,
            actor.other(),
            Card::from_name("Murder").unwrap(),
            Zone::Hand,
        );
        g.state.players[actor.other().index()].mana = [30; 6];
        if g.actor() != Some(actor.other()) {
            act(&mut g, Action::Pass);
        }
        act(&mut g, Action::AnnounceCast { card: murder });
        for _ in 0..100 {
            if g.state.pending == Pending::Priority
                && g.state.stack.last().is_some_and(|s| s.card == murder)
            {
                break;
            }
            let actions = g.legal_actions();
            let action = actions.iter().find(|a| matches!(a, Action::ChooseTarget { target: Target::Card(card) } if *card == id))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede))).unwrap().clone();
            act(&mut g, action);
        }
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[id as usize].owner, actor.other());
        assert_eq!(g.state.objects[id as usize].controller, actor.other());
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.card == id && s.controller == actor)
        );
        settle(&mut g);
        assert_eq!(g.state.players[actor.index()].life, 23);
        assert_eq!(g.state.players[actor.other().index()].life, 20);
    }
}
