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
use jukebox_util::peripheral::{KeyInputs, KnobInputs, PedalInputs};
use serde::{Deserialize, Serialize};

use crate::{gui::JukeBoxConfig, serial::SerialEvent};

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub enum InputKey {
    UnknownKey,

    KeySwitch1,
    KeySwitch2,
    KeySwitch3,
    KeySwitch4,
    KeySwitch5,
    KeySwitch6,
    KeySwitch7,
    KeySwitch8,
    KeySwitch9,
    KeySwitch10,
    KeySwitch11,
    KeySwitch12,
    KeySwitch13,
    KeySwitch14,
    KeySwitch15,
    KeySwitch16,

    KnobLeftSwitch,
    KnobLeftClockwise,
    KnobLeftCounterClockwise,
    KnobRightSwitch,
    KnobRightClockwise,
    KnobRightCounterClockwise,

    PedalLeft,
    PedalMiddle,
    PedalRight,
}
impl InputKey {
    pub fn trans_keys(i: KeyInputs) -> HashSet<Self> {
        let mut res = HashSet::new();

        let mut doif = |c, f| {
            if c {
                res.insert(f);
            }
        };

        doif(i.key1.is_down(), Self::KeySwitch1);
        doif(i.key2.is_down(), Self::KeySwitch2);
        doif(i.key3.is_down(), Self::KeySwitch3);
        doif(i.key4.is_down(), Self::KeySwitch4);
        doif(i.key5.is_down(), Self::KeySwitch5);
        doif(i.key6.is_down(), Self::KeySwitch6);
        doif(i.key7.is_down(), Self::KeySwitch7);
        doif(i.key8.is_down(), Self::KeySwitch8);
        doif(i.key9.is_down(), Self::KeySwitch9);
        doif(i.key10.is_down(), Self::KeySwitch10);
        doif(i.key11.is_down(), Self::KeySwitch11);
        doif(i.key12.is_down(), Self::KeySwitch12);
        doif(i.key13.is_down(), Self::KeySwitch13);
        doif(i.key14.is_down(), Self::KeySwitch14);
        doif(i.key15.is_down(), Self::KeySwitch15);
        doif(i.key16.is_down(), Self::KeySwitch16);

        res
    }

    pub fn trans_knob(i: KnobInputs) -> HashSet<Self> {
        let mut res = HashSet::new();

        let mut doif = |c, f| {
            if c {
                res.insert(f);
            }
        };

        doif(i.left_switch.is_down(), Self::KnobLeftSwitch);
        doif(i.left_direction.is_clockwise(), Self::KnobLeftClockwise);
        doif(
            i.left_direction.is_counter_clockwise(),
            Self::KnobLeftCounterClockwise,
        );

        doif(i.right_switch.is_down(), Self::KnobRightSwitch);
        doif(i.right_direction.is_clockwise(), Self::KnobRightClockwise);
        doif(
            i.right_direction.is_counter_clockwise(),
            Self::KnobRightCounterClockwise,
        );

        res
    }

    pub fn trans_pedals(i: PedalInputs) -> HashSet<Self> {
        let mut res = HashSet::new();

        let mut doif = |c, f| {
            if c {
                res.insert(f);
            }
        };

        doif(i.left.is_down(), Self::PedalLeft);
        doif(i.middle.is_down(), Self::PedalMiddle);
        doif(i.right.is_down(), Self::PedalRight);

        res
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
