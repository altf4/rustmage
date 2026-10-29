//! Current land subtypes, rather than the physical face, determine mana.
use super::super::ability_world::CharacteristicWorld;
use super::super::*;
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

struct ProjectedSubtype<'a> {
    game: &'a Game,
    object: CardId,
    kind: Option<crate::cards::BasicLandType>,
}
impl CharacteristicWorld for ProjectedSubtype<'_> {
    fn game(&self) -> &Game {
        self.game
    }
    fn object(&self, id: CardId) -> &Object {
        &self.game.state.objects[id as usize]
    }
    fn subtypes(&self, id: CardId) -> GrantedSubtypes {
        if id == self.object {
            GrantedSubtypes {
                basic_land: self.kind,
                ..GrantedSubtypes::default()
            }
        } else {
            subtypes::in_world(self.game, id)
        }
    }
}

#[test]
fn subtype_replacement_changes_mana_without_mutating_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Island", Zone::Battlefield);
        g.state.objects[land as usize].card = Card::from_name("Theorist's Sanctum").unwrap();
        let before = g.snapshot();
        let island = ProjectedSubtype {
            game: &g,
            object: land,
            kind: None,
        };
        let forest = ProjectedSubtype {
            game: &g,
            object: land,
            kind: Some(crate::cards::BasicLandType::Forest),
        };
        assert_eq!(island.intrinsic_mana_color(land), Some(1));
        assert_eq!(forest.intrinsic_mana_color(land), Some(4));
        assert_eq!(g.intrinsic_mana_color(land), Some(1));
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn ordinary_nonbasic_land_acquires_basic_subtype_mana_in_projected_world() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Meticulous Commons", Zone::Battlefield);
        let before = g.snapshot();
        let forest = ProjectedSubtype {
            game: &g,
            object: land,
            kind: Some(crate::cards::BasicLandType::Forest),
        };
        assert_eq!(g.intrinsic_mana_color(land), None);
        assert_eq!(forest.intrinsic_mana_color(land), Some(4));
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn projected_gained_forest_remains_green_with_unrelated_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Meticulous Commons", Zone::Battlefield);
        let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = named(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
        let before = g.snapshot();
        let forest = ProjectedSubtype {
            game: &g,
            object: land,
            kind: Some(crate::cards::BasicLandType::Forest),
        };
        assert!(forest.has_ability_removal());
        assert_eq!(forest.intrinsic_mana_color(land), Some(4));
        assert_eq!(g.intrinsic_mana_color(land), None);
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn characteristic_query_keeps_projected_copy_ability_origin() {
    struct OriginWorld<'a> {
        game: &'a Game,
        source: CardId,
    }
    impl CharacteristicWorld for OriginWorld<'_> {
        fn game(&self) -> &Game {
            self.game
        }
        fn object(&self, id: CardId) -> &Object {
            &self.game.state.objects[id as usize]
        }
        fn ability_origin(&self, id: CardId) -> Option<u64> {
            (id == self.source).then_some(42)
        }
    }
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let world = OriginWorld { game: &g, source };
        let query = characteristic_query::CharacteristicQuery::in_world(&world);
        assert_eq!(world.ability_origin(source), Some(42));
        assert_eq!(query.ability_origin(source), Some(42));
        assert_ne!(g.ability_origin(source), Some(42));
    }
}
