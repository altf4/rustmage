//! A once-per-turn library route belongs to the exact provider and committed cast.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const LIMITED_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LookAtLibraryTop],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CastInstantSorceryFromLibraryTopOnceEachTurn],
    },
];
const LIMITED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: LIMITED_GROUPS,
    extras: &CardExtras {
        static_ability_groups: LIMITED_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const UNLIMITED_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CastFromLibraryTop(
        CastFilter::InstantSorcery,
    )],
}];
const UNLIMITED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: UNLIMITED_GROUPS,
    extras: &CardExtras {
        static_ability_groups: UNLIMITED_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const RED: Mana = [0, 0, 0, 1, 0, 0];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];

fn route(
    object: ObjectRef,
    face: CastFace,
    method: CastMethod,
    permission: LibraryTopCastPermission,
) -> Action {
    Action::AnnounceLibraryTop {
        object,
        face,
        method,
        flash_surcharge: false,
        permission,
    }
}

fn paid_permissions(game: &Game, card: CardId, face: CastFace) -> Vec<LibraryTopCastPermission> {
    let current = game.object_ref(card);
    game.legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::AnnounceLibraryTop {
                object,
                face: chosen,
                method: CastMethod::Normal,
                flash_surcharge: false,
                permission,
            } if object == current && chosen == face => Some(permission),
            _ => None,
        })
        .collect()
}

fn select_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn use_top_shock(game: &mut Game, card: CardId, permission: LibraryTopCastPermission) {
    let opponent = game.actor().unwrap().other();
    let action = route(
        game.object_ref(card),
        CastFace::Normal,
        CastMethod::Normal,
        permission,
    );
    act(game, action);
    select_target(game, Target::Player(opponent));
    pay(game);
    settle(game);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}

#[test]
fn paid_adventure_uses_private_top_and_commits_after_payment_with_owned_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LIMITED)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let next = put(&mut game, player, "Shock", Zone::Library);
        let top = put(&mut game, player, "Frolicking Familiar", Zone::Library);
        let top_object = game.object_ref(top);
        game.state.players[player.index()].mana = [0; 6];
        assert!(paid_permissions(&game, top, CastFace::Adventure).is_empty());
        assert_eq!(
            game.observe(player).known_library_top[player.index()]
                .unwrap()
                .object,
            top_object
        );
        assert!(game.observe(opponent).known_library_top[player.index()].is_none());

        let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
        let available = paid_permissions(&game, top, CastFace::Adventure);
        assert_eq!(available.len(), 1);
        let permission = available[0];
        assert_eq!(permission.source, game.object_ref(provider));
        reject(
            &mut game,
            route(top_object, CastFace::Normal, CastMethod::Normal, permission),
        );
        reject(&mut game, Action::AnnounceAdventure { card: top });
        let action = route(
            top_object,
            CastFace::Adventure,
            CastMethod::Normal,
            permission,
        );

        let before = game.snapshot();
        let revision = game.revision();
        assert_eq!(game.apply(opponent, action.clone()), Err(Error::WrongActor));
        assert_eq!(game.snapshot(), before);
        assert_eq!(game.revision(), revision);
        act(&mut game, action.clone());
        let announcing = game.observe(player);
        assert!(announcing.known_library_top[player.index()].is_none());
        assert!(game.observe(opponent).known_library_top[player.index()].is_none());
        assert_eq!(
            announcing.announcing_library_top_cast_permission,
            Some(permission)
        );
        assert!(announcing.used_library_top_cast_permissions.is_empty());
        select_target(&mut game, Target::Player(opponent));
        reject(&mut game, Action::PayCosts { payment: [0; 6] });

        let mut abandoned = game.clone();
        abandoned.restore(&before).unwrap();
        assert!(abandoned.legal_actions().contains(&action));
        assert!(
            abandoned
                .observe(player)
                .used_library_top_cast_permissions
                .is_empty()
        );
        assert_eq!(abandoned.object_view(top).zone, Zone::Library);

        act(&mut game, Action::TapMana { card: mountain });
        assert!(game.observe(player).known_library_top[player.index()].is_none());
        let payment = Action::PayCosts { payment: RED };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == payment)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, payment);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());

        for branch in [&mut game, &mut indexed] {
            let after = branch.observe(player);
            assert_eq!(
                after.known_library_top[player.index()].unwrap().object,
                branch.object_ref(next)
            );
            assert!(branch.observe(opponent).known_library_top[player.index()].is_none());
            assert_eq!(
                after.used_library_top_cast_permissions,
                [LibraryTopCastUse { player, permission }]
            );
            let cast = branch.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.origin, Zone::Library);
            assert_eq!(cast.face, CastFace::Adventure);
            assert_eq!(cast.payment, RED);
            settle(branch);
            assert_eq!(branch.object_view(top).zone, Zone::Exile);
            assert_eq!(branch.state.players[opponent.index()].life, 19);
            branch.state.players[player.index()].mana = RED;
            assert!(paid_permissions(branch, next, CastFace::Normal).is_empty());
        }
    }
}

#[test]
fn ordinary_route_and_exact_providers_spend_separately_and_new_incarnations_reset() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &LIMITED),
        (Card::from_name("Bear Cub").unwrap(), &UNLIMITED),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let unlimited = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let free_choice = put(&mut game, player, "Shock", Zone::Library);
        assert_eq!(
            paid_permissions(&game, free_choice, CastFace::Normal).len(),
            2
        );
        cast(&mut game, free_choice, Some(Target::Player(opponent)));
        settle(&mut game);
        assert!(
            game.observe(player)
                .used_library_top_cast_permissions
                .is_empty()
        );
        game.move_object(unlimited, Zone::Exile);

        let first_cast = put(&mut game, player, "Shock", Zone::Library);
        let available = paid_permissions(&game, first_cast, CastFace::Normal);
        let first_permission = *available
            .iter()
            .find(|p| p.source == game.object_ref(first))
            .unwrap();
        let second_permission = *available
            .iter()
            .find(|p| p.source == game.object_ref(second))
            .unwrap();
        use_top_shock(&mut game, first_cast, second_permission);
        let second_cast = put(&mut game, player, "Shock", Zone::Library);
        assert_eq!(
            paid_permissions(&game, second_cast, CastFace::Normal),
            [first_permission]
        );
        use_top_shock(&mut game, second_cast, first_permission);

        let later_cast = put(&mut game, player, "Shock", Zone::Library);
        let later_object = game.object_ref(later_cast);
        assert!(paid_permissions(&game, later_cast, CastFace::Normal).is_empty());
        assert_eq!(
            game.observe(player).known_library_top[player.index()]
                .unwrap()
                .object,
            later_object
        );
        game.move_object(second, Zone::Exile);
        game.move_object(second, Zone::Battlefield);
        let renewed = paid_permissions(&game, later_cast, CastFace::Normal);
        assert_eq!(renewed.len(), 1);
        assert_eq!(renewed[0].source, game.object_ref(second));
        assert_ne!(renewed[0].source, second_permission.source);
        reject(
            &mut game,
            route(
                later_object,
                CastFace::Normal,
                CastMethod::Normal,
                second_permission,
            ),
        );
        use_top_shock(&mut game, later_cast, renewed[0]);

        let next = put(&mut game, player, "Shock", Zone::Library);
        put(&mut game, player, "Mountain", Zone::Battlefield);
        assert!(paid_permissions(&game, next, CastFace::Normal).is_empty());
        game.state.step = Step::End;
        game.give_priority();
        pass_pair(&mut game);
        assert_eq!(game.state.active, opponent);
        assert!(
            game.observe(player)
                .used_library_top_cast_permissions
                .is_empty()
        );
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(player));
        assert_eq!(paid_permissions(&game, next, CastFace::Normal).len(), 2);
    }
}

#[test]
fn live_control_and_ability_removal_apply_without_resetting_existing_uses() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LIMITED)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let source = game.object_ref(provider);
        let top = put(&mut game, player, "Shock", Zone::Library);
        let aura = put(&mut game, player, "Witness Protection", Zone::Hand);
        cast(&mut game, aura, Some(Target::Card(provider)));
        settle(&mut game);
        assert!(game.observe(player).known_library_top[player.index()].is_none());
        assert!(paid_permissions(&game, top, CastFace::Normal).is_empty());

        let remove = put(&mut game, player, "Disenchant", Zone::Hand);
        cast(&mut game, remove, Some(Target::Card(aura)));
        settle(&mut game);
        let permission = paid_permissions(&game, top, CastFace::Normal)[0];
        use_top_shock(&mut game, top, permission);
        let own_next = put(&mut game, player, "Shock", Zone::Library);

        game.change_controller(source, opponent);
        let sorcery = put(&mut game, opponent, "Divination", Zone::Library);
        assert!(game.observe(player).known_library_top[player.index()].is_none());
        assert_eq!(
            game.observe(opponent).known_library_top[opponent.index()]
                .unwrap()
                .object,
            game.object_ref(sorcery)
        );
        act(&mut game, Action::Pass);
        assert!(paid_permissions(&game, sorcery, CastFace::Normal).is_empty());
        let wrong_library = route(
            game.object_ref(own_next),
            CastFace::Normal,
            CastMethod::Normal,
            permission,
        );
        reject(&mut game, wrong_library);
        let instant = put(&mut game, opponent, "Shock", Zone::Library);
        assert_eq!(
            paid_permissions(&game, instant, CastFace::Normal),
            [permission]
        );
        use_top_shock(&mut game, instant, permission);

        game.change_controller(source, player);
        assert_eq!(game.object_ref(provider), source);
        assert!(paid_permissions(&game, own_next, CastFace::Normal).is_empty());
        assert_eq!(
            game.observe(player).used_library_top_cast_permissions,
            [
                LibraryTopCastUse { player, permission },
                LibraryTopCastUse {
                    player: opponent,
                    permission
                },
            ]
        );
    }
}

#[test]
fn paid_alternative_and_independent_free_cost_use_the_selected_provider() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &LIMITED)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let source = game.object_ref(provider);
        let victim = put(&mut game, opponent, "Shock", Zone::Hand);
        let top = put(&mut game, player, "Spider-Sense", Zone::Library);
        let mut free = game.clone();
        put(&mut free, player, "Weftwalking", Zone::Battlefield);
        let shock = put(&mut free, player, "Shock", Zone::Library);
        let permission = paid_permissions(&free, shock, CastFace::Normal)[0];
        let action = route(
            free.object_ref(shock),
            CastFace::Normal,
            CastMethod::Free,
            permission,
        );
        act(&mut free, action);
        select_target(&mut free, Target::Player(opponent));
        act(&mut free, Action::PayCosts { payment: [0; 6] });
        assert_eq!(
            free.state.stack.last().unwrap().cast.unwrap().payment,
            [0; 6]
        );
        assert_eq!(
            free.observe(player).used_library_top_cast_permissions,
            [LibraryTopCastUse { player, permission }]
        );

        act(&mut game, Action::Pass);
        cast(&mut game, victim, Some(Target::Player(player)));
        act(&mut game, Action::Pass);
        game.state.objects[provider as usize].tapped = true;
        game.state.players[player.index()].mana = BLUE;
        let web = Action::AnnounceWebSlinging {
            card: top,
            face: CastFace::Normal,
            cost: ManaCost {
                colored: BLUE,
                generic: 0,
            },
            flash_surcharge: false,
            origin: WebSlingingOrigin::LibraryTop { permission },
        };
        act(&mut game, web);
        select_target(&mut game, Target::Spell(victim));
        act(&mut game, Action::BeginOrderedCosts);
        for mana in [true, false] {
            let costs = game
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap();
            let clause = costs
                .clauses
                .iter()
                .position(|row| {
                    !row.paid
                        && if mana {
                            row.clause == CastCostClause::Mana
                        } else {
                            matches!(row.clause, CastCostClause::WebSlingingReturn(_))
                        }
                })
                .unwrap();
            act(
                &mut game,
                Action::ChooseCastCost {
                    clause: clause.try_into().unwrap(),
                },
            );
            if mana {
                act(&mut game, Action::PayCosts { payment: BLUE });
                assert!(
                    game.observe(player)
                        .used_library_top_cast_permissions
                        .is_empty()
                );
            } else {
                act(&mut game, Action::PayWebSlingingReturn { object: source });
            }
        }
        assert_eq!(game.object_view(provider).zone, Zone::Hand);
        assert_eq!(
            game.observe(player).used_library_top_cast_permissions,
            [LibraryTopCastUse { player, permission }]
        );
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].life, 20);
    }
}
