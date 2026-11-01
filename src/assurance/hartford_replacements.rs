use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn lynx_counts_each_players_nonbasic_lands_and_suppresses_life_gain_and_prevention() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Abraded Bluffs", Zone::Battlefield);
        put(&mut g, p, "Forest", Zone::Battlefield);
        put(&mut g, p.other(), "Abraded Bluffs", Zone::Battlefield);
        put(&mut g, p.other(), "Bristling Backwoods", Zone::Battlefield);
        let lynx = put(&mut g, p, "Sunspine Lynx", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: lynx });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(g.damage_prevention_prohibited());
        g.gain_life(p, 4);
        assert_eq!(g.state.players[p.index()].life, 19);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let source = g.object_ref(prevention);
        let recipient = g.object_ref(dealer);
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
        let event = DamageEvent {
            source: g.damage_source(dealer),
            target: Target::Player(p.other()),
            amount: 2,
            combat: false,
            preventable: true,
        };
        g.apply_damage_batch(&[event]);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let snapshot = g.snapshot();
        let mut branch = g.clone();
        branch.restore(&snapshot).unwrap();
        assert_eq!(branch.snapshot(), snapshot);
        g.move_object(lynx, Zone::Graveyard);
        put(&mut g, p, "Crystal Barricade", Zone::Battlefield);
        assert!(!g.damage_prevention_prohibited());
        g.gain_life(p, 1);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}

#[test]
fn vren_tracks_preentry_and_replaced_exiles_and_creates_scaling_rats() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let prior = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(prior, Zone::Exile);
        let vren = put(&mut g, p, "Vren, the Relentless", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(other, Zone::Graveyard);
        assert_eq!(g.object_view(other).zone, Zone::Exile);
        assert_eq!(
            g.state.players[p.other().index()].controlled_creatures_exiled_this_turn,
            2
        );
        g.begin_step(Step::End);
        settle(&mut g);
        let rats: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card.identity() == "@token/blb-vren-rat"
            })
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(rats.len(), 2);
        for rat in rats {
            assert_eq!(stats(&g, rat), (3, 3));
        }
        let snapshot = g.snapshot();
        let mut branch = g.clone();
        branch.restore(&snapshot).unwrap();
        assert_eq!(branch.snapshot(), snapshot);
        g.move_object(vren, Zone::Graveyard);
        let unprotected = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(unprotected, Zone::Graveyard);
        assert_eq!(g.object_view(unprotected).zone, Zone::Graveyard);
    }
}
