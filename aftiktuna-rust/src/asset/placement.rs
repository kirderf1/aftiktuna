use super::model::{Model, ModelAccess, Offsets};
use crate::core::position::{Coord, Direction};
use crate::view::area::ObjectRenderData;
use std::collections::HashMap;
use std::mem;

pub type Vec2 = (f32, f32);

// Coordinates are mapped like this so that when the left edge of the window is 0,
// coord 3 will be placed in the middle of the window.
pub fn coord_to_center_x(coord: Coord) -> f32 {
    40. + 120. * coord as f32
}

pub fn position_objects<T>(
    objects: &[ObjectRenderData],
    models: &mut impl ModelAccess<T>,
) -> Vec<(Vec2, ObjectRenderData)> {
    let mut positioned_objects = Vec::new();
    let mut positioner = Positioner::default();
    let mut groups_cache: Vec<Vec<ObjectRenderData>> =
        vec![Vec::new(); (objects.iter().map(|obj| obj.coord).max().unwrap_or(0) + 1) as usize];

    let mut objects = objects.to_owned();
    objects.sort_by(|data1, data2| {
        let weight1 = models.lookup_model(&data1.model_id).order_weight;
        let weight2 = models.lookup_model(&data2.model_id).order_weight;
        weight2
            .cmp(&weight1)
            .then(data1.is_controlled.cmp(&data2.is_controlled))
    });

    for data in objects {
        let object_group = &mut groups_cache[data.coord as usize];
        if models.lookup_model(&data.model_id).large_displacement {
            positioned_objects
                .extend(positioner.position_object_group(mem::take(object_group), models));
            if let Some(object_group) = data
                .coord
                .checked_add_signed(data.properties.direction.opposite().into())
                .and_then(|coord| groups_cache.get_mut(coord as usize))
            {
                positioned_objects
                    .extend(positioner.position_object_group(mem::take(object_group), models));
            }
            positioned_objects.push((
                positioner.position_object(
                    data.coord,
                    data.properties.direction,
                    models.lookup_model(&data.model_id),
                ),
                data,
            ));
        } else {
            if object_group
                .first()
                .is_some_and(|cached_object| cached_object.model_id != data.model_id)
            {
                positioned_objects
                    .extend(positioner.position_object_group(mem::take(object_group), models));
            }

            object_group.push(data);
        }
    }

    for object_group in groups_cache {
        positioned_objects.extend(positioner.position_object_group(object_group, models));
    }

    positioned_objects.sort_by(|((_, z1), data1), ((_, z2), data2)| {
        let weight1 = models.lookup_model(&data1.model_id).order_weight;
        let weight2 = models.lookup_model(&data2.model_id).order_weight;
        z1.cmp(z2)
            .then(weight2.cmp(&weight1))
            .then(data1.is_controlled.cmp(&data2.is_controlled))
            .then(data1.coord.cmp(&data2.coord))
    });
    positioned_objects
        .into_iter()
        .map(|((pos, _), data)| (pos, data))
        .collect()
}

#[derive(Default)]
pub struct Positioner {
    coord_counts: HashMap<Coord, (u16, i16)>,
}

impl Positioner {
    fn position_object_group<T>(
        &mut self,
        object_group: Vec<ObjectRenderData>,
        models: &mut impl ModelAccess<T>,
    ) -> Vec<((Vec2, i16), ObjectRenderData)> {
        if let Some((coord, direction, model)) = object_group.first().map(|object| {
            (
                object.coord,
                object.properties.direction,
                models.lookup_model(&object.model_id),
            )
        }) {
            self.position_groups_from_offsets(
                model.group_placement.position(object_group.len() as u16),
                coord,
                direction,
                model,
            )
            .into_iter()
            .zip(object_group)
            .collect()
        } else {
            Vec::default()
        }
    }

    pub fn position_groups_from_offsets<T>(
        &mut self,
        offset_groups: Vec<Offsets>,
        coord: Coord,
        direction: Direction,
        model: &Model<T>,
    ) -> Vec<(Vec2, i16)> {
        offset_groups
            .into_iter()
            .flat_map(|offsets| {
                let ((x, y), z) = self.position_object(coord, direction, model);
                offsets
                    .into_iter()
                    .map(move |offset| ((x + f32::from(offset.0), y + f32::from(offset.1)), z))
            })
            .collect()
    }

    pub fn position_object<T>(
        &mut self,
        coord: Coord,
        direction: Direction,
        model: &Model<T>,
    ) -> (Vec2, i16) {
        let (x_count, z_displacement) = if model.large_displacement
            && let Some(coord2) = coord.checked_add_signed(direction.opposite().into())
        {
            self.calculate_displacement(&[coord, coord2], model)
        } else {
            self.calculate_displacement(&[coord], model)
        };

        let pos = position_from_coord(coord, x_count, z_displacement, model.z_offset);
        (pos, -pos.1 as i16)
    }

    fn calculate_displacement<T>(&mut self, range: &[Coord], model: &Model<T>) -> (u16, i16) {
        let (x_count, z_displacement) = range
            .iter()
            .map(|coord| self.coord_counts.get(coord).copied().unwrap_or_default())
            .fold(
                (0, 0),
                |(x_count1, z_displacement1), (x_count2, z_displacement2)| {
                    (x_count1.max(x_count2), z_displacement1.max(z_displacement2))
                },
            );

        let updated_x_count = x_count + if model.has_x_displacement { 1 } else { 0 };
        let updated_z_displacement = z_displacement + model.z_displacement;

        for &coord in range {
            self.coord_counts
                .insert(coord, (updated_x_count, updated_z_displacement));
        }

        (x_count, z_displacement)
    }
}

fn position_from_coord(
    coord: Coord,
    x_displacement_count: u16,
    z_displacement: i16,
    z_offset: i16,
) -> Vec2 {
    (
        coord_to_center_x(coord) - f32::from(x_displacement_count * 15),
        f32::from(190 - z_displacement - z_offset),
    )
}
