//! Complete top-library, blink, artifact-control and transient-copy consumers.
use super::ability_timeline_tests::{checked, named, next_turn, settle, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn finish(g: &mut Game) {
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..16 {
        if g.state.announcing.is_none()
            || g.state.announcing.as_ref().map(|a| a.item.id) != original
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                        | Action::ChooseSpellCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("payment {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unfinished");
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(s, g.snapshot());
    assert_eq!(r, g.revision());
}
fn end(g: &mut Game) {
    for _ in 0..80 {
        if g.state.step == Step::End {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("endstep");
}
fn copy_activate(g: &mut Game, chrome: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: chrome,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    finish(g);
    settle(g);
}
fn copies(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.object_name(*id as CardId) == name
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn tmt_blink_mikey_paid_hybrid_private_top_and_land_allowance() {
    for color in [1, 4] {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].mana[color] = 2;
        g.state.players[p.index()].mana[5] = 2;
        let mikey = paid(&mut g, p, "Mikey & Don, Party Planners");
        let top = named(&mut g, p, "Forest", Zone::Library);
        assert_eq!(
            g.observe(p).known_library_top[p.index()]
                .as_ref()
                .map(|o| o.object.card),
            Some(top)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        checked(&mut g, Action::PlayLand { card: top });
        let second = named(&mut g, p, "Island", Zone::Library);
        assert!(
            !g.legal_actions()
                .contains(&Action::PlayLand { card: second })
        );
        g.remove_abilities_exact(
            &[g.object_ref(mikey)],
            crate::cards::AnimationDuration::EndOfTurn,
        );
        assert!(g.observe(p).known_library_top[p.index()].is_none());
    }
}
#[test]
fn tmt_blink_mikey_three_subtypes_paid_top_cast_frozen_counter_and_other_origins() {
    for name in ["High-Flying Ace", "Foot Mystic", "Aegis Turtle"] {
        let p = Player::P0;
        let mut g = setup(p);
        let mikey = paid(&mut g, p, "Mikey & Don, Party Planners");
        let top = named(&mut g, p, name, Zone::Library);
        checked(&mut g, Action::AnnounceCast { card: top });
        assert!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .item
                .cast
                .unwrap()
                .origin_permissions
                .library_creature_counter()
        );
        g.move_object(mikey, Zone::Hand);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(top, CounterKind::PlusOne), 1, "{name}");
        let hand = named(&mut g, p, name, Zone::Hand);
        cast(&mut g, hand, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(hand, CounterKind::PlusOne), 0);
    }
}
#[test]
fn tmt_blink_mikey_explicit_vizier_permission_omits_counter_and_stale_provider_rejects() {
    let p = Player::P0;
    let mut g = setup(p);
    let mikey = paid(&mut g, p, "Mikey & Don, Party Planners");
    let vizier = paid(&mut g, p, "Vizier of the Menagerie");
    let top = named(&mut g, p, "Aegis Turtle", Zone::Library);
    let permission = g.object_ref(vizier);
    let action = Action::AnnounceLibrarySpell {
        card: top,
        permission,
        method: CastMethod::Normal,
        attacker: None,
        flash_surcharge: false,
    };
    let before = g.snapshot();
    g.move_object(vizier, Zone::Hand);
    reject(&mut g, action.clone());
    g.restore(&before).unwrap();
    checked(&mut g, action);
    assert!(
        !g.state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .cast
            .unwrap()
            .origin_permissions
            .library_creature_counter()
    );
    g.remove_abilities_exact(
        &[g.object_ref(mikey)],
        crate::cards::AnimationDuration::EndOfTurn,
    );
    finish(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(top, CounterKind::PlusOne), 0);
}
#[test]
fn tmt_blink_mikey_rejects_nonmatching_top_and_wrong_player_and_preserves_normal_timing() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Mikey & Don, Party Planners");
    let top = named(&mut g, p, "Grizzly Bears", Zone::Library);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: top })
    );
    let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Library);
    reject(&mut g, Action::AnnounceCast { card: enemy });
    let own = named(&mut g, p, "Aegis Turtle", Zone::Library);
    g.state.step = Step::Upkeep;
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: own })
    );
}
#[test]
fn tmt_blink_mikey_ward_two_can_counter_opponent_targeting_spell() {
    let p = Player::P0;
    let mut g = setup(p);
    let mikey = paid(&mut g, p, "Mikey & Don, Party Planners");
    checked(&mut g, Action::Pass);
    let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
    cast(&mut g, shock, None, &[Target::Card(mikey)]);
    g.state.players[p.other().index()].mana = [0; 6];
    settle(&mut g);
    assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[mikey as usize].damage, 0);
}
#[test]
fn tmt_blink_don_leo_paid_endstep_two_optional_targets_return_to_owners() {
    let p = Player::P0;
    let mut g = setup(p);
    let don = paid(&mut g, p, "Don & Leo, Problem Solvers");
    assert!(g.keywords(don).contains(Keywords::VIGILANCE));
    let artifact = paid(&mut g, p, "Weather Maker");
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.record_control(g.object_ref(bear), p, None);
    let before = [g.object_ref(artifact), g.object_ref(bear)];
    end(&mut g);
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(&mut g);
    settle(&mut g);
    assert_ne!(g.object_ref(artifact), before[0]);
    assert_ne!(g.object_ref(bear), before[1]);
    assert_eq!(g.state.objects[bear as usize].controller, p.other());
    assert!(g.on_battlefield(artifact));
}
#[test]
fn tmt_blink_don_leo_same_artifact_creature_selected_twice_moves_only_once() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Don & Leo, Problem Solvers");
    let droid = paid(&mut g, p, "Fugitive Droid");
    let old = g.object_ref(droid);
    end(&mut g);
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(droid),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(droid),
        },
    );
    finish(&mut g);
    settle(&mut g);
    assert_eq!(g.object_ref(droid).generation, old.generation + 2);
}
#[test]
fn tmt_blink_don_leo_decline_both_and_token_does_not_return() {
    let p = Player::P0;
    let mut g = setup(p);
    let don = paid(&mut g, p, "Don & Leo, Problem Solvers");
    let old = g.object_ref(don);
    end(&mut g);
    settle(&mut g);
    assert_eq!(g.object_ref(don), old);
    next_turn(&mut g);
    next_turn(&mut g);
    while g.state.step != Step::Main1 {
        checked(&mut g, Action::Pass);
    }
    let food = named(
        &mut g,
        p,
        "@token/283fd83a9aec83a51b736926",
        Zone::Battlefield,
    );
    end(&mut g);
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(food),
        },
    );
    finish(&mut g);
    settle(&mut g);
    assert!(!g.on_battlefield(food));
}
#[test]
fn tmt_blink_broadcast_paid_nontargeted_mass_control_stun_untap_haste_and_expiry() {
    let p = Player::P0;
    let mut g = setup(p);
    let enemy = paid(&mut g, p, "Weather Maker");
    g.record_control(g.object_ref(enemy), p.other(), None);
    g.state.objects[enemy as usize].tapped = true;
    g.add_counter(enemy, CounterKind::Stun, 1);
    let droid = named(&mut g, p.other(), "Fugitive Droid", Zone::Battlefield);
    g.state.objects[droid as usize].tapped = true;
    let own = named(&mut g, p, "Hedron Archive", Zone::Battlefield);
    g.state.objects[own as usize].tapped = true;
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    paid(&mut g, p, "Broadcast Takeover");
    assert_eq!(g.state.objects[enemy as usize].controller, p);
    assert_eq!(g.counter_count(enemy, CounterKind::Stun), 0);
    assert!(g.state.objects[enemy as usize].tapped);
    assert!(!g.state.objects[droid as usize].tapped);
    assert!(g.keywords(droid).contains(Keywords::HASTE));
    assert!(g.state.objects[own as usize].tapped);
    assert_eq!(g.state.objects[bear as usize].controller, p.other());
    next_turn(&mut g);
    assert_eq!(g.state.objects[enemy as usize].controller, p.other());
    assert_eq!(g.state.objects[droid as usize].controller, p.other());
    assert!(!g.keywords(droid).contains(Keywords::HASTE));
}
#[test]
fn tmt_blink_fugitive_paid_self_entry_unblockable_history_resets_and_tracks_tokens() {
    let p = Player::P0;
    let mut g = setup(p);
    let droid = paid(&mut g, p, "Fugitive Droid");
    assert!(g.keywords(droid).contains(Keywords::CANT_BE_BLOCKED));
    next_turn(&mut g);
    assert!(!g.keywords(droid).contains(Keywords::CANT_BE_BLOCKED));
    g.create_tokens(
        p,
        Card::from_name("@token/283fd83a9aec83a51b736926").unwrap(),
        1,
        false,
    );
    settle(&mut g);
    assert!(g.keywords(droid).contains(Keywords::CANT_BE_BLOCKED));
}
#[test]
fn tmt_blink_fugitive_paid_counter_sacrifices_source_and_counters_spell_targeting_other_owned_permanent()
 {
    let p = Player::P0;
    let mut g = setup(p);
    let droid = paid(&mut g, p, "Fugitive Droid");
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    checked(&mut g, Action::Pass);
    let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
    cast(&mut g, shock, None, &[Target::Card(bear)]);
    checked(&mut g, Action::Pass);
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: droid,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    finish(&mut g);
    assert_eq!(g.state.objects[droid as usize].zone, Zone::Graveyard);
    settle(&mut g);
    assert!(g.on_battlefield(bear));
    assert_eq!(g.state.objects[bear as usize].damage, 0);
}
#[test]
fn tmt_blink_fugitive_target_predicate_ignores_departure_reentry_and_live_control_changes() {
    let p = Player::P0;
    let mut g = setup(p);
    let droid = paid(&mut g, p, "Fugitive Droid");
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let shock = named(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, None, &[Target::Card(bear)]);
    let item = g.state.stack.last().unwrap().clone();
    assert!(g.spell_targets_controlled_artifact_or_creature(&item, p));
    g.record_control(g.object_ref(bear), p.other(), None);
    assert!(!g.spell_targets_controlled_artifact_or_creature(&item, p));
    g.record_control(g.object_ref(bear), p, None);
    g.move_object(bear, Zone::Hand);
    assert!(!g.spell_targets_controlled_artifact_or_creature(&item, p));
    g.move_object(bear, Zone::Battlefield);
    assert!(!g.spell_targets_controlled_artifact_or_creature(&item, p));
    let _ = droid;
}
#[test]
fn tmt_blink_chrome_paid_lord_other_owned_artifact_creatures_and_target_restrictions() {
    let p = Player::P0;
    let mut g = setup(p);
    let chrome = paid(&mut g, p, "Chrome Dome");
    let own = paid(&mut g, p, "Fugitive Droid");
    let enemy = named(&mut g, p.other(), "Fugitive Droid", Zone::Battlefield);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(g.stats(chrome).power, 1);
    assert_eq!(g.stats(own).power, 2);
    assert_eq!(g.stats(enemy).power, 1);
    assert_eq!(g.stats(bear).power, 2);
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: chrome,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for id in [chrome, enemy, bear] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    finish(&mut g);
    settle(&mut g);
    assert_eq!(copies(&g, p, "Fugitive Droid").len(), 1);
}
#[test]
fn tmt_blink_chrome_noncreature_copy_has_noncopyable_haste_and_source_independent_delayed_sacrifice()
 {
    let p = Player::P0;
    let mut g = setup(p);
    let chrome = paid(&mut g, p, "Chrome Dome");
    let weather = paid(&mut g, p, "Weather Maker");
    let ordinary_bytes = g.state_bytes();
    copy_activate(&mut g, chrome, weather);
    eprintln!(
        "TMT Chrome transient-copy initialized state delta: {} bytes",
        g.state_bytes() - ordinary_bytes
    );
    let token = copies(&g, p, "Weather Maker")[0];
    assert!(g.keywords(token).contains(Keywords::HASTE));
    let recipe = g.copiable_recipe(g.object_ref(token));
    assert!(!recipe.exceptions.haste);
    assert_eq!(recipe.exceptions.sacrifice_abilities, 0);
    let before = g.state_bytes();
    g.move_object(chrome, Zone::Graveyard);
    g.remove_abilities_exact(
        &[g.object_ref(token)],
        crate::cards::AnimationDuration::EndOfTurn,
    );
    end(&mut g);
    settle(&mut g);
    assert!(!g.on_battlefield(token));
    assert!(g.state_bytes() < before + 4096);
}
#[test]
fn tmt_blink_chrome_copied_copy_does_not_inherit_delayed_sacrifice_or_gained_haste() {
    let p = Player::P0;
    let mut g = setup(p);
    let chrome = paid(&mut g, p, "Chrome Dome");
    let droid = paid(&mut g, p, "Fugitive Droid");
    copy_activate(&mut g, chrome, droid);
    let first = copies(&g, p, "Fugitive Droid")[0];
    let recipe = g.copiable_recipe(g.object_ref(first));
    g.stage_copy_creation(p, recipe, 1);
    settle(&mut g);
    let second = *copies(&g, p, "Fugitive Droid")
        .iter()
        .find(|&&id| id != first)
        .unwrap();
    assert!(!g.keywords(second).contains(Keywords::HASTE));
    end(&mut g);
    settle(&mut g);
    assert!(!g.on_battlefield(first));
    assert!(g.on_battlefield(second));
}
#[test]
fn tmt_blink_mikey_counter_promise_survives_control_change_and_uses_entry_doubling() {
    let p = Player::P0;
    let mut g = setup(p);
    let mikey = paid(&mut g, p, "Mikey & Don, Party Planners");
    named(&mut g, p, "Doubling Season", Zone::Battlefield);
    let top = named(&mut g, p, "Aegis Turtle", Zone::Library);
    let permission = g.object_ref(mikey);
    checked(
        &mut g,
        Action::AnnounceLibrarySpell {
            card: top,
            permission,
            method: CastMethod::Normal,
            attacker: None,
            flash_surcharge: false,
        },
    );
    g.record_control(g.object_ref(mikey), p.other(), None);
    finish(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(top, CounterKind::PlusOne), 2);
}
#[test]
fn tmt_blink_fugitive_sacrificing_only_qualifying_target_makes_counter_target_illegal() {
    let p = Player::P0;
    let mut g = setup(p);
    let droid = paid(&mut g, p, "Fugitive Droid");
    checked(&mut g, Action::Pass);
    let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
    cast(&mut g, shock, None, &[Target::Card(droid)]);
    checked(&mut g, Action::Pass);
    checked(
        &mut g,
        Action::AnnounceActivate {
            card: droid,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    finish(&mut g);
    assert_eq!(g.state.objects[droid as usize].zone, Zone::Graveyard);
    assert!(
        !g.spell_targets_controlled_artifact_or_creature(
            g.state
                .stack
                .iter()
                .find(|s| s.card == shock && s.kind == StackKind::Spell)
                .unwrap(),
            p
        )
    );
    settle(&mut g);
    assert_eq!(g.state.objects[shock as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_blink_fugitive_ignores_nonartifact_noncreature_targets_and_uses_current_types() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Fugitive Droid");
    let land = named(&mut g, p, "Forest", Zone::Battlefield);
    let mut spell = g.cast_preview(p, land, Zone::Hand, g.object_ref(land), 0, [0; 6]);
    spell.targets = vec![g.bind_target(Target::Card(land))];
    assert!(!g.spell_targets_controlled_artifact_or_creature(&spell, p));
    let item = g.activation_item(p, land, 0);
    g.animate_source(
        &item,
        crate::cards::AnimationSpec {
            artifact: false,
            base: Some((1, 1)),
            creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
            keywords: Keywords::NONE,
            duration: crate::cards::AnimationDuration::EndOfTurn,
        },
    );
    assert!(g.spell_targets_controlled_artifact_or_creature(&spell, p));
}
#[test]
fn tmt_blink_chrome_creation_doubled_tokens_both_get_nonintrinsic_haste_and_delayed_sacrifice() {
    let p = Player::P0;
    let mut g = setup(p);
    let chrome = paid(&mut g, p, "Chrome Dome");
    let weather = paid(&mut g, p, "Weather Maker");
    named(&mut g, p, "Doubling Season", Zone::Battlefield);
    copy_activate(&mut g, chrome, weather);
    let tokens = copies(&g, p, "Weather Maker");
    assert_eq!(tokens.len(), 2);
    for &id in &tokens {
        assert!(g.keywords(id).contains(Keywords::HASTE));
        assert_eq!(
            g.copiable_recipe(g.object_ref(id)).exceptions,
            CopyExceptions::NONE
        );
    }
    end(&mut g);
    settle(&mut g);
    assert!(tokens.into_iter().all(|id| !g.on_battlefield(id)));
}
#[test]
fn tmt_blink_chrome_stolen_token_not_sacrificed_by_former_controller_and_no_repeat() {
    let p = Player::P0;
    let mut g = setup(p);
    let chrome = paid(&mut g, p, "Chrome Dome");
    let weather = paid(&mut g, p, "Weather Maker");
    copy_activate(&mut g, chrome, weather);
    let token = copies(&g, p, "Weather Maker")[0];
    g.record_control(g.object_ref(token), p.other(), None);
    end(&mut g);
    settle(&mut g);
    assert!(g.on_battlefield(token));
    g.record_control(g.object_ref(token), p, None);
    next_turn(&mut g);
    end(&mut g);
    settle(&mut g);
    assert!(g.on_battlefield(token));
    assert!(g.keywords(token).contains(Keywords::HASTE));
}

pub(super) fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
#[test]
fn tmt_blink_don_leo_partly_illegal_group_still_blinks_the_live_creature() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Don & Leo, Problem Solvers");
    let weather = paid(&mut g, p, "Weather Maker");
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let before = g.object_ref(bear);
    end(&mut g);
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(weather),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    finish(&mut g);
    g.move_object(weather, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.objects[weather as usize].zone, Zone::Hand);
    assert_ne!(g.object_ref(bear), before);
}
#[test]
fn tmt_blink_don_leo_returns_linked_exile_before_returning_the_blinking_source() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Don & Leo, Problem Solvers");
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let rip = named(&mut g, p, "Glass Casket", Zone::Hand);
    cast(&mut g, rip, None, &[]);
    super::ability_timeline_tests::settle_on(&mut g, Some(Target::Card(bear)));
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Exile);
    end(&mut g);
    g.prepare_triggers();
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(rip),
        },
    );
    finish(&mut g);
    checked(&mut g, Action::Pass);
    checked(&mut g, Action::Pass);
    assert!(g.on_battlefield(bear));
    assert!(g.on_battlefield(rip));
    assert!(g.current_kinds(rip) & 16 != 0);
    settle(&mut g);
}
#[test]
fn tmt_blink_library_permission_layout_and_query_cost_are_measured_without_state_growth() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<CastInfo>(), 40);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Mikey & Don, Party Planners");
    let top = named(&mut g, p, "Aegis Turtle", Zone::Library);
    let before = g.snapshot();
    let bytes = g.state_bytes();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: top })
        );
    }
    eprintln!(
        "TMT library permission: 100 legal-action queries in {:?}; {} initialized bytes",
        start.elapsed(),
        bytes
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
}

#[test]
fn tmt_blink_mikey_sneak_from_library_can_return_its_permission_source_and_keeps_counter_promise() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mikey = paid(&mut g, p, "Mikey & Don, Party Planners");
        next_turn(&mut g);
        next_turn(&mut g);
        while g.state.step != Step::Main1 {
            checked(&mut g, Action::Pass);
        }
        for state in &mut g.state.players {
            state.mana = [30; 6];
        }
        let top = named(&mut g, p, "Foot Ninjas", Zone::Library);
        super::tmt_sneak_tests::unblocked(&mut g, mikey);
        let permission = g.object_ref(mikey);
        checked(
            &mut g,
            Action::AnnounceLibrarySpell {
                card: top,
                permission,
                method: CastMethod::Sneak,
                attacker: Some(permission),
                flash_surcharge: false,
            },
        );
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        finish(&mut g);
        assert_eq!(g.state.objects[mikey as usize].zone, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(top, CounterKind::PlusOne), 1);
        let view = g.object_view(top);
        assert_eq!(view.power, 6);
        assert!(view.tapped && view.attacking);
    }
}

#[test]
fn tmt_blink_smoke_policy_scores_visible_library_cast_without_reading_hidden_next() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Mikey & Don, Party Planners");
        let hidden = named(&mut g, p, "Shock", Zone::Library);
        let top = named(&mut g, p, "High-Flying Ace", Zone::Library);
        let view = g.observe(p);
        assert!(view.objects.iter().all(|o| o.id != top && o.id != hidden));
        assert_eq!(
            view.known_library_top[p.index()].unwrap().object,
            g.object_ref(top)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        let cast = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Cast{card,..}if *card==top))
            .expect("public legal top-library cast");
        let choices = vec![Action::Pass, cast.clone()];
        let before = g.snapshot();
        assert_eq!(crate::policy::smoke_index(&g, &choices), 1);
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(&view).unwrap()
        );
        let mut sibling = g.clone();
        sibling.state.objects[hidden as usize].card = Card::from_name("Unsummon").unwrap();
        assert_eq!(
            serde_json::to_value(sibling.observe(p)).unwrap(),
            serde_json::to_value(&view).unwrap()
        );
        assert_eq!(crate::policy::smoke_index(&sibling, &choices), 1);
        checked(&mut g, cast);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().origin,
            Zone::Library
        );
        settle(&mut g);
        assert!(g.on_battlefield(top));
        assert_eq!(g.counter_count(top, CounterKind::PlusOne), 1);
    }
}
