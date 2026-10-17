//! Primitive boundaries before any new card admission; later paid full-body tests use the same path.
use super::tests::{scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported());
    spawn(g, p, c, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for q in [p, p.other()] {
        for _ in 0..8 {
            named(&mut g, q, "Forest", Zone::Library);
        }
    }
    g
}
fn rule_item(g: &mut Game, p: Player, id: CardId) -> SpellView {
    let source = named(g, p, "Shock", Zone::Hand);
    let mut item = g.cast_item(p, source, [0; 6]);
    item.targets.push(g.bind_target(Target::Card(id)));
    item
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
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
fn settle_on(g: &mut Game, chosen: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn paid_cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(
        g,
        if g.object_view(id).zone == Zone::Exile {
            Action::AnnounceExile { card: id }
        } else {
            Action::AnnounceCast { card: id }
        },
    );
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}

#[test]
fn primitive_predeath_redirect_suppresses_printed_death_and_granted_return_for_both_owners() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let owner = p.other();
        let id = named(&mut g, owner, "Clockwork Percussionist", Zone::Battlefield);
        g.state.objects[id as usize].controller = p;
        let grant = named(&mut g, p, "Not Dead After All", Zone::Hand);
        let mut grant_item = g.cast_item(p, grant, [0; 6]);
        grant_item.targets.push(g.bind_target(Target::Card(id)));
        g.grant_temporary_ability(&grant_item, 0);
        let item = rule_item(&mut g, p, id);
        g.register_exile_if_dies(&item);
        let library = g.state.players[p.index()].library.len();
        let deaths = g.state.creatures_died_this_turn;
        g.move_object(id, Zone::Graveyard);
        assert_eq!(
            (
                g.object_view(id).zone,
                g.object_view(id).owner,
                g.object_view(id).controller
            ),
            (Zone::Exile, owner, owner)
        );
        assert_eq!(g.state.creatures_died_this_turn, deaths);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.exile_permissions.is_empty());
        assert!(g.observe(p).temporary_abilities.is_empty());
        assert!(g.observe(p).death_exile_replacements.is_empty());
    }
}
#[test]
fn primitive_non_death_moves_forget_exact_rules_and_never_follow_a_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let item = rule_item(&mut g, p.other(), id);
        g.register_exile_if_dies(&item);
        let old = g.object_ref(id);
        g.move_object(id, Zone::Hand);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert!(g.observe(p).death_exile_replacements.is_empty());
        g.move_object(id, Zone::Battlefield);
        assert_ne!(g.object_ref(id), old);
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        g.register_exile_if_dies(&item);
        assert!(g.observe(p).death_exile_replacements.is_empty());
    }
}
#[test]
fn primitive_duplicate_mandatory_exile_rules_are_idempotent_and_simultaneous_destinations_stay_separate()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let item = rule_item(&mut g, p, a);
        let other = rule_item(&mut g, p.other(), a);
        let mut first = g.clone();
        first.register_exile_if_dies(&item);
        first.register_exile_if_dies(&other);
        let mut second = g.clone();
        second.register_exile_if_dies(&other);
        second.register_exile_if_dies(&item);
        for branch in [&mut first, &mut second] {
            let views = [branch.object_ref(a), branch.object_ref(b)];
            let events = branch.move_objects_raw(&[a, b], Zone::Graveyard);
            assert_eq!(events[0].destination, Zone::Exile);
            assert_eq!(events[1].destination, Zone::Graveyard);
            assert_eq!(
                events
                    .iter()
                    .map(|e| e.before.reference)
                    .collect::<Vec<_>>(),
                views
            );
            branch.finish_zone_events(&events, &[]);
            assert!(branch.observe(p).death_exile_replacements.is_empty());
        }
        assert_eq!(first.snapshot(), second.snapshot());
    }
}
#[test]
fn primitive_replacement_survives_source_movement_but_expires_before_cleanup_state_actions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let item = rule_item(&mut g, p.other(), id);
        g.register_exile_if_dies(&item);
        g.move_object(item.card, Zone::Graveyard);
        assert_eq!(g.observe(p).death_exile_replacements[0].source, item.object);
        g.state.objects[id as usize].counters.minus_one = 3;
        g.state.objects[id as usize].growth.toughness = 2;
        assert_eq!(g.stats(id).toughness, 1);
        g.finish_cleanup();
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert!(g.observe(p).death_exile_replacements.is_empty());
    }
}
#[test]
fn primitive_token_exile_is_public_before_normal_token_state_action_and_never_reenterable() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Frog"))
            .unwrap();
        let id = spawn(&mut g, p, token, Zone::Battlefield);
        let item = rule_item(&mut g, p.other(), id);
        g.register_exile_if_dies(&item);
        let event = g.move_object_raw(id, Zone::Graveyard);
        assert_eq!(event.destination, Zone::Exile);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        g.finish_zone_events(&[event], &[]);
        g.state_based_actions();
        assert_eq!(g.object_view(id).zone, Zone::Outside);
        let event = g.move_object_raw(id, Zone::Battlefield);
        assert!(!event.moved);
        assert_eq!(g.object_view(id).zone, Zone::Outside);
    }
}
#[test]
fn primitive_registry_memory_is_sparse_owned_and_exact_in_both_observations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        assert_eq!(g.death_exile_bytes(), 0);
        let id = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let item = rule_item(&mut g, p.other(), id);
        g.register_exile_if_dies(&item);
        assert_eq!(
            g.death_exile_bytes(),
            std::mem::size_of::<Vec<ExileDeathReplacementView>>()
                + std::mem::size_of::<ExileDeathReplacementView>()
        );
        assert_eq!(
            g.observe(p).death_exile_replacements,
            g.observe(p.other()).death_exile_replacements
        );
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.move_object(id, Zone::Graveyard);
        assert_eq!(child.object_view(id).zone, Zone::Exile);
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.observe(p).death_exile_replacements.len(), 1);
        g.restore(&before).unwrap();
        g.move_object(id, Zone::Graveyard);
        assert_eq!(g.snapshot(), child.snapshot());
    }
}

#[test]
fn all_eleven_paid_whole_bodies_every_mode_both_seats_and_foreign_spell_ownership() {
    let rows = [
        ("Feed the Flames", None, true, false),
        ("Bot Bashing Time", None, true, false),
        ("Elspeth's Smite", None, true, false),
        ("Narset's Rebuke", None, true, false),
        ("Obliterating Bolt", None, true, false),
        ("Scorching Dragonfire", None, true, false),
        ("Essence Burn", None, true, false),
        ("Combustion Technique", None, true, false),
        ("Suplex", Some(0), true, false),
        ("Suplex", Some(1), false, true),
        ("Agate Assault", Some(0), true, false),
        ("Agate Assault", Some(1), false, true),
        ("Pinecone Strike", Some(0), true, false),
        ("Pinecone Strike", Some(1), false, true),
        ("Pinecone Strike", Some(2), true, true),
    ];
    for p in [Player::P0, Player::P1] {
        for foreign in [false, true] {
            for (name, mode, damage, artifact) in rows {
                let mut g = setup(p);
                g.state.players[p.index()].mana = [30; 6];
                let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                if name == "Elspeth's Smite" {
                    g.state.attackers.push(AttackDeclaration {
                        card: bear,
                        defender: Some(CombatDefender::Player(p)),
                        blocked: false,
                    });
                }
                let token = Card::catalogue()
                    .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
                    .unwrap();
                let treasure = spawn(&mut g, p.other(), token, Zone::Battlefield);
                let owner = if foreign { p.other() } else { p };
                let spell = named(
                    &mut g,
                    owner,
                    name,
                    if foreign { Zone::Exile } else { Zone::Hand },
                );
                if foreign {
                    g.state.exile_permissions.push(ExilePlayPermission {
                        object: g.object_ref(spell),
                        player: p,
                        expires_turn: g.state.turn,
                    });
                }
                let mut targets = Vec::new();
                if damage {
                    targets.push(Target::Card(bear));
                }
                if artifact {
                    targets.push(Target::Card(treasure));
                }
                let before = g.state.players[p.index()].mana.iter().sum::<u32>();
                let mv = g.object_view(spell).card.fixed_mana_value();
                paid_cast(&mut g, spell, mode, &targets);
                assert_eq!(
                    before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                    u32::try_from(mv).unwrap(),
                    "{name}"
                );
                let after_paid = g.state.players[p.index()].mana;
                settle(&mut g);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard, "{name}");
                assert_eq!(g.object_view(spell).owner, owner);
                assert_eq!(
                    g.object_view(bear).zone,
                    if damage {
                        Zone::Exile
                    } else {
                        Zone::Battlefield
                    },
                    "{name}"
                );
                if artifact {
                    assert_eq!(g.object_view(treasure).zone, Zone::Outside, "{name}");
                } else {
                    assert_eq!(g.object_view(treasure).zone, Zone::Battlefield, "{name}");
                }
                assert_eq!(g.state.creatures_died_this_turn, 0, "{name}");
                assert!(g.observe(p).death_exile_replacements.is_empty());
                if name == "Narset's Rebuke" {
                    let mut expected = after_paid;
                    for i in [0, 1, 3] {
                        expected[i] += 1;
                    }
                    assert_eq!(g.state.players[p.index()].mana, expected);
                }
            }
        }
    }
}

#[test]
fn actual_nonlethal_damage_registers_then_paid_murder_exiles_without_death_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        for player in &mut g.state.players {
            player.mana = [30; 6];
        }
        let target = named(&mut g, p.other(), "Bygone Colossus", Zone::Battlefield);
        let grant = named(&mut g, p.other(), "Not Dead After All", Zone::Hand);
        // Both owners may cast the printed grant on the opposing permanent.
        g.state.priority = p.other();
        paid_cast(&mut g, grant, None, &[Target::Card(target)]);
        settle(&mut g);
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let spell = named(&mut g, p, "Scorching Dragonfire", Zone::Hand);
        paid_cast(&mut g, spell, None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(target).damage, 3);
        let rule = g.observe(p).death_exile_replacements[0];
        assert_eq!(rule.object, g.object_ref(target));
        assert_eq!(
            rule.source_card,
            Card::from_name("Scorching Dragonfire").unwrap()
        );
        assert_eq!(rule.source_controller, p);
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        paid_cast(&mut g, murder, None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        assert!(g.observe(p).temporary_abilities.is_empty());
        assert_eq!(g.state.creatures_died_this_turn, 0);
        assert!(g.state.roles.is_none());
    }
}
#[test]
fn actual_lesson_damage_counts_only_casters_graveyard_and_color_target_checks_whole_union() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [30; 6];
        for _ in 0..2 {
            named(&mut g, p, "Combustion Technique", Zone::Graveyard);
        }
        for _ in 0..3 {
            named(&mut g, p.other(), "Combustion Technique", Zone::Graveyard);
        }
        let target = named(&mut g, p.other(), "Bygone Colossus", Zone::Battlefield);
        let spell = named(&mut g, p, "Combustion Technique", Zone::Hand);
        paid_cast(&mut g, spell, None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 4);
        assert_eq!(g.observe(p).death_exile_replacements.len(), 1);
        // The spell is still on the stack when its Lesson count is evaluated.
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                .count(),
            3
        );
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let green = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let colorless = named(&mut g, p.other(), "Chrome Companion", Zone::Battlefield);
        let burn = named(&mut g, p, "Essence Burn", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: burn });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(colorless),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(green),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets))
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. }))
                })
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.object_view(green).zone, Zone::Exile);
        assert_eq!(g.object_view(colorless).zone, Zone::Battlefield);
    }
}
#[test]
fn paid_combined_mode_partial_group_invalidity_stale_target_and_counter_never_register_wrong_objects()
 {
    for p in [Player::P0, Player::P1] {
        for route in 0..4 {
            let mut g = setup(p);
            for player in &mut g.state.players {
                player.mana = [30; 6];
            }
            let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let token = Card::catalogue()
                .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
                .unwrap();
            let treasure = spawn(&mut g, p.other(), token, Zone::Battlefield);
            let spell = named(&mut g, p, "Pinecone Strike", Zone::Hand);
            paid_cast(
                &mut g,
                spell,
                Some(2),
                &[Target::Card(target), Target::Card(treasure)],
            );
            if route == 0 || route == 2 {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            if route == 1 || route == 2 {
                g.move_object(treasure, Zone::Exile);
            }
            if route == 3 {
                if g.actor() != Some(p.other()) {
                    checked(&mut g, Action::Pass);
                }
                let negate = named(&mut g, p.other(), "Negate", Zone::Hand);
                paid_cast(&mut g, negate, None, &[Target::Spell(spell)]);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if route == 1 {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(treasure).zone,
                if route == 3 {
                    Zone::Battlefield
                } else {
                    Zone::Outside
                }
            );
            assert!(g.observe(p).death_exile_replacements.is_empty());
        }
    }
}
