//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn lyra_activate(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    settle(g);
}
fn lyra_hit(g: &mut Game, source: CardId, player: Player, combat: bool) {
    let event = DamageEvent {
        source: g.damage_source(source),
        target: Target::Player(player),
        amount: 1,
        combat,
        preventable: true,
    };
    g.apply_damage_batch(&[event]);
}

#[test]
fn lyra_end_step_checks_actual_draw_history_and_creates_blue_angel() {
    {
        let p = Player::P0;
        for drawn in [2, 3] {
            let mut g = scenario(p);
            let lyra = put(&mut g, p, "Lyra, Tolarian Archangel", Zone::Battlefield);
            assert!(g.keywords(lyra).contains(Keywords::FLYING));
            g.draw_many(p, drawn);
            g.state.active = p.other();
            g.begin_step(Step::End);
            g.prepare_triggers();

            settle(&mut g);
            let angels: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    (o.zone == Zone::Battlefield
                        && o.card == Card::from_name("@token/fra-blue-angel").unwrap()
                        && o.controller == p)
                        .then_some(id as CardId)
                })
                .collect();
            assert_eq!(angels.len(), usize::from(drawn == 3));
            if let Some(&angel) = angels.first() {
                assert_eq!(stats(&g, angel), (3, 3));
                assert_eq!(g.object_view(angel).colors, 2);
                assert!(g.keywords(angel).contains(Keywords::FLYING));
                assert!(g.has_subtype(angel, "Angel"));
            }
        }
    }
}

#[test]
fn lyra_granted_damage_trigger_tracks_controller_and_expires() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let lyra = put(&mut g, p, "Lyra, Tolarian Archangel", Zone::Battlefield);
        lyra_hit(&mut g, lyra, p.other(), true);
        assert!(g.state.pending_triggers.is_empty());
        lyra_activate(&mut g, lyra);
        lyra_hit(&mut g, lyra, p.other(), false);
        assert!(g.state.pending_triggers.is_empty());
        g.change_controller(g.object_ref(lyra), p.other());
        let before = g.hand_count(p.other());
        lyra_hit(&mut g, lyra, p, true);
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.prepare_triggers();

        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), before + 2);
        g.finish_cleanup();
        lyra_hit(&mut g, lyra, p, true);
        assert!(g.state.pending_triggers.is_empty());
    }
}
