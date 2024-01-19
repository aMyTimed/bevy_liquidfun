use std::pin::Pin;

use bevy::ecs::system::EntityCommand;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, World};
use libliquidfun_sys::box2d::ffi;

use crate::dynamics::{b2Joint, b2JointType, b2World, JointPtr};
use crate::internal::to_b2Vec2;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2DistanceJoint {
    /// The local anchor point relative to bodyA's origin.
    local_anchor_a: Vec2,

    /// The local anchor point relative to bodyB's origin.
    local_anchor_b: Vec2,

    /// The minimum distance between the two anchors.
    pub min_length: f32,

    /// The maximum distance between the two anchors.
    pub max_length: f32,

    /// The linear stiffness in N/m.
    pub stiffness: f32,

    /// The linear damping in N*s/m.
    pub damping: f32,

    /// The rest length that the joint targets.
    pub length: f32,
}

impl b2DistanceJoint {
    pub fn new(def: &b2DistanceJointDef) -> Self {
        Self {
            local_anchor_a: def.local_anchor_a,
            local_anchor_b: def.local_anchor_b,
            min_length: def.min_length,
            max_length: def.max_length,
            stiffness: def.stiffness,
            damping: def.damping,
            length: def.length,
        }
    }

    pub(crate) fn create_ffi_joint<'a>(
        &self,
        b2_world: &mut b2World,
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
    ) -> JointPtr<'a> {
        unsafe {
            let body_a = b2_world.get_body_ptr_mut(body_a).unwrap().as_mut();
            let body_a = body_a.get_unchecked_mut() as *mut ffi::b2Body;
            let body_b = b2_world.get_body_ptr_mut(body_b).unwrap().as_mut();
            let body_b = body_b.get_unchecked_mut() as *mut ffi::b2Body;
            let ffi_world = b2_world.get_world_ptr().as_mut();
            let ffi_joint = ffi::CreateDistanceJoint(
                ffi_world,
                body_a,
                body_b,
                collide_connected,
                to_b2Vec2(&self.local_anchor_a),
                to_b2Vec2(&self.local_anchor_b),
                self.length,
                self.min_length,
                self.max_length,
                self.stiffness,
                self.damping,
            );
            let ffi_joint = Pin::new_unchecked(ffi_joint.as_mut().unwrap());
            JointPtr::Distance(ffi_joint)
        }
    }

    pub(crate) fn sync_to_world(&self, mut joint_ptr: Pin<&mut ffi::b2DistanceJoint>) {
        joint_ptr.as_mut().SetLength(self.length);
        joint_ptr.as_mut().SetMinLength(self.min_length);
        joint_ptr.as_mut().SetMaxLength(self.max_length);
        joint_ptr.as_mut().SetStiffness(self.stiffness);
        joint_ptr.as_mut().SetDamping(self.damping);
    }
}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub struct b2DistanceJointDef {
    pub local_anchor_a: Vec2,
    pub local_anchor_b: Vec2,
    pub min_length: f32,
    pub max_length: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub length: f32,
}

pub struct CreateDistanceJoint {
    body_a: Entity,
    body_b: Entity,
    collide_connected: bool,
    def: b2DistanceJointDef,
}

impl CreateDistanceJoint {
    pub fn new(
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
        def: &b2DistanceJointDef,
    ) -> Self {
        Self {
            body_a,
            body_b,
            collide_connected,
            def: def.clone(),
        }
    }
}

impl EntityCommand for CreateDistanceJoint {
    fn apply(self, id: Entity, world: &mut World) {
        let joint = b2Joint::new(
            b2JointType::Distance,
            self.body_a,
            self.body_b,
            self.collide_connected,
        );
        let distance_joint = b2DistanceJoint::new(&self.def);
        world.entity_mut(id).insert((joint, distance_joint));
    }
}
