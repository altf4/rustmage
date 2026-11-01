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
fn curator_counts_distinct_linked_types_and_tracks_exile_incarnations() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Keen-Eyed Curator", Zone::Hand);
        big_helpers::cast(&mut g, source, None, &[]);
        big_helpers::settle(&mut g);
        let cards = [
            put(&mut g, p, "Forest", Zone::Graveyard),
            put(&mut g, p.other(), "Bear Cub", Zone::Graveyard),
            put(&mut g, p, "Giant Growth", Zone::Graveyard),
            put(&mut g, p, "Sleight of Hand", Zone::Graveyard),
        ];
        for (i, &id) in cards.iter().enumerate() {
            activate(&mut g, source, id);
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_eq!(g.stats(source).power, if i == 3 { 7 } else { 3 });
        }
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        let before = g.snapshot();
        let mut branch = g.clone();
        g.move_object(cards[0], Zone::Graveyard);
        g.move_object(cards[0], Zone::Exile);
        assert_eq!(g.stats(source).power, 3);
        assert!(!g.keywords(source).contains(Keywords::TRAMPLE));
        branch.restore(&before).unwrap();
        branch.move_object(cards[0], Zone::Graveyard);
        branch.move_object(cards[0], Zone::Exile);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}

#[test]
fn curator_exiles_after_source_departure_but_new_incarnation_has_no_links() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Keen-Eyed Curator", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        activate(&mut g, source, target);
        let old = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        assert_ne!(old, g.object_ref(source));
        assert_eq!(g.stats(source).power, 3);
        assert!(!g.keywords(source).contains(Keywords::TRAMPLE));
        let invalid = put(&mut g, p, "Bear Cub", Zone::Hand);
        put(&mut g, p, "Forest", Zone::Graveyard);
        big_helpers::checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(invalid),
            },
        );
    }
}

#[test]
fn curator_counts_multitype_cards_once_per_type_and_revalidates_targets() {
    for p in [Player::P0, Player::P1] {
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
