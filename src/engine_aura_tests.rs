use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
use crate::{SearchDestination, SearchFilter};
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn permanent(g: &mut Game, name: &str) -> CardId {
    named(g, Player::P0, name, Zone::Battlefield)
}
fn replay(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(g, action.clone());
    act(&mut fork, action.clone());
    assert_eq!(g.snapshot(), fork.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision;
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, revision);
}
fn funded(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, n) = g.state.objects[id as usize].card.cost();
    mana[5] += n;
    g.state.players[p.index()].mana = mana;
    id
}
fn choose_entry(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    replay(g, Action::ChooseAuraAttachment { object });
}
fn enter_attached(g: &mut Game, p: Player, name: &str, target: CardId) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    let mut movement = g.entry_move(id, Zone::Battlefield, false);
    movement.attachment = Some(g.object_ref(target));
    g.stage_entries(vec![movement], None);
    id
}
fn resolve_all(g: &mut Game) {
    g.prepare_triggers();
    while !g.state.stack.is_empty() || !g.state.ordering_triggers.is_empty() {
        if let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(g, a);
        } else {
            replay(g, Action::Pass);
            replay(g, Action::Pass);
        }
    }
}
// Synthetic IR/link fixtures below exercise future complete-card combinations;
// they do not admit an otherwise unsupported card or bypass public API validation.
fn program(g: &mut Game, code: &'static [Instruction], targets: Vec<BoundTarget>) {
    let id = named(g, Player::P0, "Shock", Zone::Hand);
    let mut item = g.cast_item(Player::P0, id, [0; 6]);
    item.targets = targets;
    g.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        selection: None,
        search: None,
        choice: None,
    });
    g.run_program();
    g.advance_entries();
}
fn link(g: &mut Game, source: CardId, aura: CardId) {
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(aura),
        return_zone: Zone::Battlefield,
    });
}

#[test]
fn cast_requires_a_legal_target_and_enters_attached_before_its_draw_trigger() {
    let mut g = scenario();
    let aura = funded(&mut g, Player::P0, "Feather of Flight");
    reject(
        &mut g,
        Action::Cast {
            card: aura,
            target: None,
            payment: [1, 0, 0, 0, 0, 1],
        },
    );
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: aura })
    );
    let bear = permanent(&mut g, "Grizzly Bears");
    replay(&mut g, Action::AnnounceCast { card: aura });
    reject(&mut g, Action::FinishTargets);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, pay);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
    assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(bear)));
    assert_eq!(g.stats(bear).power, 3);
    assert!(g.keywords(bear).contains(Keywords::FLYING));
    assert_eq!(g.state.players[0].library.len(), 20);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].source_attachment, Some(g.object_ref(bear)));
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
}
#[test]
fn source_target_incarnation_and_hexproof_revalidate_the_whole_aura_spell() {
    for hexproof in [false, true] {
        let mut g = scenario();
        let bear = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let aura = funded(&mut g, Player::P0, "Feather of Flight");
        cast(&mut g, aura, Some(Target::Card(bear)));
        if hexproof {
            g.state.objects[bear as usize].gained_keywords = Keywords::HEXPROOF;
        } else {
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.players[0].library.len(), 20);
    }
}
#[test]
fn dead_weight_kills_indestructible_by_toughness_then_goes_to_graveyard() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[bear as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let aura = funded(&mut g, Player::P0, "Dead Weight");
    cast(&mut g, aura, Some(Target::Card(bear)));
    pass_pair(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(g.object_view(aura).attached_to, None);
    assert!(g.state.attachment_sources.is_empty());
}
#[test]
fn own_creature_aura_rejects_enemy_while_flash_does_not_remove_its_target_requirement() {
    let mut g = scenario();
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let aura = funded(&mut g, Player::P0, "Aquitect's Defenses");
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: aura })
    );
    let (mut payment, n) = g.state.objects[aura as usize].card.cost();
    payment[5] += n;
    reject(
        &mut g,
        Action::Cast {
            card: aura,
            target: Some(Target::Card(enemy)),
            payment,
        },
    );
    let own = permanent(&mut g, "Grizzly Bears");
    g.state.step = Step::BeginCombat;
    cast(&mut g, aura, Some(Target::Card(own)));
    resolve_all(&mut g);
    assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(own)));
    assert!(g.keywords(own).contains(Keywords::HEXPROOF));
}
#[test]
fn noncasting_entry_ignores_hexproof_and_stays_in_source_zone_until_choice() {
    let mut g = scenario();
    let enemy = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Exile);
    g.stage_entries(vec![g.entry_move(aura, Zone::Battlefield, false)], None);
    assert!(g.advance_entries());
    assert_eq!(g.observe(Player::P0).decision, "aura_entry");
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    assert_eq!(
        g.observe(Player::P1).aura_entry.unwrap().card,
        Card::from_name("Frantic Strength").unwrap()
    );
    reject(&mut g, Action::Pass);
    reject(
        &mut g,
        Action::ChooseAuraAttachment {
            object: ObjectRef {
                card: u32::MAX,
                generation: 0,
            },
        },
    );
    choose_entry(&mut g, enemy);
    assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(enemy)));
    assert_eq!(g.stats(enemy).power, 4);
    g.state_based_actions();
    assert_eq!(g.object_view(aura).zone, Zone::Battlefield); // Hexproof isn't an enchant restriction.
}
#[test]
fn simultaneous_entries_cannot_enchant_newcomers_and_impossible_auras_stay_put() {
    let mut g = scenario();
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Exile);
    let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
    g.stage_entries(
        vec![
            g.entry_move(aura, Zone::Battlefield, false),
            g.entry_move(bear, Zone::Battlefield, false),
        ],
        None,
    );
    assert!(!g.advance_entries());
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    let aura2 = named(&mut g, Player::P0, "Dead Weight", Zone::Stack);
    g.move_object(bear, Zone::Graveyard);
    g.stage_entries(vec![g.entry_move(aura2, Zone::Battlefield, false)], None);
    assert!(!g.advance_entries());
    assert_eq!(g.object_view(aura2).zone, Zone::Graveyard);
}
#[test]
fn all_public_apnap_attachments_precede_one_entry_batch_and_later_players_see_choices() {
    let mut g = scenario();
    g.state.active = Player::P1;
    let bear = permanent(&mut g, "Hill Giant");
    let a = named(&mut g, Player::P0, "Frantic Strength", Zone::Exile);
    let b = named(&mut g, Player::P1, "Untamed Hunger", Zone::Exile);
    g.stage_entries(
        vec![
            g.entry_move(a, Zone::Battlefield, false),
            g.entry_move(b, Zone::Battlefield, false),
        ],
        None,
    );
    g.advance_entries();
    assert_eq!(g.actor(), Some(Player::P1));
    choose_entry(&mut g, bear);
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.object_view(a).zone, Zone::Exile);
    assert_eq!(g.object_view(b).zone, Zone::Exile);
    let view = g.observe(Player::P0).aura_entry.unwrap();
    assert_eq!(view.selected.len(), 1);
    assert_eq!(view.selected[0].aura.card, b);
    choose_entry(&mut g, bear);
    assert_eq!(
        g.stats(bear),
        Stats {
            power: 7,
            toughness: 6
        }
    );
    assert!(g.state.entry_batch.is_none());
}
#[test]
fn search_selection_stays_private_through_attachment_choice_then_shuffles_and_resumes() {
    const CODE: &[Instruction] = &[
        Instruction::Search(SearchSpec {
            filter: SearchFilter::Enchantment,
            maximum: 2,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
        }),
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let a = named(&mut g, Player::P0, "Frantic Strength", Zone::Library);
    let b = named(&mut g, Player::P0, "Untamed Hunger", Zone::Library);
    program(&mut g, CODE, Vec::new());
    replay(&mut g, Action::ChooseSearch { card: a });
    replay(&mut g, Action::ChooseSearch { card: b });
    let before = g.state.players[0].library.clone();
    let rng = g.state.rng;
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.state.players[0].library, before);
    assert_eq!(g.state.rng, rng);
    assert!(g.observe(Player::P1).aura_entry.is_none());
    assert!(g.observe(Player::P1).actions.is_empty());
    assert!(
        !g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == a || o.id == b)
    );
    choose_entry(&mut g, bear);
    assert_eq!(g.object_view(a).zone, Zone::Library);
    assert_eq!(g.object_view(b).zone, Zone::Library);
    choose_entry(&mut g, bear);
    assert_eq!(g.state.players[0].library.len(), before.len() - 3);
    assert!(g.object_view(a).tapped && g.object_view(b).tapped);
    assert_eq!(g.stats(bear).power, 6);
    assert!(g.state.resolving.is_none());
}
#[test]
fn failed_search_entry_is_shuffled_back_and_the_program_still_resumes() {
    const CODE: &[Instruction] = &[
        Instruction::Search(SearchSpec {
            filter: SearchFilter::Enchantment,
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: false,
            reveal: false,
        }),
        Instruction::GainLife(PlayerSet::Controller, 2),
    ];
    let mut g = scenario();
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Library);
    program(&mut g, CODE, Vec::new());
    replay(&mut g, Action::ChooseSearch { card: aura });
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.object_view(aura).zone, Zone::Library);
    assert_eq!(g.state.players[0].library.len(), 21);
    assert_eq!(g.state.players[0].life, 22);
    assert!(g.state.entry_batch.is_none() && g.state.resolving.is_none());
}
#[test]
fn return_to_battlefield_program_waits_for_attachment_before_draw_and_sbas() {
    const CODE: &[Instruction] = &[
        Instruction::ReturnToBattlefield,
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    g.state.objects[bear as usize].damage = 3;
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Graveyard);
    let bound = g.bind_target(Target::Card(aura));
    program(&mut g, CODE, vec![bound]);
    assert_eq!(g.state.players[0].library.len(), 20);
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    assert!(g.state.resolving.is_some());
    choose_entry(&mut g, bear);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(g.stats(bear).toughness, 4);
}
#[test]
fn linked_exile_return_suspends_real_removal_resolution_and_replays_after_choice() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let aura = enter_attached(&mut g, Player::P1, "Frantic Strength", bear);
    let source = named(&mut g, Player::P0, "Banishing Light", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    pass_pair(&mut g);
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    let spell = funded(&mut g, Player::P0, "Disenchant");
    cast(&mut g, spell, Some(Target::Card(source)));
    pass_pair(&mut g);
    assert_eq!(g.observe(Player::P1).decision, "aura_entry");
    assert!(g.state.resolving.is_some());
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    choose_entry(&mut g, bear);
    assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.stats(bear).power, 4);
}
#[test]
fn sba_linked_return_choices_precede_more_sbas_and_trigger_ordering() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Hill Giant");
    let source = permanent(&mut g, "Grizzly Bears");
    let aura = named(&mut g, Player::P1, "Frantic Strength", Zone::Exile);
    link(&mut g, source, aura);
    g.state.objects[source as usize].damage = 2;
    g.state_based_actions();
    assert!(g.state.entry_batch.is_some());
    g.advance_entries();
    assert_eq!(g.actor(), Some(Player::P1));
    choose_entry(&mut g, bear);
    assert_eq!(g.stats(bear).power, 5);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert!(g.state.entry_batch.is_none());
}
#[test]
fn linked_return_during_activation_cost_preserves_the_independent_ability() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Hill Giant");
    let source = permanent(&mut g, "Cathar Commando");
    let bow = permanent(&mut g, "Short Bow");
    let aura = named(&mut g, Player::P1, "Frantic Strength", Zone::Exile);
    link(&mut g, source, aura);
    g.state.players[0].mana[5] = 1;
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,target:Some(Target::Card(t)),..} if *card==source && *t==bow)).unwrap();
    replay(&mut g, a);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    choose_entry(&mut g, bear);
    resolve_all(&mut g);
    assert_eq!(g.object_view(bow).zone, Zone::Graveyard);
    assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
}
#[test]
fn mana_cost_linked_return_restores_an_in_progress_payment_window() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let aura = named(&mut g, Player::P1, "Frantic Strength", Zone::Exile);
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap();
    g.create_tokens(Player::P0, token, 1, false);
    let source = (g.state.objects.len() - 1) as CardId;
    link(&mut g, source, aura);
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    let mana=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==source && production[4]==1)).unwrap();
    replay(&mut g, mana);
    assert_eq!(g.actor(), Some(Player::P1));
    choose_entry(&mut g, bear);
    assert_eq!(g.observe(Player::P0).decision, "pay_costs");
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, pay);
    resolve_all(&mut g);
    assert_eq!(g.stats(bear).power, 7);
}
#[test]
fn attached_entry_subject_uses_departure_lki_and_never_a_returned_target() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Hill Giant");
    let aura = enter_attached(&mut g, Player::P0, "Fae Flight", bear);
    g.prepare_triggers();
    g.move_object(aura, Zone::Hand);
    assert_eq!(g.state.stack[0].source_attachment, Some(g.object_ref(bear)));
    resolve_all(&mut g);
    assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
    let aura = enter_attached(&mut g, Player::P0, "Fire-Rim Form", bear);
    g.prepare_triggers();
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    g.state_based_actions();
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    resolve_all(&mut g);
    assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
}
#[test]
fn colossification_taps_its_attached_creature_and_robotics_creates_complete_tokens() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Hill Giant");
    let aura = funded(&mut g, Player::P0, "Colossification");
    cast(&mut g, aura, Some(Target::Card(bear)));
    resolve_all(&mut g);
    assert_eq!(g.stats(bear).power, 23);
    assert!(g.object_view(bear).tapped);
    let robotics = funded(&mut g, Player::P0, "Robotics Mastery");
    cast(&mut g, robotics, Some(Target::Card(bear)));
    resolve_all(&mut g);
    let robots: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Robot")
        })
        .map(|(id, _)| id as CardId)
        .collect();
    assert_eq!(robots.len(), 2);
    assert!(
        robots
            .iter()
            .all(|&id| g.keywords(id).contains(Keywords::FLYING) && g.stats(id).power == 1)
    );
}
#[test]
fn adventuring_gear_resolves_against_current_attachment_or_departure_lki() {
    let mut g = scenario();
    let bear = permanent(&mut g, "Grizzly Bears");
    let other = permanent(&mut g, "Hill Giant");
    let gear = permanent(&mut g, "Adventuring Gear");
    assert!(g.attach_equipment(g.object_ref(gear), g.object_ref(bear)));
    let land = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.move_object(land, Zone::Battlefield);
    g.prepare_triggers();
    assert!(g.attach_equipment(g.object_ref(gear), g.object_ref(other)));
    g.move_object(gear, Zone::Hand);
    resolve_all(&mut g);
    assert_eq!(g.stats(bear).power, 2);
    assert_eq!(g.stats(other).power, 5);
}

#[test]
fn unattached_aura_uses_graveyard_sba_even_if_indestructible() {
    let mut g = scenario();
    let aura = permanent(&mut g, "Frantic Strength");
    g.state.objects[aura as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    g.state_based_actions();
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
}
#[test]
fn granted_lifelink_belongs_to_creature_controller_not_aura_controller() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    enter_attached(&mut g, Player::P0, "Unflinching Courage", bear);
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(bear),
        target: Target::Player(Player::P0),
        amount: 4,
        combat: true,
    }]);
    assert_eq!(g.state.players[0].life, 16);
    assert_eq!(g.state.players[1].life, 24);
    assert_eq!(g.state.players[1].life_gain_events_this_turn, 1);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 0);
}
#[test]
fn foreign_aura_haste_is_included_in_real_and_planned_mana_legality() {
    let mut g = scenario();
    let druid = permanent(&mut g, "Druid of the Cowl");
    g.state.objects[druid as usize].summoning_sick = true;
    let aura = enter_attached(&mut g, Player::P1, "Super Speed", druid);
    resolve_all(&mut g);
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    assert!(g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==druid && production[4]==1)));
    g.move_object(aura, Zone::Graveyard);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana {card,..} if *card==druid))
    );
}

#[test]
fn failed_linked_return_respects_own_creature_restriction_and_is_not_retried_later() {
    let mut g = scenario();
    let host = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let aura = enter_attached(&mut g, Player::P1, "Aquitect's Defenses", host);
    resolve_all(&mut g);
    let source = named(&mut g, Player::P0, "Banishing Light", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    pass_pair(&mut g);
    permanent(&mut g, "Hill Giant");
    g.state.objects[host as usize].damage = 2;
    g.state_based_actions();
    let removal = funded(&mut g, Player::P0, "Disenchant");
    cast(&mut g, removal, Some(Target::Card(source)));
    pass_pair(&mut g);
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    assert!(g.state.entry_batch.is_none());
    assert!(g.state.exile_links.is_empty());
    named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state_based_actions();
    assert_eq!(g.object_view(aura).zone, Zone::Exile);
    assert_eq!(g.object_view(removal).zone, Zone::Graveyard);
}
