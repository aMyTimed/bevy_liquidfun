use std::pin::Pin;

use bevy::ecs::system::EntityCommand;
use bevy::math::Vec2;
use bevy::prelude::{Component, Entity, World};
use libliquidfun_sys::box2d::ffi;

use crate::dynamics::{b2Joint, b2JointType, b2World, JointPtr};
use crate::internal::to_b2Vec2;

#[allow(non_camel_case_types)]
#[derive(Component, Debug)]
pub struct b2MouseJoint {
    /// The target point.
    pub target: Vec2,

    /// The maximum force in Newtons.
    pub max_force: f32,

    /// The linear stiffness in N/m.
    pub stiffness: f32,

    /// The linear damping in N*s/m.
    pub damping: f32,
}

impl b2MouseJoint {
    pub fn new(def: &b2MouseJointDef) -> Self {
        Self {
            target: def.target,
            max_force: def.max_force,
            stiffness: def.stiffness,
            damping: def.damping,
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
            let ffi_joint = ffi::CreateMouseJoint(
                ffi_world,
                body_a,
                body_b,
                collide_connected,
                to_b2Vec2(&self.target),
                self.max_force,
                self.stiffness,
                self.damping,
            );
            let ffi_joint = Pin::new_unchecked(ffi_joint.as_mut().unwrap());
            JointPtr::Mouse(ffi_joint)
        }
    }

    pub(crate) fn sync_to_world(&self, mut joint_ptr: Pin<&mut ffi::b2MouseJoint>) {
        joint_ptr.as_mut().SetTarget(&to_b2Vec2(&self.target));
        joint_ptr.as_mut().SetMaxForce(self.max_force);
        joint_ptr.as_mut().SetStiffness(self.stiffness);
        joint_ptr.as_mut().SetDamping(self.damping);
    }
}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub struct b2MouseJointDef {
    pub target: Vec2,
    pub max_force: f32,
    pub stiffness: f32,
    pub damping: f32,
}

pub struct CreateMouseJoint {
    body_a: Entity,
    body_b: Entity,
    collide_connected: bool,
    def: b2MouseJointDef,
}

impl CreateMouseJoint {
    pub fn new(
        body_a: Entity,
        body_b: Entity,
        collide_connected: bool,
        def: &b2MouseJointDef,
    ) -> Self {
        Self {
            body_a,
            body_b,
            collide_connected,
            def: def.clone(),
        }
    }
}

impl EntityCommand for CreateMouseJoint {
    fn apply(self, id: Entity, world: &mut World) {
        let joint = b2Joint::new(
            b2JointType::Mouse,
            self.body_a,
            self.body_b,
            self.collide_connected,
        );
        let mouse_joint = b2MouseJoint::new(&self.def);
        world.entity_mut(id).insert((joint, mouse_joint));
    }
}
