use super::tests::{act, pass_pair, scenario, spawn};
use super::*;

fn legend(g: &mut Game, p: Player, name: &str) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), Zone::Battlefield)
}
fn yargle(g: &mut Game, p: Player) -> CardId {
    legend(g, p, "Yargle, Glutton of Urborg")
}
fn keep(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::KeepLegend { object });
}
fn rejected(g: &mut Game, p: Player, action: Action) {
    let before = g.snapshot();
    let rev = g.revision;
    assert!(g.apply(p, action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, rev);
}

#[test]
fn legend_rule_uses_controller_and_name_and_checks_exact_objects() {
    let mut g = scenario();
    let a = yargle(&mut g, Player::P0);
    yargle(&mut g, Player::P1);
    let different = legend(&mut g, Player::P0, "Yargle, Goliath of Otaria");
    g.state_based_actions();
    assert_eq!(g.state.pending, Pending::Priority);
    let b = yargle(&mut g, Player::P0);
    let c = yargle(&mut g, Player::P0);
    g.state_based_actions();
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.observe(Player::P1).decision, "legend_rule");
    assert_eq!(
        g.legal_actions()
            .iter()
            .filter(|a| matches!(a, Action::KeepLegend { .. }))
            .count(),
        3
    );
    rejected(&mut g, Player::P0, Action::Pass);
    rejected(
        &mut g,
        Player::P0,
        Action::KeepLegend {
            object: ObjectRef {
                card: a,
                generation: 1,
            },
        },
    );
    let object = g.object_ref(a);
    rejected(&mut g, Player::P1, Action::KeepLegend { object });
    let object = g.object_ref(different);
    rejected(&mut g, Player::P0, Action::KeepLegend { object });
    keep(&mut g, b);
    assert_eq!(g.object_view(b).zone, Zone::Battlefield);
    for id in [a, c] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.state.pending, Pending::Priority);
    assert!(g.observe(Player::P0).legend_choices.is_empty());
}

#[test]
fn apnap_choices_are_public_frozen_and_replayable_in_any_own_group_order() {
    let mut g = scenario();
    g.state.active = Player::P1;
    let a = yargle(&mut g, Player::P1);
    let b = yargle(&mut g, Player::P1);
    let c = legend(&mut g, Player::P1, "Zetalpa, Primal Dawn");
    let d = legend(&mut g, Player::P1, "Zetalpa, Primal Dawn");
    let e = yargle(&mut g, Player::P0);
    let f = yargle(&mut g, Player::P0);
    g.state.objects[f as usize].counters.plus_one = 2;
    g.state.objects[f as usize].counters.minus_one = 1;
    g.state_based_actions();
    assert_eq!(g.actor(), Some(Player::P1));
    // The active player can choose their second name group first.
    keep(&mut g, d);
    assert_eq!(g.actor(), Some(Player::P1));
    let saved = g.snapshot();
    let bytes = g.state_bytes();
    assert!(bytes > std::mem::size_of::<State>());
    let mut fork = g.clone();
    for branch in [&mut g, &mut fork] {
        keep(branch, b);
        assert_eq!(branch.actor(), Some(Player::P0));
        for viewer in [Player::P0, Player::P1] {
            let groups = branch.observe(viewer).legend_choices;
            assert_eq!(groups.len(), 3);
            assert_eq!(groups.iter().filter(|x| x.keep.is_some()).count(), 2);
        }
        assert!(
            [a, b, c, d, e, f]
                .iter()
                .all(|&id| branch.object_view(id).zone == Zone::Battlefield)
        );
        assert_eq!(branch.object_view(f).counters.plus_one, 2);
        keep(branch, f);
    }
    assert_eq!(g.snapshot(), fork.snapshot());
    let result = g.snapshot();
    g.restore(&saved).unwrap();
    keep(&mut g, b);
    keep(&mut g, f);
    assert_eq!(g.snapshot(), result);
    assert_eq!(g.object_view(f).counters.plus_one, 1);
    assert_eq!(g.object_view(f).counters.minus_one, 0);
    for id in [a, c, e] {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(g.state.pending, Pending::Priority);
}

#[test]
fn a_dying_legend_can_be_kept_and_indestructible_does_not_prevent_legend_rule() {
    for zero_toughness in [false, true] {
        let mut g = scenario();
        let a = yargle(&mut g, Player::P0);
        let b = yargle(&mut g, Player::P0);
        if zero_toughness {
            g.state.objects[a as usize].counters.minus_one = 3;
        } else {
            g.state.objects[a as usize].damage = 3;
        }
        g.state_based_actions();
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        keep(&mut g, a);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
    let mut g = scenario();
    let a = legend(&mut g, Player::P0, "Zetalpa, Primal Dawn");
    let b = legend(&mut g, Player::P0, "Zetalpa, Primal Dawn");
    g.state.objects[a as usize].damage = 100;
    g.state.objects[a as usize].deathtouch_since_sba = true;
    g.state_based_actions();
    keep(&mut g, a);
    assert_eq!(g.object_view(a).zone, Zone::Battlefield);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert!(!g.state.objects[a as usize].deathtouch_since_sba);
    g.state.objects[a as usize].counters.minus_one = 100;
    g.state_based_actions();
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
}

#[test]
fn other_deaths_and_losses_wait_for_the_same_frozen_batch() {
    for lost in [[false, false], [true, false], [true, true]] {
        let mut g = scenario();
        let a = yargle(&mut g, Player::P0);
        let b = yargle(&mut g, Player::P0);
        let dying = legend(&mut g, Player::P0, "Agents of HYDRA");
        g.state.objects[dying as usize].damage = 100;
        for (p, lost) in lost.into_iter().enumerate() {
            if lost {
                g.state.players[p].life = 0;
            }
        }
        g.state_based_actions();
        assert!(g.outcome().is_none());
        assert_eq!(g.object_view(dying).zone, Zone::Battlefield);
        assert!(g.state.pending_triggers.is_empty());
        keep(&mut g, a);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(g.object_view(dying).zone, Zone::Graveyard);
        assert_eq!(
            g.outcome(),
            match lost {
                [false, false] => None,
                [true, false] => Some(Outcome::Winner(Player::P1)),
                _ => Some(Outcome::Draw),
            }
        );
        // The death trigger is captured even though its source leaves in this batch.
        assert_eq!(
            g.state.pending_triggers.len() + g.state.ordering_triggers.len() + g.state.stack.len(),
            1
        );
    }
}

#[test]
fn linked_returns_create_another_legend_batch_before_priority_or_triggers() {
    let mut g = scenario();
    let keep1 = yargle(&mut g, Player::P0);
    let source = yargle(&mut g, Player::P0);
    let old = legend(&mut g, Player::P1, "Tatyova, Benthic Druid");
    let held = spawn(
        &mut g,
        Player::P1,
        Card::from_name("Tatyova, Benthic Druid").unwrap(),
        Zone::Exile,
    );
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(held),
        return_zone: Zone::Battlefield,
    });
    g.state_based_actions();
    keep(&mut g, keep1);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(held).zone, Zone::Battlefield);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.observe(Player::P1).decision, "legend_rule");
    let saved = g.snapshot();
    keep(&mut g, held);
    let expected = g.snapshot();
    g.restore(&saved).unwrap();
    keep(&mut g, held);
    assert_eq!(g.snapshot(), expected);
    assert_eq!(g.object_view(old).zone, Zone::Graveyard);
    assert_eq!(g.state.pending, Pending::Priority);
}

#[test]
fn cleanup_preserves_legend_choice_and_resumes_extra_priority_cycle() {
    let mut g = scenario();
    g.state.step = Step::Cleanup;
    let a = yargle(&mut g, Player::P0);
    let b = yargle(&mut g, Player::P0);
    g.state.objects[a as usize].counters.minus_one = 3;
    g.state.objects[a as usize].growth.toughness = 1;
    let turn = g.state.turn;
    g.finish_cleanup();
    assert_eq!(g.observe(Player::P0).decision, "legend_rule");
    assert_eq!(g.state.turn, turn);
    keep(&mut g, b);
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    assert_eq!(g.state.pending, Pending::Priority);
    pass_pair(&mut g);
    assert_eq!(g.state.turn, turn + 1);
}

#[test]
fn queued_landfall_triggers_survive_departing_legend_and_wait_until_choices_end() {
    let mut g = scenario();
    let a = legend(&mut g, Player::P0, "Tatyova, Benthic Druid");
    let b = legend(&mut g, Player::P0, "Tatyova, Benthic Druid");
    // A single resolving effect may create the duplicate and a land before SBAs.
    let land = spawn(&mut g, Player::P0, Card::Forest, Zone::Hand);
    g.move_object(land, Zone::Battlefield);
    assert_eq!(g.state.pending_triggers.len(), 2);
    g.state_based_actions();
    g.prepare_triggers();
    assert!(g.state.stack.is_empty());
    assert!(g.state.ordering_triggers.is_empty());
    let saved = g.snapshot();
    keep(&mut g, a);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    while let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        act(&mut g, action);
    }
    let library = g.state.players[0].library.len();
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.state.players[0].library.len(), library - 2);
    let expected = g.snapshot();
    g.restore(&saved).unwrap();
    keep(&mut g, a);
    while let Some(action) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        act(&mut g, action);
    }
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.snapshot(), expected);
}

#[test]
fn tokens_cease_only_after_all_choices_and_counter_pairs_preserve_departure_lki() {
    let mut g = scenario();
    let a = yargle(&mut g, Player::P0);
    yargle(&mut g, Player::P0);
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == "Dog Token"))
        .unwrap();
    let id = spawn(&mut g, Player::P0, token, Zone::Graveyard);
    let dying = legend(&mut g, Player::P0, "Agents of HYDRA");
    g.state.objects[dying as usize].damage = 100;
    g.state.objects[dying as usize].counters.plus_one = 2;
    g.state.objects[dying as usize].counters.minus_one = 1;
    g.state_based_actions();
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    assert!(!g.state.free_tokens.contains(&id));
    keep(&mut g, a);
    assert_eq!(g.state.objects[id as usize].zone, Zone::Outside);
    assert!(g.state.free_tokens.contains(&id));
    assert_eq!(g.object_view(dying).counters, Counters::default());
    // The source's death trigger remains executable after counter cancellation/movement.
    pass_pair(&mut g);
    assert!(g.state.objects.iter().any(|o| {
        o.zone == Zone::Battlefield
            && o.card
                .token_info()
                .is_some_and(|i| i.name == "Villain Token")
    }));
}
