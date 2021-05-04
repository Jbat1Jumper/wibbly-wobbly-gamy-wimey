use bevy::prelude::*;
use std::collections::HashSet;

pub struct PlainSimplePhysicsPlugin;

impl Plugin for PlainSimplePhysicsPlugin {
    fn build(&self, application: &mut AppBuilder) {
        application
            .insert_resource(CurrentContacts {
                set: HashSet::new(),
            })
            .add_event::<ContactEvent>()
            .add_system(add_next_pos_components.system())
            .add_system(calculate_movement.system())
            .add_system(update_transforms.system());
    }
}

#[derive(Clone, Debug, Default)]
pub struct RigidBody {
    pub velocity: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub enum Collider {
    AABB(Vec2),
}

fn distance(pos_a: Vec2, col_a: Collider, pos_b: Vec2, col_b: Collider) -> f32 {
    let Collider::AABB(half_size_a) = col_a;
    let Collider::AABB(half_size_b) = col_b;

    let a = pos_a - half_size_a;
    let b = pos_a + half_size_a;
    let c = pos_b - half_size_b;
    let d = pos_b + half_size_b;

    // This is not norm2 distance, but is enough for my needs now
    let d_h = 0f32.max(a.x - d.x).max(c.x - b.x);
    let d_v = 0f32.max(a.y - d.y).max(c.y - b.y);
    return d_h + d_v;
}

fn intersects(pos_a: Vec2, col_a: Collider, pos_b: Vec2, col_b: Collider) -> bool {
    let Collider::AABB(half_size_a) = col_a;
    let Collider::AABB(half_size_b) = col_b;

    let a = pos_a - half_size_a;
    let b = pos_a + half_size_a;
    let c = pos_b - half_size_b;
    let d = pos_b + half_size_b;

    d.x > a.x && b.x > c.x && d.y > a.y && b.y > c.y
}

#[derive(Clone, Copy, Debug)]
pub enum ContactEvent {
    Started(Entity, Entity),
    Ended(Entity, Entity),
}

#[derive(Clone, Debug)]
pub struct CurrentContacts {
    set: HashSet<(Entity, Entity)>,
}

#[derive(Clone, Debug)]
pub struct MoveTransform(Vec2);

fn add_next_pos_components(
    mut commands: Commands,
    query: Query<Entity, (With<RigidBody>, With<Transform>, Without<MoveTransform>)>,
) {
    for e in query.iter() {
        commands.entity(e).insert(MoveTransform(Vec2::ZERO));
    }
}

// TODO: Use constant delta with multiple iterations
fn calculate_movement(
    query: Query<(Entity, &RigidBody, &Collider, &Transform)>,
    mut move_query: Query<&mut MoveTransform>,
    mut contacts: ResMut<CurrentContacts>,
    mut contact_events: EventWriter<ContactEvent>,
    time: Res<Time>,
) {
    let mut new_contacts = HashSet::new();
    let mut would_collide_if_move = HashSet::new();
    for (e1, rb1, c1, t1) in query.iter() {
        let fp1 = t1.translation.truncate() + rb1.velocity * time.delta_seconds();
        for (e2, rb2, c2, t2) in query.iter() {
            if e1 == e2 {
                continue;
            }
            let fp2 = t2.translation.truncate() + rb2.velocity * time.delta_seconds();
            if intersects(fp1, *c1, fp2, *c2) {
                new_contacts.insert((e1, e2));
                would_collide_if_move.insert(e1);
            }
        }
    }

    for c in new_contacts.iter() {
        if !contacts.set.contains(c) {
            contact_events.send(ContactEvent::Started(c.0, c.1));
        }
        contacts.set.insert(*c);
    }

    let ended_contacts: Vec<_> = contacts
        .set
        .iter()
        .filter(|c| {
            !new_contacts.contains(c)
                && query.get(c.0).map_or(true, |(_, _, c1, t1)| {
                    query.get(c.1).map_or(true, |(_, _, c2, t2)| {
                        distance(
                            t1.translation.truncate(),
                            *c1,
                            t2.translation.truncate(),
                            *c2,
                        ) > 0.4
                    })
                })
        })
        .cloned()
        .collect();
    for c in ended_contacts {
        contact_events.send(ContactEvent::Ended(c.0, c.1));
        contacts.set.remove(&c);
    }

    for (e, rb, _, _) in query.iter() {
        if !would_collide_if_move.contains(&e) {
            if let Ok(mut mt) = move_query.get_mut(e) {
                mt.0 = rb.velocity * time.delta_seconds()
            };
        }
    }
}

// TODO: Update global transform at the same time
// TODO: Filter out static bodies in the query
fn update_transforms(mut query: Query<(&mut MoveTransform, &mut Transform)>) {
    for (mut d, mut t) in query.iter_mut() {
        if d.0.length_squared() > 0.001 {
            t.translation += d.0.extend(0.);
            d.0 = Vec2::ZERO;
        }
    }
}
