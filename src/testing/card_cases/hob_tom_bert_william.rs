//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid sacrifice quantities and exact-incarnation artifact resurrection.
use super::*;
const NAME: &str = "Tom, Bert, and William";
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("Tom budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn tom(g: &mut Game, p: Player) -> CardId {
    put(g, p, NAME, Zone::Battlefield)
}
fn activate(g: &mut Game, s: CardId, fuel: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: s,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(fuel);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}

fn artifact(g: &Game, s: CardId, p: Player) {
    let o = g.object_view(s);
    assert_eq!((o.zone, o.controller, o.kinds), (Zone::Battlefield, p, 16));
    assert!(!o.has_subtype("Troll"));
    assert!(o.card.is_legendary());
    assert_eq!(o.colors, 20);
    assert_eq!(o.card.activations().len(), 1);
    assert_eq!(
        g.copiable_recipe(o.reference).kinds(),
        1,
        "type effect is not copiable"
    );
    let row = g
        .observe(p)
        .animations
        .into_iter()
        .find(|r| r.object == o.reference)
        .unwrap();
    assert!(row.artifact_noncreature);
    assert_eq!(row.expires_turn, None);
    assert_eq!(
        serde_json::to_value(row).unwrap()["artifact_noncreature"],
        true
    );
}
#[test]
fn hob_tom_paid_five_five_real_murder_returns_under_death_controller() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let s = put(&mut g, p, NAME, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 3];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (5, 5));
        assert!(g.has_subtype(s, "Troll"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.change_controller(g.object_ref(s), p.other());
        let old = g.object_ref(s);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, murder, Some(Target::Card(s)));
        finish(&mut g);
        artifact(&g, s, p.other());
        assert_eq!(g.object_view(s).owner, p);
        assert_ne!(g.object_ref(s), old);
        assert_eq!(g.object_view(murder).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_tom_paid_sacrifice_current_power_tokens_and_discard_are_controller_relative() {
    {
        let p = Player::P0;
        for power in [0, 2, 4, -1] {
            let mut g = scenario(p);
            let s = tom(&mut g, p);
            let fuel = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.change_controller(g.object_ref(fuel), p);
            g.state.objects[fuel as usize].growth.power = power - 2;
            let h = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            activate(&mut g, s, fuel);
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.stack.last().unwrap().sacrificed_power.as_deref(),
                Some(&power)
            );
            g.move_object(s, Zone::Exile);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            act(&mut g, Action::Discard { card: h });
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                power.max(0) as u64
            );
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
            assert_eq!(g.object_view(h).zone, Zone::Graveyard);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Hand && o.owner == p)
                    .count(),
                power.max(0) as usize
            );
        }
    }
}

#[test]
fn hob_tom_token_copy_dies_without_return_and_ability_loss_suppresses_trigger() {
    let p = Player::P0;
    for token in [true, false] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        if token {
            let source = g.object_view(s);
            let recipe = g.copiable_recipe(source.reference);
            g.move_object(s, Zone::Outside);
            g.stage_copy_creation(p, recipe, 1);
            let t = (0..g.state.objects.len())
                .find(|&i| g.state.objects[i].is_token())
                .unwrap() as CardId;
            g.move_object(t, Zone::Graveyard);
            finish(&mut g);
            assert_ne!(g.object_view(t).zone, Zone::Battlefield);
        } else {
            let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
            assert_eq!(stats(&g, s), (1, 1));
            g.move_object(s, Zone::Graveyard);
            finish(&mut g);
            assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn hob_tom_staff_doubles_ordinary_death_trigger_but_only_first_exact_return_succeeds() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = tom(&mut g, p);
    let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
    g.state.objects[staff as usize].attached_to = Some(g.object_ref(s));
    assert_eq!(g.object_view(s).additional_triggers, 1);
    g.move_object(s, Zone::Graveyard);
    g.prepare_triggers();
    assert_eq!(
        g.state.pending_triggers.len() + g.state.ordering_triggers.len() + g.state.stack.len(),
        2
    );
    finish(&mut g);
    artifact(&g, s, p);
    assert_eq!(g.object_ref(s).generation, 2);
    assert_eq!(g.observe(p).animations.len(), 1);
}
