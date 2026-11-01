//! Focused printed outcomes; shared engine invariants live in core tests.
//! Private persistent exile rights and ordinary cast-trigger outcomes for both faces.
use super::*;
const GANDALF: &str = "Gandalf, Goblins' Bane // Flameshape";
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishStackTargets
            )
        });
    }
    panic!("Gandalf budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
    });
}
fn wizard(g: &mut Game, p: Player) -> CardId {
    put(g, p, GANDALF, Zone::Battlefield)
}
fn top(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.clone();
    for id in old {
        g.move_object(id, Zone::Graveyard);
    }
    names.iter().map(|n| put(g, p, n, Zone::Library)).collect()
}
fn expose(g: &mut Game, p: Player, names: &[&str]) -> (CardId, Vec<CardId>) {
    let cards = top(g, p, names);
    let id = put(g, p, GANDALF, Zone::Hand);
    g.state.priority = p;
    g.state.active = p;
    g.state.step = Step::Main1;
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    act(g, Action::AnnounceAdventure { card: id });
    pay(g);
    finish(g);
    (id, cards)
}

fn own(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
}
fn shock(g: &mut Game, p: Player) -> CardId {
    g.state.priority = p;
    let id = put(g, p, "Shock", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(g, id, Some(Target::Player(p.other())));
    id
}
#[test]
fn hob_gandalf_paid_body_and_noncreature_cast_trigger_is_temporary() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let s = put(&mut g, p, GANDALF, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (2, 3));
        assert!(g.has_subtype(s, "Avatar") && g.has_subtype(s, "Wizard"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        cast(&mut g, pick, None);
        let spell = g
            .state
            .stack
            .iter()
            .find(|v| v.source.is_some_and(|identity| identity.id == pick))
            .unwrap()
            .id;
        g.counter_stack_item(spell, false);
        finish(&mut g);
        assert_eq!(stats(&g, s), (3, 4));
        assert_eq!(g.state.players[p.other().index()].life, 19);
        let turn = g.state.turn;
        for _ in 0..100 {
            if g.state.turn != turn {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert_ne!(g.state.turn, turn);
        assert_eq!(stats(&g, s), (2, 3));
    }
}
#[test]
fn hob_gandalf_trigger_ignores_own_creature_and_opponent_noncreature() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        finish(&mut g);
        own(&mut g, p.other());
        let pick = put(&mut g, p.other(), "Goldvein Pick", Zone::Hand);
        cast(&mut g, pick, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (2, 3));
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

#[test]
fn hob_gandalf_staff_doubles_whole_trigger_and_spell_copies_are_not_casts() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(s)));
        shock(&mut g, p);
        finish(&mut g);
        assert_eq!(stats(&g, s), (6, 7));
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let original = shock(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(original),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(stats(&g, s), (4, 5));
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn hob_gandalf_ability_suppression_stops_new_packets_and_control_uses_current_caster() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let s = wizard(&mut g, p);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
        shock(&mut g, p);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.move_object(aura, Zone::Graveyard);
        g.change_controller(g.object_ref(s), p.other());
        own(&mut g, p.other());
        shock(&mut g, p.other());
        finish(&mut g);
        assert_eq!(stats(&g, s), (3, 4));
        assert_eq!(g.state.players[p.index()].life, 17);
    }
}

#[test]
fn hob_flameshape_mana_sacrificing_last_wizard_preserves_approved_cast_and_locked_cost() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Goldvein Pick"]);
        let s = wizard(&mut g, p);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(s)));
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceExile { card: cards[0] });
        assert_eq!(
            g.state
                .announcing
                .as_ref()
                .unwrap()
                .total_cost
                .unwrap()
                .generic,
            2
        );
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: s,
                grantor,
                production: [0, 0, 3, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        act(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 2, 0, 0, 0],
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
    }
}

#[test]
fn gandalf_front_and_combined_names_resolve_to_same_physical_card() {
    let front = Card::from_name("Gandalf, Goblins' Bane").unwrap();
    let combined = Card::from_name("Gandalf, Goblins' Bane // Flameshape").unwrap();
    assert_eq!(front, combined);
    assert_eq!(front.name(), "Gandalf, Goblins' Bane");
}
