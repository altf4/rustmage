//! Public subtype choice, live creature count and ordinary Equipment combat.
use super::*;
const FIXTURES: &[&str] = &[
    "Orcrist, Goblin-cleaver",
    "Bear Cub",
    "Barkform Harvester",
    "Morcant's Eyes",
    "Goldvein Pick",
    "Doubling Season",
    "Iron Hills Blacksmith",
    "Forest",
    "@token/hob-wolf",
    "@token/b60f6cd3b44b22bd289fe4e2",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        if let Some(r) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: r.target,
                    amount: r.minimum,
                },
            );
        } else {
            choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
    }
    panic!("Orcrist budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn type_choice(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::CreatureTypeChoice { .. })
    });
}
fn typ(name: &str) -> crate::cards::CreatureType {
    crate::cards::CreatureType::from_name(name).unwrap()
}
fn choose_type(g: &mut Game, name: &str) {
    act(
        g,
        Action::ChooseCreatureType {
            creature_type: typ(name),
        },
    );
}
fn treasures(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Treasure"))
            .then_some(id as CardId)
        })
        .collect()
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Orcrist, Goblin-cleaver", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    cast(g, id, None);
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(g.object_view(id).is_legendary());
    id
}
fn equip(g: &mut Game, p: Player, e: CardId, b: CardId) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    act(
        g,
        Action::Activate {
            card: e,
            ability: 0,
            target: Some(Target::Card(b)),
            payment: [0, 0, 0, 0, 0, 3],
            counter_payment: Counters::default(),
        },
    );
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(b)));
}
fn combat(g: &mut Game, b: CardId, blocker: Option<CardId>) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[b as usize].summoning_sick = false;
    act(g, Action::Attack { card: b });
    act(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
    if let Some(blocker) = blocker {
        act(
            g,
            Action::Block {
                blocker,
                attacker: b,
            },
        );
    }
    act(g, Action::FinishBlockers);
}
fn root(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let e = paid(&mut g, p);
    equip(&mut g, p, e, bear);
    combat(&mut g, bear, None);
    type_choice(&mut g);
    (g, e, bear)
}
#[test]
fn orcrist_paid_equip_legality_boost_trample_actual_combat_treasure_recipe_and_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let e = paid(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        g.state.active = p.other();
        g.state.priority = p;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.active = p;
        reject(
            &mut g,
            Action::Activate {
                card: e,
                ability: 0,
                target: Some(Target::Card(blocker)),
                payment: [0, 0, 0, 0, 0, 3],
                counter_payment: Counters::default(),
            },
        );
        equip(&mut g, p, e, b);
        assert_eq!(stats(&g, b), (4, 4));
        assert!(g.keywords(b).contains(Keywords::TRAMPLE));
        combat(&mut g, b, Some(blocker));
        type_choice(&mut g);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        choose_type(&mut g, "Bear");
        finish(&mut g);
        let ts = treasures(&g, p);
        assert_eq!(ts.len(), 1);
        let t = ts[0];
        let view = g.object_view(t);
        assert!(!view.is_creature());
        assert_eq!(view.kinds, 16);
        assert!(!view.tapped);
        assert_eq!(view.colors, 0);
        g.state.players[p.index()].mana = [0; 6];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production:[1,0,0,0,0,0],..}if *card==t),
        );
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        assert!(treasures(&g, p).is_empty());
    }
}
#[test]
fn orcrist_every_valid_type_absent_choice_changeling_tokens_and_noncreature_exclusion() {
    let p = Player::P0;
    let (mut g, _, _) = root(p);
    fixture(&mut g, p, "Barkform Harvester", Zone::Battlefield);
    fixture(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
    fixture(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
    fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    fixture(&mut g, p, "Bear Cub", Zone::Graveyard);
    let offered = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| {
            if let Action::ChooseCreatureType { creature_type } = a {
                Some(creature_type.name())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(offered, crate::CREATURE_TYPES);
    for name in crate::CREATURE_TYPES {
        let mut child = g.clone();
        choose_type(&mut child, name);
        finish(&mut child);
        assert_eq!(
            treasures(&child, p).len(),
            1 + usize::from(*name == "Bear") + usize::from(*name == "Wolf"),
            "{name}"
        );
    }
    let (mut empty, _, _) = root(p);
    choose_type(&mut empty, "Aetherborn");
    finish(&mut empty);
    assert!(treasures(&empty, p).is_empty());
    for invalid in ["Equipment", "Treasure", "Land", "", "NotAType"] {
        assert!(crate::cards::CreatureType::from_name(invalid).is_none());
        let bad = serde_json::json!({"ChooseCreatureType":{"creature_type":invalid}});
        assert!(serde_json::from_value::<Action>(bad).is_err());
    }
}
#[test]
fn orcrist_count_is_current_controller_and_current_type_and_trigger_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let (mut g, e, b) = root(p);
        let foreign = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(foreign), p, None);
        g.refresh_controllers();
        let original = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(original), p.other(), None);
        g.refresh_controllers();
        let timestamp = g.next_role_timestamp();
        let recipient = g.object_ref(b);
        g.state.roles.get_or_insert_default().creature_types.push(
            subtypes::CreatureTypesEffectView {
                recipient,
                source: recipient,
                types: &["Elf"],
                additional: false,
                timestamp,
            },
        );
        g.move_object(e, Zone::Graveyard);
        choose_type(&mut g, "Bear");
        finish(&mut g);
        assert_eq!(treasures(&g, p).len(), 1);
        assert_eq!(g.object_view(foreign).owner, p.other());
        assert_eq!(stats(&g, b), (2, 2));
    }
}
#[test]
fn orcrist_equipment_controller_gets_choice_and_count_even_on_enemy_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let e = paid(&mut g, p);
        equip(&mut g, p, e, b);
        let elf = fixture(&mut g, p.other(), "Barkform Harvester", Zone::Battlefield);
        g.record_control(g.object_ref(e), p.other(), None);
        g.refresh_controllers();
        combat(&mut g, b, None);
        type_choice(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer).creature_type_choice.unwrap().player,
                p.other()
            );
        }
        choose_type(&mut g, "Elf");
        finish(&mut g);
        assert_eq!(treasures(&g, p.other()).len(), 1);
        assert!(treasures(&g, p).is_empty());
        assert_eq!(g.object_view(elf).controller, p.other());
    }
}
#[test]
fn orcrist_layer_suppression_restores_stats_keywords_and_equipment_trigger() {
    let p = Player::P0;
    let mut g = scenario(p);
    let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let e = paid(&mut g, p);
    equip(&mut g, p, e, b);
    let blank = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
    g.blank_and_grant_haste(g.object_ref(e), g.object_ref(blank));
    assert_eq!(stats(&g, b), (2, 2));
    assert!(!g.keywords(b).contains(Keywords::TRAMPLE));
    combat(&mut g, b, None);
    until(&mut g, |g| g.state.step == Step::EndCombat);
    assert!(g.observe(p).creature_type_choice.is_none());
    g.move_object(blank, Zone::Graveyard);
    // This resolved ability-removal effect has no duration and is independent
    // of its historical source. Only a new Equipment incarnation escapes it.
    assert_eq!(stats(&g, b), (2, 2));
    g.move_object(e, Zone::Hand);
    g.move_object(e, Zone::Battlefield);
    g.state.step = Step::Main2;
    equip(&mut g, p, e, b);
    assert_eq!(stats(&g, b), (4, 4));
    assert!(g.keywords(b).contains(Keywords::TRAMPLE));
    g.state.objects[b as usize].tapped = false;
    combat(&mut g, b, None);
    type_choice(&mut g);
    choose_type(&mut g, "Bear");
    finish(&mut g);
    assert_eq!(treasures(&g, p).len(), 1);
}
#[test]
fn orcrist_noncombat_player_damage_and_combat_creature_only_do_not_trigger() {
    let p = Player::P0;
    let mut g = scenario(p);
    let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let e = paid(&mut g, p);
    equip(&mut g, p, e, b);
    let source = g.damage_source(b);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(p.other()),
        amount: 1,
        combat: false,
        preventable: true,
    }]);
    g.prepare_triggers();
    finish(&mut g);
    assert!(g.observe(p).creature_type_choice.is_none());
    let other = fixture(&mut g, p.other(), "Barkform Harvester", Zone::Battlefield);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Card(other),
        amount: 1,
        combat: true,
        preventable: true,
    }]);
    g.prepare_triggers();
    finish(&mut g);
    assert!(g.observe(p).creature_type_choice.is_none());
    assert!(treasures(&g, p).is_empty());
}
#[test]
fn orcrist_double_strike_has_independent_combat_damage_triggers() {
    let p = Player::P0;
    let mut g = scenario(p);
    let b = fixture(&mut g, p, "Iron Hills Blacksmith", Zone::Battlefield);
    let e = paid(&mut g, p);
    equip(&mut g, p, e, b);
    combat(&mut g, b, None);
    type_choice(&mut g);
    choose_type(&mut g, "Dwarf");
    finish(&mut g);
    assert_eq!(treasures(&g, p).len(), 1);
    type_choice(&mut g);
    choose_type(&mut g, "Aetherborn");
    finish(&mut g);
    assert_eq!(treasures(&g, p).len(), 1);
    assert_eq!(g.state.players[p.other().index()].life, 14);
}
#[test]
fn orcrist_replacement_suspension_public_choice_and_owned_restore_does_not_recreate_tokens() {
    let p = Player::P1;
    let (mut g, _, _) = root(p);
    fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
    fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
    contracts::branch_contract(g.clone());
    choose_type(&mut g, "Bear");
    assert!(matches!(g.state.pending, Pending::Replacement { .. }));
    for viewer in [p, p.other()] {
        assert_eq!(
            g.observe(viewer).creature_type_choice.unwrap().selected,
            Some(typ("Bear"))
        );
    }
    contracts::branch_contract(g.clone());
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    drop(g);
    let action = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        .unwrap();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    act(&mut raw, action.clone());
    indexed.apply_index(p, indexed.revision(), index).unwrap();
    for game in [&mut raw, &mut indexed] {
        for _ in 0..8 {
            if !matches!(game.state.pending, Pending::Replacement { .. }) {
                break;
            }
            choose(game, |a| matches!(a, Action::ChooseReplacement { .. }));
        }
        finish(game);
        assert_eq!(treasures(game, p).len(), 4);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    for _ in 0..8 {
        if !matches!(indexed.state.pending, Pending::Replacement { .. }) {
            break;
        }
        choose(&mut indexed, |a| {
            matches!(a, Action::ChooseReplacement { .. })
        });
    }
    finish(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
#[test]
fn orcrist_choice_late_token_generation_failure_rolls_back_selection_and_private_views() {
    let p = Player::P0;
    let (mut g, _, _) = root(p);
    let token = fixture(&mut g, p, "@token/b60f6cd3b44b22bd289fe4e2", Zone::Outside);
    g.state.objects[token as usize].generation = u64::MAX;
    g.state.free_tokens.push(token);
    let before = g.snapshot();
    let revision = g.revision();
    let views = [
        serde_json::to_value(g.observe(p)).unwrap(),
        serde_json::to_value(g.observe(p.other())).unwrap(),
    ];
    assert!(matches!(
        g.apply(
            p,
            Action::ChooseCreatureType {
                creature_type: typ("Bear")
            }
        ),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
    assert_eq!(
        serde_json::to_value(g.observe(p.other())).unwrap(),
        views[1]
    );
    assert_eq!(g.observe(p).creature_type_choice.unwrap().selected, None);
}
