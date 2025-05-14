use std::collections::BTreeMap;

use bevy_picking::backend::prelude::*;

use crate::{cursor::PxCursorPosition, math::RectExt, prelude::*, set::PxSet};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_systems(PostUpdate, pick::<L>.in_set(PxSet::Picking));
}

// TODO Pick other entities in a generic way
// TODO Other pointers support
fn pick<L: PxLayer>(
    mut hits: EventWriter<PointerHits>,
    pointers: Query<&PointerId>,
    rects: Query<(
        &PxRect,
        &PxFilterLayers<L>,
        &PxPosition,
        &PxAnchor,
        &PxCanvas,
        &InheritedVisibility,
        Entity,
    )>,
    cursor: Res<PxCursorPosition>,
    px_camera: Res<PxCamera>,
    cameras: Query<(&Camera, Entity)>,
) {
    let Some(cursor) = **cursor else {
        return;
    };
    let cursor = cursor.as_ivec2();

    let Ok((camera, camera_id)) = cameras.single() else {
        return;
    };

    let cam_pos = **px_camera;

    for &pointer in &pointers {
        let PointerId::Mouse = pointer else {
            continue;
        };

        let mut layer_depths = BTreeMap::new();

        hits.write(PointerHits {
            pointer,
            picks: rects
                .iter()
                .filter_map(|(&rect, layer, &pos, &anchor, canvas, visibility, id)| {
                    if !visibility.get() {
                        return None;
                    }

                    let layer = match layer {
                        PxFilterLayers::Single { layer, .. } => Some(layer),
                        PxFilterLayers::Many(layers) => layers.iter().max(),
                        // TODO Can't pick rects with this variant
                        PxFilterLayers::Range(range) => Some(range.end()),
                    }?;

                    let depth = if let Some(&depth) = layer_depths.get(layer) {
                        depth
                    } else {
                        let depth = match (
                            layer_depths.range(..layer).last(),
                            layer_depths.range(layer..).next(),
                        ) {
                            (Some((_, &lower)), Some((_, &upper))) => (lower + upper) / 2.,
                            (Some((_, &lower)), None) => lower - 1.,
                            (None, Some((_, &upper))) => upper + 1.,
                            (None, None) => 0.,
                        };

                        // R-A workaround
                        BTreeMap::insert(&mut layer_depths, layer.clone(), depth);
                        depth
                    };

                    // TODO This is duplicated from `draw_spatial`
                    let size = *rect;
                    let position = *pos - anchor.pos(size).as_ivec2();
                    let position = match canvas {
                        PxCanvas::World => position - cam_pos,
                        PxCanvas::Camera => position,
                    };

                    IRect {
                        min: position,
                        max: position.saturating_add(size.as_ivec2()),
                    }
                    .contains_exclusive(cursor)
                    .then_some((
                        id,
                        HitData {
                            camera: camera_id,
                            depth,
                            position: None,
                            normal: None,
                        },
                    ))
                })
                .collect(),
            order: camera.order as f32,
        });
    }
}
