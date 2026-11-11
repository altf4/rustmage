//! Exact linked cast permissions and frozen payment/entry facts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

const EXILE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 2,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
    program: &[Instruction::ExileGraveTargetsLinked],
    sorcery: false,
    once_per_turn: false,
};

const fn permission(subtype: &'static str) -> StaticAbilityFragment {
    StaticAbilityFragment::LinkedExilePaidCast {
        filter: CastFilter::CreatureSubtype(subtype),
        entry_counter: Some(CounterKind::Finality),
    }
}

const LINKER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[EXILE],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[permission("Dinosaur")],
    }],
    ..CardRules::EMPTY
};

const SPIDER_LINKER: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[permission("Spider")],
    }],
    ..LINKER
};

fn linked(game: &Game, spell: CardId, source: CardId, face: CastFace) -> Action {
    Action::AnnounceLinkedPaid {
        object: game.object_ref(spell),
        source: game.object_ref(source),
        face,
        flash_surcharge: false,
    }
}

fn link(game: &mut Game, spell: CardId, source: CardId) {
    game.link_exile_to_ability(game.object_ref(source), None, game.object_ref(spell));
}

fn costs(game: &mut Game) {
    for _ in 0..24 {
        if game.state.announcing.is_none() {
            return;
        }
        choose(game, |a| {
            matches!(
                a,
                Action::PayCosts { .. }
                    | Action::FinishTargets
                    | Action::BeginOrderedCosts
                    | Action::ChooseCastCost { .. }
            )
        });
    }
    panic!("linked spell did not complete payment");
}

#[test]
fn link_uses_current_control_and_exact_provider_and_card_incarnations() {
    let _rules = fixture_rules::activate(&[(card("Bear Cub"), &LINKER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let other_source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let owned = put(&mut g, p, "Belligerent Yearling", Zone::Graveyard);
    let foreign = put(&mut g, p.other(), "Belligerent Yearling", Zone::Graveyard);
    let wrong_type = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    for id in [owned, foreign, wrong_type] {
        let target = Target::Card(id);
        let direct = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate { card, ability: 0, target: Some(chosen), .. } if *card == source && *chosen == target));
        if let Some(a) = direct {
            act(&mut g, a);
        } else {
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(&mut g, Action::ChooseTarget { target });
            costs(&mut g);
        }
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
    let wanted = linked(&g, owned, source, CastFace::Normal);
    assert!(g.legal_actions().contains(&wanted));
    for invalid in [
        linked(&g, owned, other_source, CastFace::Normal),
        linked(&g, foreign, source, CastFace::Normal),
        linked(&g, wrong_type, source, CastFace::Normal),
    ] {
        reject(&mut g, invalid);
    }
    g.state.step = Step::Upkeep;
    reject(&mut g, wanted.clone());
    g.state.step = Step::Main1;
    let mut changed = g.clone();
    changed.record_control(changed.object_ref(source), p.other(), None);
    reject(&mut changed, wanted.clone());
    changed.state.active = p.other();
    changed.state.priority = p.other();
    assert!(
        changed
            .legal_actions()
            .contains(&linked(&changed, foreign, source, CastFace::Normal))
    );
    reject(&mut changed, wanted.clone());

    let mut new_card = g.clone();
    new_card.move_object(owned, Zone::Graveyard);
    new_card.move_object(owned, Zone::Exile);
    link(&mut new_card, owned, source);
    reject(&mut new_card, wanted.clone());
    assert!(
        new_card
            .legal_actions()
            .contains(&linked(&new_card, owned, source, CastFace::Normal))
    );
    g.move_object(source, Zone::Graveyard);
    g.move_object(source, Zone::Battlefield);
    reject(&mut g, wanted);
    let newer = linked(&g, owned, source, CastFace::Normal);
    reject(&mut g, newer);
}

#[test]
fn free_and_limited_permissions_keep_separate_payment_and_entry_consequences() {
    let _rules = fixture_rules::activate(&[(card("Bear Cub"), &LINKER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Belligerent Yearling", Zone::Exile);
    link(&mut g, spell, source);
    let object = g.object_ref(spell);
    let paid = linked(&g, spell, source, CastFace::Normal);
    g.grant_exile_free_permission(p, object);
    let lender = g.activation_item(p, source, 0);
    g.grant_linked_exile_play(&lender);
    let saved = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == paid).unwrap();
    let mut indexed = g.clone();
    let mut independent = g.clone();
    act(&mut g, paid.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(g.snapshot(), indexed.snapshot());
    assert!(g.observe(p).cast_entry_counters.is_empty());
    costs(&mut g);
    costs(&mut indexed);
    assert_eq!(g.snapshot(), indexed.snapshot());
    assert_eq!(g.observe(p).cast_entry_counters.len(), 1);
    assert_eq!(g.state.stack.last().unwrap().cast.unwrap().payment[3], 1);
    assert_eq!(
        g.state.linked_choices.as_ref().unwrap().play_grants[0].remaining,
        1
    );
    settle(&mut g);
    assert_eq!(g.counter_count(spell, CounterKind::Finality), 1);
    g.move_object(spell, Zone::Graveyard);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);

    act(
        &mut independent,
        Action::AnnounceExileFree {
            card: spell,
            face: CastFace::Normal,
            flash_surcharge: false,
        },
    );
    costs(&mut independent);
    settle(&mut independent);
    assert_eq!(independent.counter_count(spell, CounterKind::Finality), 0);

    indexed.restore(&saved).unwrap();
    let ordinary = indexed
        .grant_exile_play_permission_with_expiry(p, object, None, ExilePlayExpiry::WhileExiled)
        .unwrap();
    ordinary.spend_any_mana = true;
    ordinary.exile_on_graveyard = true;
    indexed.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
    reject(&mut indexed, paid.clone());
    indexed.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
    act(&mut indexed, paid);
    costs(&mut indexed);
    let receipt = indexed.state.stack.last().unwrap().cast.unwrap();
    assert!(!receipt.spend_any_mana);
    assert!(!receipt.exile_on_graveyard);
}

#[test]
fn chosen_faces_x_and_flash_use_the_proposed_spell() {
    let p = Player::P0;
    {
        let _rules = fixture_rules::activate(&[(card("Bear Cub"), &SPIDER_LINKER)]);
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Gwen Stacy // Ghost-Spider", Zone::Exile);
        link(&mut g, spell, source);
        let front = linked(&g, spell, source, CastFace::Normal);
        let back = linked(&g, spell, source, CastFace::Back);
        reject(&mut g, front);
        act(&mut g, back);
        costs(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).name, "Ghost-Spider");
        assert_eq!(g.counter_count(spell, CounterKind::Finality), 1);
    }
    {
        let _rules = fixture_rules::activate(&[(card("Bear Cub"), &LINKER)]);
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let flash = put(&mut g, p, "Flock Impostor", Zone::Exile);
        let spell = put(&mut g, p, "Pterafractyl", Zone::Exile);
        link(&mut g, flash, source);
        link(&mut g, spell, source);
        g.state.step = Step::Upkeep;
        assert!(
            g.legal_actions()
                .contains(&linked(&g, flash, source, CastFace::Normal))
        );
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 2];
        let cast = linked(&g, spell, source, CastFace::Normal);
        act(&mut g, cast);
        let domain = g.x_choice().unwrap();
        assert!(domain.contains(2));
        assert!(!domain.contains(3));
        act(&mut g, Action::ChooseX { value: 2 });
        costs(&mut g);
        for _ in 0..18 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert_eq!(g.counter_count(spell, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(spell, CounterKind::Finality), 1);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}

const fn sacrifice_red(amount: u32) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: if amount == 1 {
            &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 1, 0, 0,
            ]))]
        } else {
            &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 2, 0, 0,
            ]))]
        },
        ..EXILE
    }
}
const TWO_RED: CardRules = CardRules {
    activated: &[EXILE, sacrifice_red(2)],
    ..LINKER
};
const ONE_RED: CardRules = CardRules {
    activated: &[EXILE, sacrifice_red(1)],
    ..LINKER
};

#[test]
fn sole_funding_path_can_sacrifice_the_provider_and_keep_the_finality_receipt() {
    let _rules = fixture_rules::activate(&[(card("Bear Cub"), &TWO_RED)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Belligerent Yearling", Zone::Exile);
    link(&mut g, spell, source);
    g.state.players[p.index()].mana = [0; 6];
    let cast = linked(&g, spell, source, CastFace::Normal);
    act(&mut g, cast);
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana { card, ability: 1, .. } if *card == source),
    );
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    costs(&mut g);
    settle(&mut g);
    assert_eq!(g.counter_count(spell, CounterKind::Finality), 1);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

const OPTIONAL_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 17,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn foreign_mana_entry_rewinds_the_cast_and_exact_link_before_retention() {
    let _rules = fixture_rules::activate(&[
        (card("Bear Cub"), &ONE_RED),
        (card("Grizzly Bears"), &OPTIONAL_COPY),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let green = put(&mut g, p, "Druid of the Cowl", Zone::Battlefield);
    let spell = put(&mut g, p, "Belligerent Yearling", Zone::Exile);
    link(&mut g, spell, source);
    let reference = g.object_ref(spell);
    let enchanted = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
    let blueprint = g.object_ref(aura);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(enchanted));
    g.change_object_form(
        blueprint,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    let returning = put(&mut g, p.other(), "Grizzly Bears", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: g.object_ref(source),
        exiled: g.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    g.state.players[p.index()].mana = [0; 6];
    let cast = linked(&g, spell, source, CastFace::Normal);
    act(&mut g, cast.clone());
    assert_eq!(g.state.cast_rollback.as_ref().unwrap().spell, spell);
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana { card, ability: 1, .. } if *card == source),
    );
    assert_eq!(g.actor(), Some(p.other()));
    act(
        &mut g,
        Action::ChooseEntryCopy {
            object: Some(blueprint),
        },
    );
    let green = g.object_ref(green);
    act(&mut g, Action::ChooseAuraAttachment { object: green });
    assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
    assert_eq!(g.object_ref(spell), reference);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert!(g.observe(p).cast_entry_counters.is_empty());
    let saved = g.snapshot();
    for keep in [false, true] {
        let mut raw = g.clone();
        let mut indexed = g.clone();
        indexed.restore(&saved).unwrap();
        let action = Action::ChooseManaRollback { keep };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let revision = indexed.revision();
        act(&mut raw, action);
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_ref(spell), reference);
        assert_eq!(raw.legal_actions().contains(&cast), !keep);
        assert!(raw.observe(p).cast_entry_counters.is_empty());
    }
}
