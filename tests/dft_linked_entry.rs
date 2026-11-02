//! Generic non-targeted entry exiles seed counters from exact successful results.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;
const ENTRY: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    extras: &CardExtras {
        x_symbols: 1,
        entry_exile_counters: Some(EntryExileCountersSpec {
            filter: SearchFilter::Creature,
            kind: CounterKind::PlusOne,
            per_card: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROHIBIT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};
fn goal() -> FundingGoal {
    FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn cast_x(g: &mut Game, x: u32) -> CardId {
    let id = put(g, Player::P0, "Pterafractyl", Zone::Hand);
    act(g, Action::AnnounceCast { card: id });
    act(g, Action::ChooseX { value: x });
    pay(g);
    for _ in 0..20 {
        if g.state.pending != Pending::Priority || g.state.stack.is_empty() {
            break;
        }
        act(g, Action::Pass);
    }
    id
}
fn direct(g: &mut Game, x: u32) -> CardId {
    let id = put(g, Player::P0, "Pterafractyl", Zone::Exile);
    let mut m = g.entry_move(id, Zone::Battlefield, false);
    m.x = x;
    g.stage_entries(vec![m], None);
    g.advance_entries();
    id
}
fn pick(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseEntryExile { object });
}
#[test]
fn chosen_x_entry_exiles_only_own_creature_cards_and_links_the_new_entrant() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &ENTRY)]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let b = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let wrong = put(&mut g, Player::P0, "Forest", Zone::Graveyard);
    let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Graveyard);
    let entrant = cast_x(&mut g, 2);
    assert!(matches!(g.state.pending, Pending::EntryExile { .. }));
    for id in [wrong, enemy] {
        let object = g.object_ref(id);
        reject(&mut g, Action::ChooseEntryExile { object });
    }
    pick(&mut g, a);
    let stale = g.object_ref(a);
    reject(&mut g, Action::ChooseEntryExile { object: stale });
    pick(&mut g, b);
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 6);
    assert_eq!(stats(&g, entrant), (6, 7));
    let links = &g.state.linked_choices.as_ref().unwrap().links;
    assert_eq!(links.len(), 2);
    assert!(
        links
            .iter()
            .all(|l| l.source == g.object_ref(entrant) && l.ability_origin.is_none())
    );
    assert!(
        links
            .iter()
            .all(|l| [g.object_ref(a), g.object_ref(b)].contains(&l.exiled))
    );
}
#[test]
fn optional_entry_selection_can_finish_below_x_and_x_zero_has_no_choice() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &ENTRY)]);
    let mut g = scenario(Player::P0);
    let a = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let b = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let entrant = direct(&mut g, 3);
    pick(&mut g, a);
    act(&mut g, Action::FinishEntryExile);
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 3);
    assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
    let zero = direct(&mut g, 0);
    settle(&mut g);
    assert_eq!(g.state.objects[zero as usize].zone, Zone::Battlefield);
    assert_eq!(g.counter_count(zero, CounterKind::PlusOne), 0);
    assert_eq!(g.state.linked_choices.as_ref().unwrap().links.len(), 1);
}
#[test]
fn simultaneous_graveyard_entrants_are_unavailable_for_the_entry_exile_choice() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &ENTRY)]);
    let mut g = scenario(Player::P0);
    let entrant = put(&mut g, Player::P0, "Pterafractyl", Zone::Graveyard);
    let other = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let eligible = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let mut m = g.entry_move(entrant, Zone::Battlefield, false);
    m.x = 2;
    g.stage_entries(vec![m, g.entry_move(other, Zone::Battlefield, false)], None);
    g.advance_entries();
    for id in [entrant, other] {
        let object = g.object_ref(id);
        reject(&mut g, Action::ChooseEntryExile { object });
    }
    pick(&mut g, eligible);
    settle(&mut g);
    assert_eq!(g.state.objects[other as usize].zone, Zone::Battlefield);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 3);
}
struct ClearPrevention;
impl Drop for ClearPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}
#[test]
fn prevented_selected_exile_grants_neither_entry_counters_nor_a_link_in_actual_or_projection() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &ENTRY)]);
    let mut g = scenario(Player::P0);
    let selected = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let reference = g.object_ref(selected);
    let _clear = ClearPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
    let entrant = put(&mut g, Player::P0, "Pterafractyl", Zone::Exile);
    let mut m = g.entry_move(entrant, Zone::Battlefield, false);
    m.x = 1;
    m.exile_preparation = Some(Box::new(entry_exiles::Preparation {
        selected: vec![reference],
        selection_closed: true,
        ..Default::default()
    }));
    let world = FundingState::from_game(&g, &goal());
    let mut projected = false;
    world.visit_entry_moves(vec![m.clone()], true, None, &mut |next| {
        assert_eq!(next.object(selected).zone, Zone::Graveyard);
        assert_eq!(next.counter_count(entrant, CounterKind::PlusOne), 0);
        projected = true;
        false
    });
    assert!(projected);
    g.stage_entries(vec![m], None);
    g.advance_entries();
    assert_eq!(g.state.objects[selected as usize].zone, Zone::Graveyard);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 0);
    assert!(
        g.state
            .linked_choices
            .as_ref()
            .is_none_or(|s| s.links.is_empty())
    );
}
#[test]
fn true_graveyard_exile_prohibition_stops_effects_and_projected_moves_without_a_replacement_choice()
{
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Pterafractyl").unwrap(), &ENTRY),
        (Card::from_name("Swiftfoot Boots").unwrap(), &PROHIBIT),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let card = put(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let reference = g.object_ref(card);
    let event = g.move_object(card, Zone::Exile);
    assert!(!event.moved);
    assert_eq!(g.object_ref(card), reference);
    let mut world = FundingState::from_game(&g, &goal());
    world.prepare_entry_projection();
    let mut projected = false;
    world.cost_departures(&[reference], Zone::Exile, &mut |next| {
        assert_eq!(next.object(card).zone, Zone::Graveyard);
        assert_eq!(next.reference(card), reference);
        projected = true;
        false
    });
    assert!(projected);
    let entrant = direct(&mut g, 1);
    pick(&mut g, card);
    settle(&mut g);
    assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), 0);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
    assert!(
        g.state
            .linked_choices
            .as_ref()
            .is_none_or(|s| s.links.is_empty())
    );
}
#[test]
fn partial_entry_exile_selection_is_owned_across_raw_index_restore_and_parent_disposal() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pterafractyl").unwrap(), &ENTRY)]);
    let mut parent = scenario(Player::P0);
    let a = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Graveyard);
    put(&mut parent, Player::P0, "Savannah Lions", Zone::Graveyard);
    let entrant = direct(&mut parent, 2);
    let saved = parent.snapshot();
    let action = Action::ChooseEntryExile {
        object: parent.object_ref(a),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    act(&mut raw, Action::FinishEntryExile);
    settle(&mut raw);
    assert_eq!(raw.counter_count(entrant, CounterKind::PlusOne), 3);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::FinishEntryExile);
    settle(&mut indexed);
    assert_eq!(indexed.counter_count(entrant, CounterKind::PlusOne), 0);
    assert_eq!(indexed.state.objects[a as usize].zone, Zone::Graveyard);
}
