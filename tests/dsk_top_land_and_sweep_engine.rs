//! Exact top-library decisions and simultaneous live-power destruction.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const P: Player = Player::P0;
const TOP_LAND: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LookTopMayPutLandOntoBattlefieldElseHand],
    ),
    ..CardRules::EMPTY
};
const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DestroyCreaturesWithPowerAtLeast(3)],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn exact_top_land_can_enter_or_go_to_hand_and_the_look_stays_private() {
    let definition = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &TOP_LAND)]);
    for (name, land) in [("Island", true), ("Grizzly Bears", false)] {
        let mut game = scenario(P);
        let under = put(&mut game, P, "Forest", Zone::Library);
        let top = put(&mut game, P, name, Zone::Library);
        let spell = spawn(&mut game, P, definition, Zone::Hand);
        cast(&mut game, spell, None);
        pass_pair(&mut game);

        let selection = game.library_selection_view(P).unwrap();
        assert_eq!(selection.remaining.len(), 1);
        assert_eq!(selection.remaining[0].reference, game.object_ref(top));
        assert_eq!(selection.candidates.len(), usize::from(land));
        assert!(game.library_selection_view(P.other()).is_none());
        let select = Action::SelectLibrary { card: top };
        reject(&mut game, Action::SelectLibrary { card: under });
        let mut declined = game.clone();
        act(&mut declined, Action::FinishLibrarySelection);
        assert_eq!(declined.object_view(top).zone, Zone::Hand);
        assert_eq!(declined.object_view(under).zone, Zone::Library);
        assert_eq!(declined.state.players[P.index()].cards_drawn_this_turn, 0);
        assert!(
            !declined
                .observe(P.other())
                .known_hand
                .contains(&declined.object_ref(top))
        );
        let mut previously_revealed = game.clone();
        previously_revealed.remember_library_top(P, previously_revealed.object_ref(top));
        act(&mut previously_revealed, Action::FinishLibrarySelection);
        assert!(
            previously_revealed
                .observe(P.other())
                .known_hand
                .contains(&previously_revealed.object_ref(top))
        );
        if land {
            let checkpoint = game.snapshot();
            let mut indexed = game.clone();
            let index = indexed
                .legal_actions()
                .iter()
                .position(|action| action == &select)
                .unwrap();
            indexed.apply_index(P, indexed.revision(), index).unwrap();
            act(&mut game, select.clone());
            assert_eq!(game.snapshot(), indexed.snapshot());
            act(&mut game, Action::FinishLibrarySelection);
            assert_eq!(game.object_view(top).zone, Zone::Battlefield);
            assert!(!game.object_view(top).tapped);
            assert_eq!(game.object_view(under).zone, Zone::Library);
            assert_eq!(game.state.players[P.index()].lands_entered_this_turn, 1);
            game.restore(&checkpoint).unwrap();
            act(&mut game, select);
            assert_eq!(game.snapshot(), indexed.snapshot());
        } else {
            reject(&mut game, select);
        }
    }
}

#[test]
fn a_card_with_both_zone_moves_prevented_stays_the_same_library_incarnation() {
    struct Prevent;
    impl Drop for Prevent {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let definition = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &TOP_LAND)]);
    let mut game = scenario(P);
    let top = put(&mut game, P, "Island", Zone::Library);
    let original = game.object_ref(top);
    let spell = spawn(&mut game, P, definition, Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    act(&mut game, Action::SelectLibrary { card: top });
    let _prevent = Prevent;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(original)));
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(top).zone, Zone::Library);
    assert_eq!(game.object_ref(top), original);
    assert_eq!(game.state.pending, Pending::Priority);
    assert_eq!(
        game.observe(P).known_library_top[P.index()].unwrap().object,
        original
    );
    assert!(game.observe(P.other()).known_library_top[P.index()].is_none());
    assert_eq!(game.state.players[P.index()].lands_entered_this_turn, 0);
    assert_eq!(game.state.players[P.index()].cards_drawn_this_turn, 0);
}

#[test]
fn a_fixed_power_sweep_uses_current_stats_after_responses() {
    let definition = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &SWEEP)]);
    let mut game = scenario(P);
    let threshold = put(&mut game, P, "Hill Giant", Zone::Battlefield);
    let grows = put(&mut game, P.other(), "Grizzly Bears", Zone::Battlefield);
    let survives = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let indestructible = put(&mut game, P.other(), "Hill Giant", Zone::Battlefield);
    game.grant_keyword_effects(
        &[indestructible],
        game.object_ref(indestructible),
        Keywords::INDESTRUCTIBLE,
    );
    let spell = spawn(&mut game, P, definition, Zone::Hand);
    let grow = put(&mut game, P.other(), "Giant Growth", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    cast(&mut game, grow, Some(Target::Card(grows)));
    settle(&mut game);
    assert_eq!(game.object_view(threshold).zone, Zone::Graveyard);
    assert_eq!(game.object_view(grows).zone, Zone::Graveyard);
    assert_eq!(game.object_view(survives).zone, Zone::Battlefield);
    assert_eq!(game.object_view(indestructible).zone, Zone::Battlefield);
}
