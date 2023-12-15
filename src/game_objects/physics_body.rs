use bevy_ecs::component::Component;
use rapier3d::dynamics::RigidBodyHandle;

#[derive(Component)]
pub struct PhysicsBody {
    pub body: RigidBodyHandle,
}
