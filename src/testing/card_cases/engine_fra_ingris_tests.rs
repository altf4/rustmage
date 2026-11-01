//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn ingris_only_observes_its_controllers_attackers_and_grants_haste_to_current_creatures() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let ingris = put(&mut g, p, "Ingris Stingerquill", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, ingris), (1, 4));
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ingris,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        g.move_object(ingris, Zone::Graveyard);
        settle(&mut g);
        let cadet = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.card == Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap()
                    && o.controller == p)
                    .then_some(id as CardId)
            })
            .unwrap();
        assert!(g.keywords(cadet).contains(Keywords::HASTE));
        assert!(g.keywords(own).contains(Keywords::HASTE));
        assert!(!g.keywords(enemy).contains(Keywords::HASTE));
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::HASTE));
        g.finish_cleanup();
        assert!(!g.keywords(own).contains(Keywords::HASTE));
        assert!(!g.keywords(cadet).contains(Keywords::HASTE));
    }
}
