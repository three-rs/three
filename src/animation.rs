//! Animation system.
//!
//! ## Introduction
//!
//! The `three` animation system is designed around three structures, namely
//! [`Action`], [`Clip`], and [`Mixer`].
//!
//! ### Action
//!
//! An [`Action`] controls the playback properties of an animation.
//! Methods such as [`play`], [`pause`], and [`disable`] are provided to control
//! an single animation at runtime.
//!
//! Actions must be created and updated by a [`Mixer`].
//!
//! ### Mixer
//!
//! An animation [`Mixer`] schedules the playback of actions.
//!
//! The user is expected to create actions from a mixer with the [`Mixer::action`]
//! function and update actions with the [`Mixer::update`] function.
//!
//! ### Clip
//!
//! An animation [`Clip`] defines the keyframes and target of an animation.
//! Clips are usually imported from 3D formats such as glTF.
//!
//! ## Walkthrough
//!
//! ### Creating a mixer
//!
//! First, we create a [`Mixer`] to play our animation.
//!
//! ```rust,no_run
//! # #![allow(unused_mut)]
//! // Initialization omitted.
//! let mut mixer = three::animation::Mixer::new();
//! ```
//!
//! ### Loading some animation clips
//!
//! Now, we load some clips from an animated glTF scene.
//!
//! ```rust,no_run
//! # #![allow(unused_mut)]
//! # let mut window = three::Window::new("");
//! let mut gltf = window.factory.load_gltf("AnimatedScene.gltf");
//! gltf.group.set_parent(&window.scene);
//! ```
//!
//! ### Creating animation actions
//!
//! Now, we schedule the playback of the clips by creating actions.
//!
//! The created actions are enabled by default in the 'play' state. This means that
//! when calling [`Mixer::update`] the created actions will begin to be played back
//! immediately.
//!
//! ```rust,no_run
//! # #![allow(unused_mut)]
//! # let mut window = three::Window::new("");
//! # let mut mixer = three::animation::Mixer::new();
//! # let mut gltf = window.factory.load_gltf("AnimatedScene.gltf");
//! # gltf.group.set_parent(&window.scene);
//! let actions: Vec<three::animation::Action> = gltf.clips
//!     .into_iter()
//!     .map(|clip| mixer.action(clip))
//!     .collect();
//! ```
//!
//! ### Playing the animation back
//!
//! Finally, we run the animation actions by updating their [`Mixer`] in the main
//! game loop.
//!
//! ```rust,no_run
//! # #![allow(unused_mut)]
//! # let mut window = three::Window::new("");
//! # let camera = unimplemented!();
//! # let mut mixer = three::animation::Mixer::new();
//! # let mut gltf = window.factory.load_gltf("AnimatedScene.gltf");
//! # gltf.group.set_parent(&window.scene);
//! # let actions: Vec<three::animation::Action> = gltf.clips
//! #     .into_iter()
//! #     .map(|clip| mixer.action(clip))
//! #     .collect();
//! while window.update() {
//!     mixer.update(window.input.delta_time());
//!     window.render(&camera);
//! }
//! ```
//!
//! ### Putting it all together
//!
//! See the `gltf-animation` example for the full code.
//!
//! [`disable`]: struct.Action.html#method.disable
//! [`play`]: struct.Action.html#method.play
//! [`pause`]: struct.Action.html#method.pause
//!
//! [`Action`]: struct.Action.html
//! [`Clip`]: struct.Clip.html
//! [`Mixer`]: struct.Mixer.html
//! [`Mixer::action`]: struct.Mixer.html#method.action
//! [`Mixer::update`]: struct.Mixer.html#method.update

use cgmath;
use froggy;
use mint;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;

use Object;
use mint::IntraXYZ as IntraXyz;

/// Describes the interpolation behaviour between keyframes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Interpolation {
    /// Immediate change between keyframe values.
    Discrete,

    /// Linear interpolation between keyframe values.
    Linear,

    /// Smooth cubic interpolation between keyframe values.
    Cubic,

    /// Smooth Catmull–Rom spline interpolation between keyframe values.
    CatmullRom,
}

/// Describes the looping behaviour of an [`Action`].
///
/// [`Action`]: struct.Action.html
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LoopMode {
    /// Play the clip in forward order exactly once, i.e. do not loop at all.
    Once,

    /// Play the clip in forward order, repeating from the start.
    Repeat {
        /// The maximum number of repetitions.
        ///
        /// When set to `None`, the loop will repeat indefinately.
        limit: Option<u32>,
    },

    /// Play the clip alternatively in forward and reverse order.
    PingPong {
        /// The maximum number of repetitions.
        ///
        /// When set to `None`, the loop will repeat indefinately.
        limit: Option<u32>,
    },
}

/// Describes the target property of an animation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Binding {
    /// Targets the position property of an [`Object`].
    ///
    /// The corresponding keyframe values must be [`Vector3`].
    ///
    /// [`Object`]: ../object/struct.Object.html
    /// [`Vector3`]: enum.Values.html#variant.Vector3
    Position,

    /// Targets the orientation property of an [`Object`].
    ///
    /// The corresponding keyframe values must be [`Quaternion`] or [`Euler`].
    ///
    /// [`Object`]: ../object/struct.Object.html
    /// [`Quaternion`]: enum.Values.html#variant.Quaternion
    /// [`Euler`]: enum.Values.html#variant.Euler
    Orientation,

    /// Targets the scale property of an [`Object`].
    ///
    /// The corresponding keyframe values must be [`Scalar`].
    ///
    /// [`Object`]: ../object/struct.Object.html
    /// [`Scalar`]: enum.Values.html#variant.Scalar
    Scale,
}

/// An index into the frames of a track.
enum FrameRef {
    /// The time is before the start of the frames.
    Unstarted,

    /// The time corresponds to the given frame index.
    InProgress(usize),

    /// The time is after the end of the last frame.
    Ended,
}

/// The keyframe values of a [`Track`].
///
/// [`Track`]: struct.Track.html
#[derive(Clone, Debug)]
pub enum Values {
    /// Euler angle keyframes in radians.
    Euler(Vec<mint::EulerAngles<f32, IntraXyz>>),

    /// Quaternion keyframes.
    Quaternion(Vec<mint::Quaternion<f32>>),

    /// Scalar keyframes.
    ///
    /// ## Note
    ///
    /// Only uniform scaling is supported, hence the glTF importer takes the
    /// Y axis as the scaling direction, ignoring any scaling in the X and Z axes.
    Scalar(Vec<f32>),

    /// 3D vector keyframes.
    Vector3(Vec<mint::Vector3<f32>>),
}

/// Message data sent from `Action` to `Mixer` over a channel.
enum Operation {
    Enable,
    Disable,
    Pause,
    Play,
    SetLoopMode(LoopMode),
}

/// Message type sent from `Action` to `Mixer`.
type Message = (froggy::WeakPointer<ActionData>, Operation);

/// Controls the playback properties of an animation
#[derive(Clone, Debug)]
pub struct Action {
    /// Message channel to parent mixer.
    tx: mpsc::Sender<Message>,

    /// Pointer to the action data held by the parent mixer.
    pointer: froggy::Pointer<ActionData>,
}

impl PartialEq for Action {
    fn eq(
        &self,
        other: &Action,
    ) -> bool {
        self.pointer == other.pointer
    }
}

impl Eq for Action {}

impl Hash for Action {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.pointer.hash(state);
    }
}

/// Internal data for an animation action.
struct ActionData {
    /// The animation data for this action.
    pub clip: Clip,

    /// Specifies whether the action is enabled or disabled.
    ///
    /// A disabled action has no impact.
    pub enabled: bool,

    /// Specifies the looping behaviour of this action.
    pub loop_mode: LoopMode,

    /// Specifies whether the action is paused.
    pub paused: bool,

    /// The local time of this action in seconds, starting at 0.0.
    ///
    /// This value get clamped or wrapper to [0.0, clip.duration] depending on
    /// the loop mode.
    pub local_time: f32,

    /// Time scaling factor.
    pub local_time_scale: f32,

    // Unimplemented properties
    // ------------------------
    // * weight
    // * zero_slope_at_end
    // * zero_slope_at_start
}

/// A reusable set of keyframe tracks which represent an animation.
#[derive(Clone, Debug)]
pub struct Clip {
    /// A name for this clip.
    pub name: Option<String>,

    /// The animation keyframe tracks.
    pub tracks: Vec<(Track, Object)>,
}

/// A track of animation keyframes.
#[derive(Clone, Debug)]
pub struct Track {
    /// The object property this track updates.
    pub binding: Binding,

    /// The keyframe time values.
    pub times: Vec<f32>,

    /// The keyframe values.
    pub values: Values,

    /// Specifies the interpolation strategy between keyframes.
    pub interpolation: Interpolation,
}

/// Scheduler for the playback of animation actions.
///
/// Use this to update animation actions.
pub struct Mixer {
    actions: froggy::Storage<ActionData>,
    rx: mpsc::Receiver<Message>,
    tx: mpsc::Sender<Message>,
}

impl Action {
    fn send(
        &mut self,
        operation: Operation,
    ) -> &mut Self {
        let message = (self.pointer.downgrade(), operation);
        let _ = self.tx.send(message);
        self
    }

    /// Enables the animation action.
    pub fn enable(&mut self) -> &mut Self {
        self.send(Operation::Enable)
    }

    /// Disables the animation action.
    pub fn disable(&mut self) -> &mut Self {
        self.send(Operation::Disable)
    }

    /// Pauses the animation action.
    pub fn pause(&mut self) -> &mut Self {
        self.send(Operation::Pause)
    }

    /// Plays the animation action.
    pub fn play(&mut self) -> &mut Self {
        self.send(Operation::Play)
    }

    /// Sets the animation loop mode.
    pub fn set_loop_mode(
        &mut self,
        loop_mode: LoopMode,
    ) -> &mut Self {
        self.send(Operation::SetLoopMode(loop_mode))
    }
}

impl Mixer {
    fn process_messages(&mut self) {
        while let Ok((weak_ptr, operation)) = self.rx.try_recv() {
            let action = match weak_ptr.upgrade() {
                Ok(ptr) => &mut self.actions[&ptr],
                Err(_) => continue,
            };
            match operation {
                Operation::Enable => action.enabled = true,
                Operation::Disable => action.enabled = false,
                Operation::Pause => action.paused = true,
                Operation::Play => {
                    action.paused = false;
                    action.enabled = true;
                }
                Operation::SetLoopMode(loop_mode) => action.loop_mode = loop_mode,
            }
        }
    }

    fn update_actions(
        &mut self,
        delta_time: f32,
    ) {
        for action in self.actions.iter_mut() {
            action.update(delta_time);
        }
    }

    /// Creates a new animation mixer.
    pub fn new() -> Self {
        let actions = froggy::Storage::new();
        let (tx, rx) = mpsc::channel();
        Mixer { actions, rx, tx }
    }

    /// Spawns a new animation [`Action`] to be updated by this mixer.
    ///
    /// [`Action`]: struct.Action.html
    pub fn action(
        &mut self,
        clip: Clip,
    ) -> Action {
        let action_data = ActionData::new(clip);
        let pointer = self.actions.create(action_data);
        let tx = self.tx.clone();
        Action { tx, pointer }
    }

    /// Updates the actions owned by the mixer.
    pub fn update(
        &mut self,
        delta_time: f32,
    ) {
        self.process_messages();
        self.update_actions(delta_time);
    }
}

impl ActionData {
    fn new(clip: Clip) -> Self {
        ActionData {
            clip: clip,
            enabled: true,
            loop_mode: LoopMode::Repeat { limit: None },
            paused: false,
            local_time: 0.0,
            local_time_scale: 1.0,
        }
    }

    /// Updates a single animation action.
    fn update(
        &mut self,
        delta_time: f32,
    ) {
        if self.paused || !self.enabled {
            return;
        }

        self.local_time += delta_time * self.local_time_scale;
        let mut finish_count = 0;
        for &mut (ref track, ref mut target) in self.clip.tracks.iter_mut() {
            let frame_index = match track.frame_at_time(self.local_time) {
                FrameRef::Unstarted => continue,
                FrameRef::Ended => {
                    finish_count += 1;
                    continue;
                }
                FrameRef::InProgress(i) => i,
            };
            let frame_start_time = track.times[frame_index];
            let frame_end_time = track.times[frame_index + 1];
            let frame_delta_time = frame_end_time - frame_start_time;
            // Interpolation constant in range `[0.0, 1.0]` between `frame[i]`
            // and `frame[i + 1]`.
            let s = (self.local_time - frame_start_time) / frame_delta_time;

            match (track.binding, &track.values) {
                (Binding::Orientation, &Values::Euler(ref values)) => {
                    let frame_start_value = {
                        let euler = values[frame_index];
                        cgmath::Quaternion::from(cgmath::Euler::new(
                            cgmath::Rad(euler.a),
                            cgmath::Rad(euler.b),
                            cgmath::Rad(euler.c),
                        ))
                    };
                    let frame_end_value = {
                        let euler = values[frame_index + 1];
                        cgmath::Quaternion::from(cgmath::Euler::new(
                            cgmath::Rad(euler.a),
                            cgmath::Rad(euler.b),
                            cgmath::Rad(euler.c),
                        ))
                    };
                    let update = frame_start_value.slerp(frame_end_value, s);
                    target.set_orientation(update);
                }
                (Binding::Orientation, &Values::Quaternion(ref values)) => {
                    let frame_start_value: cgmath::Quaternion<f32> = values[frame_index].into();
                    let frame_end_value: cgmath::Quaternion<f32> = values[frame_index + 1].into();
                    let update = frame_start_value.slerp(frame_end_value, s);
                    target.set_orientation(update);
                }
                (Binding::Position, &Values::Vector3(ref values)) => {
                    use cgmath::{EuclideanSpace, InnerSpace};
                    let frame_start_value: cgmath::Vector3<f32> = values[frame_index].into();
                    let frame_end_value: cgmath::Vector3<f32> = values[frame_index + 1].into();
                    let update = frame_start_value.lerp(frame_end_value, s);
                    target.set_position(cgmath::Point3::from_vec(update));
                }
                (Binding::Scale, &Values::Scalar(ref values)) => {
                    let frame_start_value = values[frame_index];
                    let frame_end_value = values[frame_index + 1];
                    let update = frame_start_value * (1.0 - s) + frame_end_value * s;
                    target.set_scale(update);
                }
                _ => panic!("Unsupported (binding, value) pair"),
            }
        }

        if finish_count == self.clip.tracks.len() {
            match self.loop_mode {
                LoopMode::Once => self.enabled = false,
                LoopMode::Repeat { limit: None } => self.local_time = 0.0,
                LoopMode::Repeat { limit: Some(0) } => self.enabled = false,
                LoopMode::Repeat { limit: Some(n) } => {
                    self.local_time = 0.0;
                    self.loop_mode = LoopMode::Repeat { limit: Some(n - 1) };
                }
                LoopMode::PingPong { .. } => {
                    // TODO
                    unimplemented!()
                }
            }
        }
    }
}

impl Track {
    fn frame_at_time(
        &self,
        t: f32,
    ) -> FrameRef {
        if t < self.times[0] {
            // The clip hasn't started yet.
            return FrameRef::Unstarted;
        }

        if t > *self.times.last().unwrap() {
            // The clip has ended.
            return FrameRef::Ended;
        }

        let mut i = 0;
        while t > self.times[i + 1] {
            i += 1;
        }

        FrameRef::InProgress(i)
    }
}
