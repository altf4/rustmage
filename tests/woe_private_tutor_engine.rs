//! A private search result can be cast once during resolution or go to hand.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TUTOR: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CaptureSearchResult,
            Instruction::SearchRequired(SearchSpec {
                filter: SearchFilter::Any,
                maximum: 1,
                destination: SearchDestination::ExileFaceDown,
                tapped: false,
                reveal: false,
            }),
            Instruction::IfBargained { skip: 1 },
            Instruction::CastExiledSearchResultFreeUpTo(4),
            Instruction::PutExiledSearchResultInHand,
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn begin_search(game: &mut Game, player: Player, bargain: bool) {
    let tutor = put(game, player, "Quick Study", Zone::Hand);
    let offering = put(game, player, "Short Bow", Zone::Battlefield);
    let sacrifice = bargain.then(|| game.object_ref(offering));
    game.state.players[player.index()].mana = [0, 3, 0, 0, 0, 0];
    act(game, Action::AnnounceCast { card: tutor });
    act(game, Action::ChooseBargain { sacrifice });
    pay(game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().bargain,
        Some(bargain)
    );
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::EffectSearch { player });
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}

fn choose_found(game: &mut Game, card: CardId) {
    act(game, Action::ChooseSearch { card });
    act(game, Action::FinishSearch);
}

fn cast_found(object: ObjectRef, face: CastFace) -> Action {
    Action::CastExiledDuringResolution { object, face }
}

fn private_result(game: &Game, card: CardId, player: Player, zone: Zone) {
    assert_eq!(game.object_view(card).zone, zone);
    assert!(game.observe(player).objects.iter().any(|o| o.id == card));
    assert!(
        !game
            .observe(player.other())
            .objects
            .iter()
            .any(|o| o.id == card)
    );
}

#[test]
fn decline_and_uncastable_results_stay_private_through_search_and_fallback() {
    let tutor = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(tutor, &TUTOR)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let affordable = put(&mut parent, player, "Hill Giant", Zone::Library);
    let too_expensive = put(&mut parent, player, "Shivan Dragon", Zone::Library);
    let no_targets = put(&mut parent, player, "Unsummon", Zone::Library);
    let land = put(&mut parent, player, "Island", Zone::Library);
    let top = parent.object_ref(land);
    parent.remember_library_top(player, top);
    let before_shuffle = parent.state.players[player.index()].library.clone();
    begin_search(&mut parent, player, true);
    assert!(parent.observe(player).search_choice.is_some());
    assert!(parent.observe(player.other()).search_choice.is_none());
    reject(&mut parent, Action::FinishSearch);
    let saved = parent.snapshot();

    let mut decline = parent.clone();
    choose_found(&mut decline, affordable);
    private_result(&decline, affordable, player, Zone::Exile);
    let exact = decline.object_ref(affordable);
    assert_ne!(
        decline.state.players[player.index()].library.to_vec(),
        before_shuffle
            .into_iter()
            .filter(|&id| id != affordable)
            .collect::<Vec<_>>()
    );
    assert!(
        decline
            .legal_actions()
            .contains(&cast_found(exact, CastFace::Normal))
    );
    assert_eq!(
        decline
            .observe(player)
            .resolution_free_casts
            .unwrap()
            .remaining,
        [exact]
    );
    let opponent_offer = serde_json::to_value(decline.observe(player.other())).unwrap();

    let mut stale = decline.clone();
    stale.move_object(affordable, Zone::Graveyard);
    stale.move_object(affordable, Zone::Exile);
    assert_ne!(stale.object_ref(affordable), exact);
    reject(&mut stale, cast_found(exact, CastFace::Normal));
    act(&mut stale, Action::FinishResolutionCasts);
    assert_eq!(stale.object_view(affordable).zone, Zone::Exile);

    act(&mut decline, Action::FinishResolutionCasts);
    private_result(&decline, affordable, player, Zone::Hand);
    let opponent_result = serde_json::to_value(decline.observe(player.other())).unwrap();
    for uncastable in [too_expensive, no_targets, land] {
        let mut branch = parent.clone();
        choose_found(&mut branch, uncastable);
        private_result(&branch, uncastable, player, Zone::Exile);
        assert_eq!(
            serde_json::to_value(branch.observe(player.other())).unwrap(),
            opponent_offer
        );
        assert_eq!(
            branch.legal_actions(),
            [Action::Concede, Action::FinishResolutionCasts]
        );
        let action = cast_found(branch.object_ref(uncastable), CastFace::Normal);
        reject(&mut branch, action);
        act(&mut branch, Action::FinishResolutionCasts);
        private_result(&branch, uncastable, player, Zone::Hand);
        assert_eq!(
            serde_json::to_value(branch.observe(player.other())).unwrap(),
            opponent_result
        );
    }
    assert_eq!(parent.snapshot(), saved);
}

#[test]
fn unbargained_search_goes_to_hand_and_an_empty_search_has_no_offer() {
    let tutor = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(tutor, &TUTOR)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Hill Giant", Zone::Library);
    begin_search(&mut game, player, false);
    choose_found(&mut game, target);
    private_result(&game, target, player, Zone::Hand);
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.observe(player).resolution_free_casts.is_none());

    let mut empty = scenario(player);
    for id in empty.state.players[player.index()].library.clone() {
        empty.move_object(id, Zone::Graveyard);
    }
    begin_search(&mut empty, player, true);
    act(&mut empty, Action::FinishSearch);
    assert_eq!(empty.state.pending, Pending::Priority);
    assert!(empty.observe(player).resolution_free_casts.is_none());
}

#[test]
fn each_spell_face_qualifies_on_its_own_and_the_exact_offer_casts_once() {
    let tutor = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(tutor, &TUTOR)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let four = put(&mut parent, player, "Hill Giant", Zone::Library);
    let cheap_adventure = put(
        &mut parent,
        player,
        "Beanstalk Wurm // Plant Beans",
        Zone::Library,
    );
    let cheap_permanent = put(
        &mut parent,
        player,
        "Bramble Familiar // Fetch Quest",
        Zone::Library,
    );
    let unrelated = put(&mut parent, player, "Grizzly Bears", Zone::Exile);
    let unrelated = parent.object_ref(unrelated);
    parent.remember_face_down_exile(unrelated, 1 << player.index(), None);
    begin_search(&mut parent, player, true);

    for (card, face, mana_value, destination) in [
        (four, CastFace::Normal, 4, Zone::Battlefield),
        (cheap_adventure, CastFace::Adventure, 2, Zone::Exile),
        (cheap_permanent, CastFace::Normal, 2, Zone::Battlefield),
    ] {
        let mut root = parent.clone();
        choose_found(&mut root, card);
        let object = root.object_ref(card);
        let action = cast_found(object, face);
        let alternate = match face {
            CastFace::Normal => CastFace::Adventure,
            CastFace::Adventure => CastFace::Normal,
            _ => unreachable!(),
        };
        reject(&mut root, cast_found(object, alternate));
        reject(&mut root, cast_found(unrelated, CastFace::Normal));
        let choices = root.legal_actions();
        let index = choices
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        let revision = root.revision();
        let saved = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        act(&mut raw, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for branch in [&mut raw, &mut indexed] {
            pay(branch);
            let spell = branch.state.stack.last().unwrap();
            assert_eq!(spell.cast.unwrap().method, CastMethod::Free);
            assert_eq!(spell.cast.unwrap().face, face);
            assert_eq!(spell.mana_value(), mana_value);
            assert!(
                branch
                    .observe(player.other())
                    .objects
                    .iter()
                    .any(|o| o.id == card)
            );
            assert_eq!(
                branch
                    .observe(player)
                    .resolution_free_casts
                    .unwrap()
                    .casts_remaining,
                Some(0)
            );
            assert_eq!(
                branch.legal_actions(),
                [Action::Concede, Action::FinishResolutionCasts]
            );
            act(branch, Action::FinishResolutionCasts);
            settle(branch);
            assert_eq!(branch.object_view(card).zone, destination);
            assert_ne!(branch.object_ref(card), object);
            if face == CastFace::Adventure {
                assert!(branch.can_play_adventure(player, card));
            }
            assert_eq!(branch.object_ref(unrelated.card), unrelated);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        raw.restore(&saved).unwrap();
        act(&mut raw, Action::FinishResolutionCasts);
        private_result(&raw, card, player, Zone::Hand);
    }
}
