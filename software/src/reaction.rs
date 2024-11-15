// Defining reactions to perform when actions happen (key pressed, knob turned, etc.)

use std::{
    collections::HashSet,
    sync::{
        atomic::AtomicBool,
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread::yield_now,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{gui::JukeBoxConfig, serial::SerialEvent};

#[derive(Eq, PartialEq, Debug, Hash, Clone)]
pub enum Peripheral {
    Keyboard,
    Knobs1,
    Knobs2,
    Pedal1,
    Pedal2,
    Pedal3,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum InputKey {
    UnknownKey,

    KeyboardSwitch1,
    KeyboardSwitch2,
    KeyboardSwitch3,
    KeyboardSwitch4,
    KeyboardSwitch5,
    KeyboardSwitch6,
    KeyboardSwitch7,
    KeyboardSwitch8,
    KeyboardSwitch9,
    KeyboardSwitch10,
    KeyboardSwitch11,
    KeyboardSwitch12,
    KeyboardSwitch13,
    KeyboardSwitch14,
    KeyboardSwitch15,
    KeyboardSwitch16,

    Knob1LeftClockwise,
    Knob1LeftCounterclockwise,
    Knob1LeftSwitch,
    Knob1RightClockwise,
    Knob1RightCounterclockwise,
    Knob1RightSwitch,

    Knob2LeftClockwise,
    Knob2LeftCounterclockwise,
    Knob2LeftSwitch,
    Knob2RightClockwise,
    Knob2RightCounterclockwise,
    Knob2RightSwitch,

    Pedal1Switch1,
    Pedal1Switch2,
    Pedal1Switch3,

    Pedal2Switch1,
    Pedal2Switch2,
    Pedal2Switch3,

    Pedal3Switch1,
    Pedal3Switch2,
    Pedal3Switch3,
}
impl InputKey {
    fn decode_word(w: u8, d: &[Self]) -> HashSet<Self> {
        let mut o = HashSet::new();

        for (i, k) in d.iter().enumerate() {
            if (w & (1 << i)) != 0 {
                o.insert(k.clone());
            }
        }

        o
    }

    pub fn decode_keyboard(w2: u8, w1: u8) -> HashSet<Self> {
        let mut i = HashSet::new();

        i.extend(Self::decode_word(
            w2,
            &[
                Self::KeyboardSwitch9,
                Self::KeyboardSwitch10,
                Self::KeyboardSwitch11,
                Self::KeyboardSwitch12,
                Self::KeyboardSwitch13,
                Self::KeyboardSwitch14,
                Self::KeyboardSwitch15,
                Self::KeyboardSwitch16,
            ],
        ));

        i.extend(Self::decode_word(
            w1,
            &[
                Self::KeyboardSwitch1,
                Self::KeyboardSwitch2,
                Self::KeyboardSwitch3,
                Self::KeyboardSwitch4,
                Self::KeyboardSwitch5,
                Self::KeyboardSwitch6,
                Self::KeyboardSwitch7,
                Self::KeyboardSwitch8,
            ],
        ));

        i
    }

    pub fn decode_knobs1(w: u8) -> HashSet<Self> {
        Self::decode_knob(
            w,
            Self::Knob1RightClockwise,
            Self::Knob1RightCounterclockwise,
            Self::Knob1RightSwitch,
            Self::Knob1LeftClockwise,
            Self::Knob1LeftCounterclockwise,
            Self::Knob1LeftSwitch,
        )
    }

    pub fn decode_knobs2(w: u8) -> HashSet<Self> {
        Self::decode_knob(
            w,
            Self::Knob2RightClockwise,
            Self::Knob2RightCounterclockwise,
            Self::Knob2RightSwitch,
            Self::Knob2LeftClockwise,
            Self::Knob2LeftCounterclockwise,
            Self::Knob2LeftSwitch,
        )
    }

    fn decode_knob(
        w: u8,
        rcw: Self,
        rccw: Self,
        rsw: Self,
        lcw: Self,
        lccw: Self,
        lsw: Self,
    ) -> HashSet<Self> {
        let mut i = HashSet::new();

        match w & 0b0000_0011 {
            0b01 => i.insert(rcw),
            0b10 => i.insert(rccw),
            _ => false,
        };
        if (w & 0b0000_0100) != 0 {
            i.insert(rsw);
        }
        match (w & 0b0001_1000) >> 3 {
            0b01 => i.insert(lcw),
            0b10 => i.insert(lccw),
            _ => false,
        };
        if (w & 0b0010_0000) != 0 {
            i.insert(lsw);
        }

        i
    }

    pub fn decode_pedal1(w: u8) -> HashSet<Self> {
        Self::decode_pedal(
            w,
            Self::Pedal1Switch3,
            Self::Pedal1Switch2,
            Self::Pedal1Switch1,
        )
    }

    pub fn decode_pedal2(w: u8) -> HashSet<Self> {
        Self::decode_pedal(
            w,
            Self::Pedal2Switch3,
            Self::Pedal2Switch2,
            Self::Pedal2Switch1,
        )
    }

    pub fn decode_pedal3(w: u8) -> HashSet<Self> {
        Self::decode_pedal(
            w,
            Self::Pedal3Switch3,
            Self::Pedal3Switch2,
            Self::Pedal3Switch1,
        )
    }

    fn decode_pedal(w: u8, k3: Self, k2: Self, k1: Self) -> HashSet<Self> {
        let mut i = HashSet::new();

        i.extend(Self::decode_word(w, &[k1, k2, k3]));

        i
    }
}

pub trait Reaction {
    // TODO: add result output for error reporting
    fn on_press(&self, key: InputKey);
    fn on_release(&self, key: InputKey);
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ReactionConfig {
    // Meta
    MetaTest(ReactionMetaTest),
    MetaSwitchProfile(),
    MetaCopyFromProfile(),

    // Input
    InputPressKey(),
    InputClickMouse(),
    InputMoveMouse(),
    InputScrollMouse(),

    // System
    SystemLaunch(),
    SystemWebsite(),
    SystemAudioInputControl(),
    SystemAudioOutputControl(),

    // Soundboard
    SoundboardPlaySound(),

    // Discord
    DiscordToggleMute(),
    DiscordToggleDeafen(),
    DiscordPushToTalk(),
    DiscordPushToMute(),
    DiscordToggleCamera(),
    DiscordToggleStream(),
    // OBS
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReactionMetaTest {}
impl Reaction for ReactionMetaTest {
    fn on_press(&self, key: InputKey) -> () {
        log::info!("Pressed {:?} !", key);
    }

    fn on_release(&self, key: InputKey) -> () {
        log::info!("Released {:?} !", key);
    }
}

fn run_key(reaction_config: &ReactionConfig, key: InputKey, pressed: bool) {
    // we cannot allow any panics to proceed past this point.
    // TODO: figure out how to do that

    match reaction_config {
        ReactionConfig::MetaTest(v) => match pressed {
            true => v.on_press(key),
            false => v.on_release(key),
        },
        _ => todo!(),
    }
}

pub fn reaction_task(
    brkr: Arc<AtomicBool>,
    s_evnt_rx: Receiver<SerialEvent>,
    r_evnt_tx: Sender<SerialEvent>,
    config: Arc<Mutex<JukeBoxConfig>>,
) -> Result<()> {
    let mut prevkeys = HashSet::<InputKey>::new();

    let mut timer = Instant::now();
    loop {
        if Instant::now() < timer {
            yield_now();
            continue;
        }
        timer = Instant::now() + Duration::from_millis(1);

        if brkr.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        while let Ok(evnt) = s_evnt_rx.try_recv() {
            r_evnt_tx
                .send(evnt.clone())
                .context("failed to send event to gui")?;
            match evnt {
                SerialEvent::GetInputKeys(keys) => {
                    let c = config.lock().unwrap();
                    let profiles = c.profiles.clone();
                    let current = c.current_profile.clone();
                    drop(c);

                    let pressed = keys.difference(&prevkeys);
                    let released = prevkeys.difference(&keys);

                    for p in pressed {
                        let c = profiles.get(&current).unwrap();
                        if let Some(r) = c.get(&p) {
                            let _ = run_key(r, *p, true);
                        }
                    }

                    for p in released {
                        let c = profiles.get(&current).unwrap();
                        if let Some(r) = c.get(&p) {
                            let _ = run_key(r, *p, false);
                        }
                    }

                    prevkeys = keys;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
