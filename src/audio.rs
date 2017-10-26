//! Primitives for audio playback.

use std::fmt;
use std::io::Cursor;
use std::rc::Rc;
use std::time::Duration;

use rodio as r;
use rodio::Source as _Source;

use hub::Operation as HubOperation;
use object::Object;

/// Audio segment with sound effects.
///
/// Can be loaded from file using [`Factory::load_audio`](struct.Factory.html#method.load_audio).
#[derive(Debug, Clone)]
pub struct Clip {
    data: Rc<Vec<u8>>,
    repeat: bool,
    duration: Option<Duration>,
    delay: Option<Duration>,
    fade_in: Option<Duration>,
    speed: f32,
}

impl Clip {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Clip {
            data: Rc::new(data),
            repeat: false,
            duration: None,
            delay: None,
            fade_in: None,
            speed: 1.0,
        }
    }

    /// Passing true enforces looping sound. Defaults to `false`.
    pub fn repeat(
        &mut self,
        enable: bool,
    ) {
        self.repeat = enable;
    }

    /// Clip the sound to the desired duration.
    pub fn take_duration(
        &mut self,
        duration: Duration,
    ) {
        self.duration = Some(duration);
    }

    /// Play sound after desired delay.
    pub fn delay(
        &mut self,
        delay: Duration,
    ) {
        self.delay = Some(delay);
    }

    /// Fade in sound in desired duration.
    pub fn fade_in(
        &mut self,
        duration: Duration,
    ) {
        self.fade_in = Some(duration);
    }

    /// Adjust the playback speed. Defaults to `1.0`.
    pub fn speed(
        &mut self,
        ratio: f32,
    ) {
        self.speed = ratio;
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Append(Clip),
    Resume,
    Pause,
    Stop,
    SetVolume(f32),
}

#[derive(Debug)]
pub(crate) struct AudioData {
    pub(crate) source: SourceInternal,
}

impl AudioData {
    pub(crate) fn new() -> Self {
        // TODO: Change to `r::default_endpoint()` in next `rodio` release.
        #[allow(deprecated)]
        let endpoint = if let Some(endpoint) = r::get_default_endpoint() {
            endpoint
        } else {
            // TODO: Better error handling
            panic!("Can't get default audio endpoint, can't play sound");
        };
        let sink = r::Sink::new(&endpoint);
        AudioData {
            source: SourceInternal::D2(sink),
        }
    }
}

/// Audio source. Can play only one sound at a time.
///
/// You must add it to the scene to play sounds.
/// You may create several `Source`s to play sounds simultaneously.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Source {
    pub(crate) object: Object,
}

impl Source {
    pub(crate) fn with_object(object: Object) -> Self {
        Source { object }
    }

    /// Add clip to the queue.
    pub fn play(
        &self,
        clip: &Clip,
    ) {
        let msg = HubOperation::SetAudio(Operation::Append(clip.clone()));
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Pause current sound.
    ///
    /// You can [`resume`](struct.Source.html#method.resume) playback.
    pub fn pause(&self) {
        let msg = HubOperation::SetAudio(Operation::Pause);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Resume playback after [`pausing`](struct.Source.html#method.pause).
    pub fn resume(&self) {
        let msg = HubOperation::SetAudio(Operation::Resume);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Stop the playback by emptying the queue.
    pub fn stop(&self) {
        let msg = HubOperation::SetAudio(Operation::Stop);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Adjust playback volume.
    ///
    /// Default value is `1.0`.
    pub fn set_volume(
        &self,
        volume: f32,
    ) {
        let msg = HubOperation::SetAudio(Operation::SetVolume(volume));
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}

//TODO: Remove dead_code lint
#[allow(dead_code)]
pub(crate) enum SourceInternal {
    D2(r::Sink),
    D3(r::SpatialSink),
}

impl fmt::Debug for SourceInternal {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match *self {
            SourceInternal::D2(_) => write!(f, "SourceInternal::D2"),
            SourceInternal::D3(_) => write!(f, "SourceInternal::D3"),
        }
    }
}

impl SourceInternal {
    pub(crate) fn pause(&self) {
        match *self {
            SourceInternal::D2(ref sink) => sink.pause(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn resume(&self) {
        match *self {
            SourceInternal::D2(ref sink) => sink.play(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn stop(&self) {
        match *self {
            SourceInternal::D2(ref sink) => sink.stop(),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn set_volume(
        &mut self,
        volume: f32,
    ) {
        match *self {
            SourceInternal::D2(ref mut sink) => sink.set_volume(volume),
            _ => unimplemented!(),
        }
    }

    pub(crate) fn append(
        &mut self,
        clip: Clip,
    ) {
        match *self {
            SourceInternal::D2(ref mut sink) => {
                let vec: Vec<u8> = (&*clip.data).clone();
                let decoder = r::Decoder::new(Cursor::new(vec));
                let mut boxed: Box<r::Source<Item = i16> + Send> = if let Ok(decoder) = decoder {
                    Box::new(decoder)
                } else {
                    eprintln!("Can't recognize audio clip format, can't play sound");
                    return;
                };
                if clip.repeat {
                    boxed = Box::new(boxed.repeat_infinite());
                }
                if clip.speed != 1.0 {
                    boxed = Box::new(boxed.speed(clip.speed));
                }
                if let Some(duration) = clip.delay {
                    boxed = Box::new(boxed.delay(duration));
                }
                if let Some(duration) = clip.duration {
                    boxed = Box::new(boxed.take_duration(duration));
                }
                if let Some(duration) = clip.fade_in {
                    boxed = Box::new(boxed.fade_in(duration));
                }
                sink.append(boxed);
            }
            SourceInternal::D3(_) => unimplemented!(),
        }
    }
}

/* TODO: Implement 3d sound.
pub struct Listener {
    pub(crate) object: Object,
}
*/
