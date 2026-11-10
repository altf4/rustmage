//! Own-turn graveyard play and owner-scoped graveyard replacement.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const GRAVEYARD_PLAY: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::PlayFromGraveyardDuringOwnTurn],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: GRAVEYARD_PLAY,
    extras: &CardExtras {
        static_ability_groups: GRAVEYARD_PLAY,
        zone_replacement: Some(ZoneReplacementRule::ExileControllerGraveyard),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn own_turn_graveyard_cast_and_land_play_keep_ordinary_timing_and_limit() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let hand_land = put(&mut g, p, "Forest", Zone::Hand);
        let other_land = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Think Twice", Zone::Graveyard);
        let normal = Action::AnnounceCast { card: spell };
        let flashback = Action::AnnounceFlashback { card: spell };
        reject(&mut g, Action::PlayLand { card: land });
        reject(&mut g, normal.clone());

        g.state.objects[provider as usize].controller = p;
        g.state.active = p.other();
        reject(&mut g, normal.clone());
        reject(&mut g, Action::PlayLand { card: land });
        assert!(g.legal_actions().contains(&flashback));
        g.state.active = p;
        reject(&mut g, Action::PlayLand { card: other_land });
        act(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        reject(&mut g, Action::PlayLand { card: hand_land });

        let mut alternate = g.clone();
        act(&mut g, normal);
        pay(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Normal
        );
        act(&mut alternate, flashback);
        pay(&mut alternate);
        assert_eq!(
            alternate.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Flashback
        );
        for branch in [&mut g, &mut alternate] {
            branch.move_object(provider, Zone::Exile);
            settle(branch);
        }
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(alternate.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn ordinary_flash_surcharge_from_graveyard_does_not_set_flashback() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, p, "Asinine Antics", Zone::Graveyard);
    let normal = Action::AnnounceCast { card: spell };
    let surcharge = Action::AnnounceGraveyardFlashSurcharge { card: spell };
    g.state.step = Step::BeginCombat;
    reject(&mut g, normal);
    act(&mut g, surcharge);
    pay(&mut g);
    let cast = g.state.stack.last().unwrap().cast.unwrap();
    assert!(cast.flash_surcharge);
    assert_eq!(cast.method, CastMethod::Normal);
    g.move_object(provider, Zone::Exile);
    settle(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn source_departure_during_resolution_differs_from_lethal_damage() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P0;
    for (name, destination) in [("Murder", Zone::Graveyard), ("Shock", Zone::Exile)] {
        let mut g = scenario(p);
        let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, name, Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(provider)));
        settle(&mut g);
        assert_eq!(g.object_view(provider).zone, Zone::Exile);
        assert_eq!(g.object_view(spell).zone, destination);
    }
}

#[test]
fn frozen_owner_scoped_replacement_competes_with_shuffle_and_restores() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let owned = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.state.objects[owned as usize].controller = p.other();
    let foreign = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    g.state.objects[foreign as usize].controller = p;
    let shuffler = put(&mut g, p, "Progenitus", Zone::Battlefield);
    g.state.objects[shuffler as usize].controller = p.other();
    let wrath = put(&mut g, p, "Day of Judgment", Zone::Hand);
    cast(&mut g, wrath, None);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);

    assert_eq!(g.actor(), Some(p.other()));
    assert!(g.observe(p).zone_replacement_choice.is_none());
    assert_eq!(
        g.observe(p.other()).zone_replacement_choice.unwrap().card,
        g.object_view(shuffler).card
    );
    assert_eq!(g.object_view(provider).zone, Zone::Battlefield);
    let saved = g.snapshot();
    let revision = g.revision();
    for (effect, destination) in [
        (ZoneReplacementKind::RevealShuffle, Zone::Library),
        (ZoneReplacementKind::Exile, Zone::Exile),
    ] {
        let choice = Action::ChooseZoneReplacement { effect };
        assert_eq!(g.apply(p, choice.clone()), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), saved);
        let index = g.legal_actions().iter().position(|a| a == &choice).unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        act(&mut raw, choice);
        indexed.apply_index(p.other(), revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(shuffler).zone, destination);
        assert_eq!(raw.object_view(provider).zone, Zone::Exile);
        assert_eq!(raw.object_view(owned).zone, Zone::Exile);
        assert_eq!(raw.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(raw.object_view(wrath).zone, Zone::Graveyard);
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.actor(), Some(p.other()));
        assert_eq!(
            indexed.apply_index(p.other(), revision, index),
            Err(Error::StaleRevision)
        );
    }
}

#[test]
fn replacement_keeps_discard_events_and_uses_live_controller_in_virtual_costs() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let provider = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.state.objects[provider as usize].controller = p;
    let watcher = put(
        &mut g,
        p.other(),
        "Tinybones, Bauble Burglar",
        Zone::Battlefield,
    );
    let discarded = put(&mut g, p, "Mountain", Zone::Hand);
    assert!(
        !g.observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == discarded)
    );
    g.discard_batch(&[discarded], None);
    assert_eq!(g.object_view(discarded).zone, Zone::Exile);
    assert!(
        g.observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == discarded)
    );
    assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
    assert!(
        g.state
            .pending_triggers
            .iter()
            .any(|trigger| { trigger.source.is_some_and(|source| source.id == watcher) })
    );

    let owned = put(&mut g, p, "Forest", Zone::Battlefield);
    let foreign = put(&mut g, p.other(), "Forest", Zone::Battlefield);
    g.state.objects[foreign as usize].controller = p;
    let later = put(&mut g, p, "Forest", Zone::Battlefield);
    let refs = [provider, owned, foreign].map(|card| g.object_ref(card));
    let saved = g.snapshot();
    let world = FundingState::from_game(
        &g,
        &FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        },
    );
    assert!(world.cost_departures(&refs, Zone::Graveyard, &mut |next| {
        assert_eq!(next.object(provider).zone, Zone::Graveyard);
        assert_eq!(next.object(owned).zone, Zone::Exile);
        assert_eq!(next.object(foreign).zone, Zone::Graveyard);
        next.cost_departures(&[next.reference(later)], Zone::Graveyard, &mut |end| {
            assert_eq!(end.object(later).zone, Zone::Graveyard);
            true
        })
    }));
    assert_eq!(g.snapshot(), saved);
}
