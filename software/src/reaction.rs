// Defining reactions to perform when actions happen (key pressed, knob turned, etc.)

#[derive(PartialEq, Debug)]
pub enum InputKey {
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

    Knob11Clockwise,
    Knob11Counterclockwise,
    Knob11Switch,
    Knob12Clockwise,
    Knob12Counterclockwise,
    Knob12Switch,

    Knob21Clockwise,
    Knob21Counterclockwise,
    Knob21Switch,
    Knob22Clockwise,
    Knob22Counterclockwise,
    Knob22Switch,

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

pub trait Reaction {
    // TODO: add result output for error reporting
    fn on_press(&self, key: InputKey);
    fn on_release(&self, key: InputKey);
}

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

pub struct ReactionMetaTest {}
impl Reaction for ReactionMetaTest {
    fn on_press(&self, key: InputKey) -> () {
        log::info!("Pressed {:?} !", key);
    }

    fn on_release(&self, key: InputKey) -> () {
        log::info!("Released {:?} !", key);
    }
}
