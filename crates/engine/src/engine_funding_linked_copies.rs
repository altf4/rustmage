//! Exact source-bound copy expiry inside borrowed cost worlds, without entries.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl FundingState<'_> {
    pub(super) fn refresh_source_bound_copies(&mut self) {
        let Some(rows) = &self.game.state.temporary_copies else {
            return;
        };
        let changes = rows
            .iter()
            .filter(|row| {
                self.object(row.object.card).zone == Zone::Battlefield
                    && self.reference(row.object.card) == row.object
                    && row
                        .layers
                        .iter()
                        .any(|layer| layer.value.deadline.source().is_some())
            })
            .filter_map(|row| {
                let active = row.layers.iter().rev().find(|layer| {
                    layer.value.deadline.source().is_none_or(|source| {
                        self.object(source.card).zone == Zone::Battlefield
                            && self.reference(source.card) == source
                    })
                });
                let origin = active.map(|layer| layer.value.origin);
                (self.ability_origin(row.object.card) != origin)
                    .then(|| (row.clone(), active.cloned()))
            })
            .collect::<Vec<_>>();
        if changes.is_empty() {
            return;
        }
        self.prepare_entry_projection();
        for (row, active) in changes {
            let source = row.object;
            let physical = row
                .original_recipe
                .unwrap_or_else(|| CopyRecipe::new(row.physical_card));
            let recipe = active
                .as_ref()
                .map(|layer| {
                    layer
                        .value
                        .recipe
                        .with_copy_effect_faces(physical.face_structure())
                })
                .or(row.original_recipe);
            let definition = active
                .as_ref()
                .map_or(row.physical_card, |layer| layer.value.recipe.definition);
            let object = &mut self
                .resources
                .iter_mut()
                .find(|r| r.id == source.card)
                .unwrap()
                .object;
            object.card = definition;
            object.copy_kind = if active.is_some() {
                if row.physical_kind == CopyKind::Token || row.physical_card.is_token() {
                    CopyKind::Token
                } else {
                    CopyKind::Physical
                }
            } else {
                row.physical_kind
            };
            object.used_abilities = active
                .as_ref()
                .map_or(row.original_used, |layer| layer.used_abilities);
            if let Some(rows) = &mut self.copy_recipes {
                rows.retain(|r| r.object != source);
            }
            if let Some(recipe) = recipe {
                let rows = self.copy_recipes.get_or_insert_default();
                let at = rows.partition_point(|r| r.object.card < source.card);
                rows.insert(
                    at,
                    copies::CopyRow {
                        object: source,
                        recipe,
                    },
                );
            }
            if self.copy_recipes.as_ref().is_some_and(|r| r.is_empty()) {
                self.copy_recipes = None;
            }
            let p = self.entry_projection.as_mut().unwrap();
            p.copy_origins.retain(|(r, _)| *r != source);
            p.copy_origins
                .push((source, active.as_ref().map(|layer| layer.value.origin)));
            p.copy_origins.sort_by_key(|(r, _)| *r);
            p.static_sources.retain(|&id| id != source.card);
            p.attachment_sources.retain(|&id| id != source.card);
            p.roles
                .base_stats
                .retain(|r| !(r.static_source && r.source == source));
            p.roles.live_static.retain(|r| r.source != source);
            p.roles.ability_timestamps.retain(|r| r.source != source);
            p.roles.ability_removal_sources.retain(|r| *r != source);
            p.roles.characteristic_sources.retain(|r| *r != source);
            let time = active
                .as_ref()
                .map(|l| l.value.provider_timestamp)
                .or(row.original_provider_time);
            p.register_at(source, definition, time.unwrap_or(0));
            p.hand_size.forget(source);
            if let Some(maximum) = definition.hand_size_setting()
                && let Some(timestamp) = active
                    .as_ref()
                    .map_or(row.original_hand_time, |l| l.value.hand_timestamp)
            {
                p.hand_size.sources.push(hand_size::HandSizeSourceView {
                    source,
                    timestamp,
                    maximum,
                });
                p.hand_size.sources.sort_by_key(|r| r.source);
            }
        }
    }
}
