//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn vanguard_vigilance_follows_live_counters_and_controller_and_source() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Vigorbloom Vanguard // Seed Suture",
            Zone::Battlefield,
        );
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.add_counter(source, CounterKind::PlusOne, 1);
        g.add_counter(own, CounterKind::PlusOne, 1);
        g.add_counter(enemy, CounterKind::PlusOne, 1);
        assert!(g.keywords(source).contains(Keywords::VIGILANCE));
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        assert!(!g.keywords(enemy).contains(Keywords::VIGILANCE));
        g.change_controller(g.object_ref(own), p.other());
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        g.change_controller(g.object_ref(own), p);
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        g.move_object(source, Zone::Graveyard);
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
    }
}
