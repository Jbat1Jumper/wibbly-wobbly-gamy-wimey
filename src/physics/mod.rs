use crate::common::*;
use crate::fw::Plugin;
use glam::f32::Vec2;
use legion::*;

use rapier2d::dynamics::{
    BodyStatus, IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{
    BroadPhase, ColliderBuilder, ColliderHandle, ColliderSet, NarrowPhase, SharedShape,
};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;

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
}

impl Default for PhysicsResources {
    fn default() -> PhysicsResources {
        PhysicsResources {
            pipeline: PhysicsPipeline::new(),
            gravity: Vector2::new(0.0, 0.0),
            integration_parameters: IntegrationParameters::default(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            // We ignore physics hooks and contact events for now.
            // let physics_hooks = ();
            // let event_handler = ();
        }
    }
}

impl Plugin for PhysicsPlugin {
    fn init(&mut self, _world: &mut World, resources: &mut Resources) {
        resources.insert(PhysicsResources::default());
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) {
        Schedule::builder()
            .add_system(create_rigidbodies_system())
            .add_system(simulation_step_system())
            .add_system(sync_rigidbodies_system())
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
        resources.insert(PhysicsResources::default());
        None
    }
}

#[system]
fn simulation_step(#[resource] physics: &mut PhysicsResources) {
    physics.pipeline.step(
        &physics.gravity,
        &physics.integration_parameters,
        &mut physics.broad_phase,
        &mut physics.narrow_phase,
        &mut physics.bodies,
        &mut physics.colliders,
        &mut physics.joints,
        &(),
        &(),
    );
}

#[system(for_each)]
fn sync_rigidbodies(
    rigidbody: &RigidBody2D,
    position: &mut Position,
    #[resource] physics: &mut PhysicsResources,
) {
    if let Some((rbh, _ch)) = rigidbody.handles {
        let rb = physics
            .bodies
            .get_mut(rbh)
            .expect("RigidBody not found for handle on back sync");
        let t = rb.position().translation.vector;
        position.0 = Vec2::new(t.x, t.y);
    }
}

#[system(for_each)]
fn create_rigidbodies(
    rigidbody: &mut RigidBody2D,
    Position(p): &Position,
    #[resource] physics: &mut PhysicsResources,
) {
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
