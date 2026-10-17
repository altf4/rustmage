//! Paid temporary grants, exact death incarnations and owned rollback.
use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 3] = [
    "Not Dead After All",
    "Fake Your Own Death",
    "Undying Malice",
];
fn act(g: &mut Game, a: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap_or_else(|| panic!("illegal {a:?}, {:?}", g.observe(actor)));
    assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(saved, g.snapshot());
    let mut child = {
        let parent = g.clone();
        let c = parent.clone();
        drop(parent);
        c
    };
    child.apply_index(actor, rev, index).unwrap();
    g.apply(actor, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
    g.apply(actor, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn choose(g: &mut Game) {
    let actions = g.legal_actions();
    let a = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone()
    };
    act(g, a);
}
fn priority(g: &mut Game, p: Player) {
    for _ in 0..30 {
        if g.actor() == Some(p) && g.state.pending == Pending::Priority {
            return;
        }
        choose(g);
    }
    panic!("priority");
}

fn drive(g: &mut Game, target: Option<Target>) {
    for _ in 0..160 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            target
                .and_then(|t| {
                    actions
                        .iter()
                        .find(|a| **a == Action::ChooseTarget { target: t })
                })
                .or_else(|| {
                    actions.iter().find(|a| {
                        matches!(
                            a,
                            Action::FinishTargets
                                | Action::PayCosts { .. }
                                | Action::PutTrigger { .. }
                        )
                    })
                })
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        act(g, a);
    }
    panic!("unsettled event game");
}
fn paid(g: &mut Game, p: Player, n: &str, target: Option<Target>) -> CardId {
    priority(g, p);
    g.state.players[p.index()].mana = [30; 6];
    let id = named(g, p, n, Zone::Hand);
    act(g, Action::AnnounceCast { card: id });
    drive(g, target);
    id
}
fn foreign_grants(actor: Player, names: [&str; 3]) -> (Game, [CardId; 3]) {
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
    drive(&mut g, None);
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

fn tokens(g: &Game, p: Player, name: &str) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card.token_info().is_some_and(|t| t.name == name)
        })
        .count()
}
#[test]
fn paid_complete_death_returns_both_seats_with_entry_counter_and_role() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let bear = paid(&mut g, p, "Bear Cub", None);
            let old = g.object_ref(bear);
            paid(&mut g, p, name, Some(Target::Card(bear)));
            let grants = g.observe(p).temporary_abilities;
            assert_eq!(grants.len(), 1);
            assert_eq!(grants, g.observe(p.other()).temporary_abilities);
            assert_eq!(grants[0].recipient, old);
            assert_eq!(grants[0].granting_card, Card::from_name(name).unwrap());
            paid(&mut g, p, "Lightning Bolt", Some(Target::Card(bear)));
            let returned = g.object_view(bear);
            assert_eq!(returned.zone, Zone::Battlefield);
            assert_eq!(returned.owner, p);
            assert_eq!(returned.controller, p);
            assert!(returned.tapped);
            assert_ne!(g.object_ref(bear), old);
            assert_eq!(returned.counters.plus_one, u64::from(i == 2));
            assert_eq!(returned.power, if i == 1 { 2 } else { 3 });
            assert_eq!(tokens(&g, p, "Wicked"), usize::from(i == 0));
            assert_eq!(tokens(&g, p, "Treasure Token"), usize::from(i == 1));
            assert!(g.state.temporary_abilities.is_none());
        }
    }
}
#[test]
fn repeated_grants_return_only_old_grave_incarnation_but_each_creates_treasure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", None);
        for _ in 0..2 {
            paid(&mut g, p, NAMES[1], Some(Target::Card(bear)));
        }
        assert_eq!(g.observe(p).temporary_abilities.len(), 2);
        assert_eq!(g.object_view(bear).power, 6);
        paid(&mut g, p, "Lightning Bolt", Some(Target::Card(bear)));
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).power, 2);
        assert_eq!(tokens(&g, p, "Treasure Token"), 2);
        assert!(g.state.temporary_abilities.is_none());
    }
}
#[test]
fn bounce_and_recast_same_card_cannot_inherit_old_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", None);
        paid(&mut g, p, NAMES[2], Some(Target::Card(bear)));
        paid(&mut g, p, "Unsummon", Some(Target::Card(bear)));
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert!(g.state.temporary_abilities.is_none());
        priority(&mut g, p);
        act(&mut g, Action::AnnounceCast { card: bear });
        drive(&mut g, None);
        paid(&mut g, p, "Lightning Bolt", Some(Target::Card(bear)));
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}
#[test]
fn foreign_permission_grants_return_to_owner_but_tokens_belong_to_death_controller() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let (mut g, [grant, bear, _]) = foreign_grants(p, [name, "Bear Cub", "Forest"]);
            priority(&mut g, p);
            act(&mut g, Action::AnnounceExile { card: bear });
            drive(&mut g, None);
            // The caster's counter replacement must not affect a creature
            // returning under its different immutable owner's control.
            if i == 2 {
                paid(&mut g, p, "Doubling Season", None);
            }
            priority(&mut g, p);
            act(&mut g, Action::AnnounceExile { card: grant });
            drive(&mut g, Some(Target::Card(bear)));
            assert_eq!(g.object_view(bear).owner, p.other());
            assert_eq!(g.object_view(bear).controller, p);
            paid(&mut g, p, "Lightning Bolt", Some(Target::Card(bear)));
            let v = g.object_view(bear);
            assert_eq!(v.zone, Zone::Battlefield);
            assert_eq!(v.controller, p.other());
            assert_eq!(v.owner, p.other());
            assert!(v.tapped);
            assert_eq!(v.counters.plus_one, u64::from(i == 2));
            assert_eq!(tokens(&g, p, "Wicked"), usize::from(i == 0));
            assert_eq!(tokens(&g, p.other(), "Wicked"), 0);
            assert_eq!(tokens(&g, p, "Treasure Token"), 2 + usize::from(i == 1));
        }
    }
}

#[test]
fn until_end_turn_grant_expires_before_cleanup_induced_death() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", None);
        paid(&mut g, p, "Giant Growth", Some(Target::Card(bear)));
        paid(&mut g, p, NAMES[1], Some(Target::Card(bear)));
        // Native boundary: persistent negative counters expose lethal toughness
        // only when real cleanup simultaneously removes growth and the grant.
        g.state.objects[bear as usize].counters.minus_one = 2;
        assert!(g.object_view(bear).toughness > 0);
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        drive(&mut g, None);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert!(g.state.temporary_abilities.is_none());
        assert_eq!(tokens(&g, p, "Treasure Token"), 0);
    }
}
#[test]
fn actual_doubling_season_replaces_counter_on_return_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", None);
        let bear = paid(&mut g, p, "Bear Cub", None);
        paid(&mut g, p, NAMES[2], Some(Target::Card(bear)));
        paid(&mut g, p, "Lightning Bolt", Some(Target::Card(bear)));
        let v = g.object_view(bear);
        assert_eq!(v.zone, Zone::Battlefield);
        assert!(v.tapped);
        assert_eq!(v.counters.plus_one, 2);
        assert_eq!((v.power, v.toughness), (4, 4));
    }
}

#[test]
fn trigger_controller_is_dying_creatures_controller_not_granting_spells_caster() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(q);
        let bear = paid(&mut g, q, "Bear Cub", None);
        paid(&mut g, p, NAMES[1], Some(Target::Card(bear)));
        assert_eq!(g.observe(p).temporary_abilities[0].grantor_controller, p);
        paid(&mut g, p, "Lightning Bolt", Some(Target::Card(bear)));
        assert_eq!(g.object_view(bear).controller, q);
        assert_eq!(tokens(&g, p, "Treasure Token"), 0);
        assert_eq!(tokens(&g, q, "Treasure Token"), 1);
    }
}
#[test]
fn sparse_registry_logical_memory_counts_each_grant_and_owned_clone() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = paid(&mut g, p, "Bear Cub", None);
        let spells = [
            named(&mut g, p, NAMES[2], Zone::Hand),
            named(&mut g, p, NAMES[2], Zone::Hand),
        ];
        let baseline = g.state_bytes();
        assert!(g.state.temporary_abilities.is_none());
        for (i, card) in spells.into_iter().enumerate() {
            priority(&mut g, p);
            act(&mut g, Action::AnnounceCast { card });
            drive(&mut g, Some(Target::Card(bear)));
            let expected = std::mem::size_of::<Vec<TemporaryAbilityView>>()
                + (i + 1) * std::mem::size_of::<TemporaryAbilityView>();
            assert_eq!(g.state_bytes(), baseline + expected);
            let owned = {
                let parent = g.clone();
                parent.clone()
            };
            assert_eq!(owned.state_bytes(), g.state_bytes());
            assert_eq!(owned.snapshot(), g.snapshot());
        }
        g.state.step = Step::Cleanup;
        g.finish_cleanup();
        assert!(g.state.temporary_abilities.is_none());
        assert_eq!(g.temporary_ability_bytes(), 0);
    }
}
#[test]
fn real_lantern_response_prevents_return_but_independent_treasure_still_resolves() {
    for p in [Player::P0, Player::P1] {
        let (mut g, [grant, bear, _]) = foreign_grants(p, [NAMES[1], "Bear Cub", "Forest"]);
        priority(&mut g, p);
        act(&mut g, Action::AnnounceExile { card: bear });
        drive(&mut g, None);
        priority(&mut g, p);
        act(&mut g, Action::AnnounceExile { card: grant });
        drive(&mut g, Some(Target::Card(bear)));
        let lantern = paid(&mut g, p, "Soul-Guide Lantern", Some(Target::Card(grant)));
        // Moving the original granting spell again does not revoke its ability.
        assert_eq!(g.object_view(grant).zone, Zone::Exile);
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        priority(&mut g, p);
        let bolt = named(&mut g, p, "Lightning Bolt", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: bolt });
        for _ in 0..100 {
            if g.state.pending == Pending::Priority
                && g.state
                    .stack
                    .iter()
                    .any(|s| s.trigger_definition() == Card::from_name(NAMES[1]).unwrap())
            {
                break;
            }
            let a = g
                .legal_actions()
                .iter()
                .find(|a| {
                    **a == Action::ChooseTarget {
                        target: Target::Card(bear),
                    }
                })
                .cloned();
            if let Some(a) = a {
                act(&mut g, a);
            } else {
                choose(&mut g);
            }
        }
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert!(!g.state.stack.is_empty());
        priority(&mut g, p);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: lantern,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        drive(&mut g, None);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(g.object_view(lantern).zone, Zone::Graveyard);
        assert_eq!(tokens(&g, p, "Treasure Token"), 3);
    }
}

#[test]
fn actual_crewed_vehicle_returns_as_noncreature_and_cannot_receive_wicked_aura() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vehicle = paid(&mut g, p, "Cultivator's Caravan", None);
        let crew = paid(&mut g, p, "Hill Giant", None);
        priority(&mut g, p);
        let ability = g.state.objects[vehicle as usize]
            .card
            .activations()
            .iter()
            .position(|a| a.crew_minimum().is_some())
            .unwrap() as u16;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: vehicle,
                ability,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(crew);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drive(&mut g, None);
        assert!(g.is_creature(vehicle));
        paid(&mut g, p, NAMES[0], Some(Target::Card(vehicle)));
        paid(&mut g, p, "Murder", Some(Target::Card(vehicle)));
        let v = g.object_view(vehicle);
        assert_eq!(v.zone, Zone::Battlefield);
        assert!(v.tapped);
        assert!(!g.is_creature(vehicle));
        assert_eq!(tokens(&g, p, "Wicked"), 0);
    }
}

#[test]
fn printed_self_death_and_multiple_granted_definitions_all_keep_own_programs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let sage = paid(&mut g, p, "Infestation Sage", None);
        paid(&mut g, p, NAMES[1], Some(Target::Card(sage)));
        paid(&mut g, p, NAMES[2], Some(Target::Card(sage)));
        paid(&mut g, p, "Lightning Bolt", Some(Target::Card(sage)));
        assert_eq!(g.object_view(sage).zone, Zone::Battlefield);
        assert_eq!(tokens(&g, p, "Treasure Token"), 1);
        let insects: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card.is_token()
                    && o.card.has_subtype("Insect")
            })
            .collect();
        assert_eq!(insects.len(), 1);
        assert!(insects[0].card.info().keywords.contains(Keywords::FLYING));
        assert!(g.state.temporary_abilities.is_none());
    }
}
