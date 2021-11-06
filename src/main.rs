// rustedmusic 

use fluidsynth::*;
use rand::{thread_rng, Rng};
use std::thread;

fn main() {
    let mut settings = settings::Settings::new();
    let mut syn = synth::Synth::new(&mut settings);
    let _adriver = audio::AudioDriver::new(&mut settings, &mut syn);
    syn.sfload("/home/dave/Lib/SoundFonts/FluidR3_GM.sf2", 1);
    
    //  let interval = Duration::milliseconds(1000);

    for _x in 0..12 {
        let num: i32 = thread_rng().gen_range(0..12);
        let key = 60 + num;
        syn.noteon(0, key, 80);
        thread::sleep_ms(1000);
        syn.noteoff(0, key);
        thread::sleep_ms(1000);
    }
}
