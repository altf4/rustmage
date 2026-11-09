//! A required search selects a zone before applying the public or hidden rules.
mod support;

use rustmage as _;
use rustmage_core::{cards::authored::CardRules, cards::*, engine::*};
use support::*;

const FIND_ARTIFACT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchRequiredLibraryAndOrGraveyardToHand(
            SearchFilter::KindsManaValue {
                kinds: 16,
                minimum: 0,
                maximum: 2,
            },
        )],
    ),
    ..CardRules::EMPTY
};

fn zones(library: bool, graveyard: bool) -> Action {
    Action::ChooseSearchZones { library, graveyard }
}

#[test]
fn required_zone_search_preserves_public_selection_and_hidden_failure() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &FIND_ARTIFACT)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let public = put(&mut parent, player, "Swiftfoot Boots", Zone::Graveyard);
    let private = put(&mut parent, player, "Swiftfoot Boots", Zone::Library);
    let expensive = put(&mut parent, player, "Meteor Golem", Zone::Graveyard);
    let source = put(&mut parent, player, "Opt", Zone::Hand);
    let unchanged_library = parent.state.players[player.index()].library.clone();
    cast(&mut parent, source, None);
    act(&mut parent, Action::Pass);
    act(&mut parent, Action::Pass);
    assert_eq!(
        parent
            .legal_actions()
            .iter()
            .filter(|action| { matches!(action, Action::ChooseSearchZones { .. }) })
            .count(),
        3,
    );
    reject(&mut parent, zones(false, false));

    let mut graveyard = parent.clone();
    let mut indexed = parent.clone();
    let choice = zones(false, true);
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let revision = indexed.revision();
    act(&mut graveyard, choice);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), graveyard.snapshot());
    reject(&mut graveyard, Action::FinishSearch);
    reject(&mut graveyard, Action::ChooseSearch { card: expensive });
    reject(&mut graveyard, Action::ChooseSearch { card: private });
    act(&mut graveyard, Action::ChooseSearch { card: public });
    act(&mut graveyard, Action::FinishSearch);
    settle(&mut graveyard);
    assert_eq!(graveyard.object_view(public).zone, Zone::Hand);
    assert_eq!(
        graveyard.state.players[player.index()].library,
        unchanged_library
    );
    assert!(
        graveyard
            .observe(player.other())
            .known_hand
            .contains(&graveyard.object_ref(public))
    );

    let mut library = parent.clone();
    act(&mut library, zones(true, false));
    assert!(
        library
            .legal_actions()
            .contains(&Action::ChooseSearch { card: private })
    );
    assert!(
        library
            .observe(player.other())
            .zone_search
            .unwrap()
            .cards
            .is_none()
    );
    act(&mut library, Action::FinishSearch);
    settle(&mut library);
    assert_eq!(library.object_view(private).zone, Zone::Library);
    assert_eq!(library.object_view(public).zone, Zone::Graveyard);

    act(&mut parent, zones(true, true));
    reject(&mut parent, Action::FinishSearch);
    act(&mut parent, Action::ChooseSearch { card: public });
    act(&mut parent, Action::ChooseSearch { card: private });
    act(&mut parent, Action::FinishSearch);
    settle(&mut parent);
    assert_eq!(parent.object_view(private).zone, Zone::Hand);
    assert_eq!(parent.object_view(public).zone, Zone::Graveyard);
}
