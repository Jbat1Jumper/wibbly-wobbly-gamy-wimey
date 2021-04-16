use crate::common::*;
use crate::fw::Plugin;
use crossbeam_channel::{unbounded, Receiver, Sender};
use glam::f32::Vec2;
use legion::*;
use rapier2d::dynamics::{
    BodyStatus, IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{
    BroadPhase, ColliderBuilder, ColliderHandle, ColliderSet, NarrowPhase, SharedShape,
};
use rapier2d::geometry::{
    ContactEvent as RapierContactEvent, IntersectionEvent as RapierIntersectionEvent,
};
use rapier2d::na::Vector2;
use rapier2d::pipeline::{ChannelEventCollector, PhysicsPipeline};
use std::collections::HashMap;

pub struct PhysicsPlugin;

pub struct PhysicsResources {
    pipeline: PhysicsPipeline,
    gravity: Vector2<f32>,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    event_handler: ChannelEventCollector,
    contact_receive: Receiver<RapierContactEvent>,
    intersection_receive: Receiver<RapierIntersectionEvent>,
    entity_rigidbody_mapping: HashMap<Entity, RigidBodyHandle>,
    entity_rigidbody_was_used: HashMap<Entity, bool>,
    collider_entity_mapping: HashMap<ColliderHandle, Entity>,
}

impl Default for PhysicsResources {
    fn default() -> PhysicsResources {
        let (intersection_send, intersection_receive) = unbounded();
        let (contact_send, contact_receive) = unbounded();
        PhysicsResources {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector2::new(0.0, 0.0),
            integration_parameters: IntegrationParameters::default(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            // We ignore physics hooks for now.
            // let physics_hooks = ();
            event_handler: ChannelEventCollector::new(intersection_send, contact_send),
            contact_receive,
            intersection_receive,
            collider_entity_mapping: HashMap::new(),
            entity_rigidbody_mapping: HashMap::new(),
            entity_rigidbody_was_used: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ContactEvent {
    Started(Entity, Entity),
    Stopped(Entity, Entity),
}

impl ContactEvent {
    fn map(
        ce: &RapierContactEvent,
        mapping: &HashMap<ColliderHandle, Entity>,
    ) -> Vec<ContactEvent> {
        match ce {
            RapierContactEvent::Started(a, b) => vec![
                ContactEvent::Started(*mapping.get(a).unwrap(), *mapping.get(b).unwrap()),
                ContactEvent::Started(*mapping.get(b).unwrap(), *mapping.get(a).unwrap()),
            ],
            RapierContactEvent::Stopped(a, b) => vec![
                ContactEvent::Stopped(*mapping.get(a).unwrap(), *mapping.get(b).unwrap()),
                ContactEvent::Stopped(*mapping.get(b).unwrap(), *mapping.get(a).unwrap()),
            ],
        }
    }
}

struct IntersectionEvent;

impl Plugin for PhysicsPlugin {
    fn name(&self) -> String {
        "PhysicsPlugin".into()
    }

    fn init(&mut self, world: &mut World, resources: &mut Resources) {
        let (sender, receiver) = unbounded::<ContactEvent>();
        resources.insert(sender);
        resources.insert(receiver);

        resources.insert(PhysicsResources::default());
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) {
        Schedule::builder()
            .add_system(create_rigidbodies_system())
            .add_system(simulation_step_system())
            .add_system(sync_rigidbodies_system())
            .add_system(pipe_events_system())
            .add_system(mark_used_bodies_system())
            .add_system(remove_unused_bodies_system())
            .build()
            .execute(world, resources);
    }

    fn draw(&mut self, world: &World, resources: &Resources) {
        debug_render_rigidbodies(world, resources);
    }

    fn load_scene(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        scene: SceneRef,
    ) -> Option<Schedule> {
        // Override all physics resources on scene change because
        // scene changes removes all handles to the rigidbodies
        println!("Load {:?} from physics plugin", scene);
        resources.insert(PhysicsResources::default());
        None
    }
}

#[system]
fn pipe_events(
    #[resource] physics: &mut PhysicsResources,
    #[resource] receiver: &mut Receiver<ContactEvent>,
    #[resource] sender: &mut Sender<ContactEvent>,
) {
    // Drain the master receiver so it does not get clogged
    for _ in receiver.try_iter() {}

    for ie in physics.intersection_receive.try_iter() {}
    for ce in physics.contact_receive.try_iter() {
        let events = ContactEvent::map(&ce, &physics.collider_entity_mapping);
        for ce in events {
            sender.send(ce);
        }
    }
}

#[system(for_each)]
#[filter(component::<RigidBody2D>())]
fn mark_used_bodies(
    entity: &Entity,
    #[resource] physics: &mut PhysicsResources,
) {
    physics.entity_rigidbody_was_used.insert(*entity, true);
}

#[system]
fn remove_unused_bodies(#[resource] physics: &mut PhysicsResources) {
    println!("Called to remove deleted bodies");
    let entities_to_delete_rb = physics
        .entity_rigidbody_was_used
        .iter()
        .filter_map(|(&e, &used)| if !used { Some(e) } else { None })
        .collect::<Vec<_>>();

    for e in entities_to_delete_rb.iter() {
        if let Some(&rbh) = physics.entity_rigidbody_mapping.get(e) {
            println!("Deleted {:?} with rigidbody handle {:?}", e, rbh);
            if let Some(rb) =
                physics
                    .bodies
                    .remove(rbh, &mut physics.colliders, &mut physics.joints)
            {
                for ch in rb.colliders() {
                    physics.collider_entity_mapping.remove(ch);
                }
            }
            physics.entity_rigidbody_mapping.remove(e);
            physics.entity_rigidbody_was_used.remove(e);
        }
    }

    for (_, was_used) in physics.entity_rigidbody_was_used.iter_mut() {
        *was_used = false;
    }
}

#[system]
fn simulation_step(#[resource] physics: &mut PhysicsResources) {
    println!("Called to simulation_step");
    physics.pipeline.step(
        &physics.gravity,
        &physics.integration_parameters,
        &mut physics.broad_phase,
        &mut physics.narrow_phase,
        &mut physics.bodies,
        &mut physics.colliders,
        &mut physics.joints,
        &(),
        &physics.event_handler,
    );
}

#[system(for_each)]
fn sync_rigidbodies(
    entity: &Entity,
    rigidbody: &RigidBody2D,
    position: &mut Position,
    #[resource] physics: &mut PhysicsResources,
) {
    println!("Called to sync rigidbodies on {:?}", entity);
    if let Some((rbh, _ch)) = rigidbody.handles {
        if let Some(rb) = physics.bodies.get_mut(rbh).or_else(|| {
            println!("Error when accesing handle: {:?}", rbh);
            None
        }) {
            let t = rb.position().translation.vector;
            position.0 = Vec2::new(t.x, t.y);
        }
    }
}

#[system(for_each)]
fn create_rigidbodies(
    entity: &Entity,
    rigidbody: &mut RigidBody2D,
    Position(p): &Position,
    #[resource] physics: &mut PhysicsResources,
) {
    println!("Called to create rigidbody on {:?}", entity);
    if rigidbody.handles.is_none() {
        let body_kind = if rigidbody.is_static {
            BodyStatus::Static
        } else {
            BodyStatus::Dynamic
        };
        let rb = RigidBodyBuilder::new(body_kind)
            .translation(p.x, p.y)
            .lock_rotations()
            .linear_damping(8.0)
            .can_sleep(true)
            .build();
        let rbh = physics.bodies.insert(rb);
        let c = ColliderBuilder::new(map_shape(&rigidbody.shape))
            .friction(0.8)
            .build();
        let ch = physics.colliders.insert(c, rbh, &mut physics.bodies);
        physics.collider_entity_mapping.insert(ch, entity.clone());
        physics.entity_rigidbody_mapping.insert(entity.clone(), rbh);
        physics
            .entity_rigidbody_was_used
            .insert(entity.clone(), true);
        rigidbody.handles = Some((rbh, ch));
    }
}

fn map_shape(s: &Shape) -> SharedShape {
    match *s {
        Shape::Circle(r) => SharedShape::ball(r),
        Shape::AABB(w, h) => SharedShape::cuboid(w, h),
    }
}

pub struct RigidBody2D {
    pub handles: Option<(RigidBodyHandle, ColliderHandle)>,
    pub shape: Shape,
    pub is_static: bool,
}

impl RigidBody2D {
    pub fn new(shape: Shape, is_static: bool) -> Self {
        RigidBody2D {
            handles: None,
            shape,
            is_static,
        }
    }
}

pub enum Shape {
    Circle(f32),
    AABB(f32, f32),
}

pub struct DrawRigidbodyFlag;

fn debug_render_rigidbodies(world: &World, resources: &Resources) {}
