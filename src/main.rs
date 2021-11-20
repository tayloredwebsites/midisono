// rustedmusic 

use fluidsynth::*;
use rand::{thread_rng, Rng};
use std::{thread, time::Duration};

fn main() {

    // http://fluidsynth.sourceforge.net/api/index.html
    // https://github.com/scholtzan/rust-fluidsynth
    // https://github.com/tayloredwebsites/rust-fluidsynth

    // create the default synthesizer settings
    let mut settings = settings::Settings::new();

    // create the synthesizer using the settings
    let mut syn = synth::Synth::new(&mut settings);

    // set the audio driver to PulseAudio
    settings.setstr("audio.driver", "pulseaudio");
    let _adriver = audio::AudioDriver::new(&mut settings, &mut syn);
    syn.sfload("/home/dave/Lib/SoundFonts/FluidR3_GM.sf2", 1);

    // input from midi controller instead of following loop
    // https://crates.io/crates/midir

    for _x in 0..12 {
        let num: i32 = thread_rng().gen_range(0..12);
        let key = 60 + num;
        syn.noteon(0, key, 80);
        thread::sleep(Duration::from_secs(1));
        syn.noteoff(0, key);
    }
}
