//! Focused printed outcomes; shared engine invariants live in core tests.
//! Exact linked exile and distinct type threshold for Keen-Eyed Curator.
use super::*;

fn activate(g: &mut Game, source: CardId, target: CardId) {
    big_helpers::checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    big_helpers::checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}

#[test]
fn curator_counts_multitype_cards_once_per_type_and_revalidates_targets() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Keen-Eyed Curator", Zone::Battlefield);
        let first = put(&mut g, p, "Forest", Zone::Graveyard);
        let duplicate = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let multi = put(&mut g, p, "Solemn Simulacrum", Zone::Graveyard);
        let instant = put(&mut g, p, "Giant Growth", Zone::Graveyard);
        for card in [first, duplicate, multi] {
            activate(&mut g, source, card);
            big_helpers::settle(&mut g);
            assert_eq!(g.stats(source).power, 3);
        }
        activate(&mut g, source, instant);
        g.move_object(instant, Zone::Hand);
        g.move_object(instant, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(instant).zone, Zone::Graveyard);
        assert_eq!(g.stats(source).power, 3);
        activate(&mut g, source, instant);
        big_helpers::settle(&mut g);
        assert_eq!(g.stats(source).power, 7);
        assert_eq!(g.stats(source).toughness, 7);
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
    }
}
