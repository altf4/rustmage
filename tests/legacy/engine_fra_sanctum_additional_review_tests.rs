//! Independent gated Sanctum tests: copy-layer types and damage prevention.
use super::*;

fn sanctum(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Sanctum Lurker").unwrap();
    g.register_battlefield_sources(id);
    id
}

#[test]
fn sanctum_grant_follows_copied_planeswalker_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = sanctum(&mut g, p);
        let object = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let reference = g.object_ref(object);
        g.state.objects[object as usize].counters.loyalty = 1;
        let action = Action::AnnouncePlaneswalkerGrant {
            source: reference,
            grantor: g.object_ref(provider),
            ability: 0,
        };
        assert!(!g.legal_actions().contains(&action));
        let before = g.snapshot();
        for _ in 0..2 {
            g.apply_temporary_copy(
                reference,
                CopyRecipe::new(Card::from_name("Ajani, Caller of the Pride").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_ne!(g.current_kinds(object) & 64, 0);
            assert!(g.legal_actions().contains(&action));
            let copied = g.snapshot();
            act(&mut g, action.clone());
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.counter_count(object, CounterKind::Loyalty), 3);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            g.restore(&copied).unwrap();
            g.restore(&before).unwrap();
            assert_eq!(g.current_kinds(object) & 64, 0);
            assert!(!g.legal_actions().contains(&action));
        }
    }
}

#[test]
fn sanctum_prevented_walker_damage_still_gains_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = sanctum(&mut g, p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 1;
        let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let recipient = g.object_ref(walker);
        let source = g.object_ref(prevention);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_default()
            .source_linked_effects
            .push(SourceLinkedEffectView {
                source,
                recipient,
                controller: p.other(),
                kind: SourceLinkedEffectKind::PreventDamageBy,
                timestamp,
            });
        let action = Action::AnnouncePlaneswalkerGrant {
            source: recipient,
            grantor: g.object_ref(provider),
            ability: 0,
        };
        let before = g.snapshot();
        for _ in 0..2 {
            act(&mut g, action.clone());
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            assert!(!g.state.objects[walker as usize].dealt_damage_this_turn);
            g.restore(&before).unwrap();
        }
    }
}
