// rustedmusic
extern crate midir;
use midir::{MidiInput, MidiIO, Ignore as MidiIgnore};  // midi input library

// use fluidsynth::*;  // fluid synth rust interface
use fluidsynth::settings::Settings as FluidSettings;
use fluidsynth::synth::Synth as FluidSynth;
use fluidsynth::audio::AudioDriver as FluidAudio;

use rand::{thread_rng, Rng};
use std::thread;
use std::time::Duration;
// use async_std::task;

use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::sync::mpsc; // multi processor single consumer for thread messages
use std::fmt;

// ability create a boxed error from a string
#[derive(Debug)]
struct StrError<'a>(&'a str);
impl<'a> Error for StrError<'a> {}
impl<'a> fmt::Display for StrError<'a>{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate to the Display impl for `&str`:
        self.0.fmt(f)
    }
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run() -> Result<(), Box<dyn Error>> {

    const channel: i32 = 0; // only using channel 0 for now
    const bank_num: u32 = 0; // only using bank_num 0 for now

    let (tx_from_midi, rx_to_synth) = mpsc::channel::<[u8; 3]>();

    // create the default synthesizer settings
    let mut settings = FluidSettings::new();

    // create the synthesizer using the settings
    let mut syn = FluidSynth::new(&mut settings);

    // set the audio driver to PulseAudio
    settings.setstr("audio.driver", "pulseaudio");
    let _adriver = FluidAudio::new(&mut settings, &mut syn);

    let sf_id = syn.sfload("/media/dave/TowerData1/AudioFiles/SoundFonts/FluidR3_GM/FluidR3_GM.sf2", 1)
        .expect("invalid sf_id");

    println!("sfcount: {}", syn.sfcount());

    println!("Returned sound font ID: {}", sf_id);
    // select program voice from on FluidR3_GM sound font
    // syn.program_select(channel,1,0,19);
    select_program_voice(&syn, channel, sf_id, bank_num);

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(MidiIgnore::None);

    let in_port = select_port(&midi_in, "input")?;

    println!("\nOpening connection");
    // let in_port_name = (midi_in.port_name(&in_port)).clone().unwrap();

    let in_port_name = "Midi Import Port Name";

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope

    // let _conn_in = thread::spawn(move || {
    //     midi_in.connect(&in_port, &in_port_name, move |stamp, message, _| {
        let _conn_in = midi_in.connect(&in_port, &in_port_name, move |_stamp, message, _| {
            let msg = [message[0].clone(), message[1].clone(), message[2].clone()];
            // tx_from_midi.send(msg).unwrap_or_else(|_error) => | println!("Error when forwarding message ..."));
            tx_from_midi.send(msg).unwrap();
            // task::sleep(Duration::from_millis(1)).await;   // async sleep function to unblock loop
            thread::sleep(Duration::from_millis(1));   // blocking sleep function
        }, ());  // close midi_in.connect  Note: no ? cannot use in a closure that returns '()'
    // });  // close thread::spawn

    // }, ())?;

    // let tx_console = tx_from_midi.clone();
    // thread::spawn( || {
    //     println!("Connection open, reading input from '{}' (press enter to exit) ...", &in_port_name);
    //     input.clear();
    //     stdin().read_line(&mut input); // wait for next enter key press
    //     println!("to close MIDI connection");
    //     let msg: Vec<u8> = vec![0, 0, 0];
    // });


    // need to get shut down message
    for message in rx_to_synth {
        println!("Got: {:#?}", message);
        let [msg, note, force] = message;
        const NOTE_ON: u8 = 149;
        const NOTE_OFF: u8 = 133;
        let note: i32 = note as i32;
        let force: i32 = force as i32;
       println!("msg: {:?}, NOTE_ON: {:?}, NOTE_OFF: {:?} ", msg, NOTE_ON, NOTE_OFF);
        if msg == NOTE_ON {
            syn.noteon(0, note, force);
            println!("Note on for {} with force {}", note, force)
        } else if msg == NOTE_OFF {
            syn.noteoff(0, note);
            println!("Note off for {}", force)
        } else {
            println!("message else");
        }
        println!("{}: {:?} {:?} (len = {})", msg, note, force, message.len());
    }

    Ok(())
}

// function to choose a midi port
fn select_port<T: MidiIO>(midi_io: &T, descr: &str) -> Result<T::Port, Box<dyn Error>> {
    println!("Available {} ports:", descr);
    let midi_ports = midi_io.ports();
    println!("midi_ports.count(): {}", midi_ports.iter().count());
    match midi_ports.iter().count() {
        2 => {
            for (i, p) in midi_ports.iter().enumerate() {
                println!("{}: {:?}", i, midi_io.port_name(p));
            }
            print!("Please select {} port: ", descr);
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            let in_str = input.trim();
            print!("in_str: {}", &in_str);
            let port = midi_ports.get(in_str.parse::<usize>()?)
                .ok_or("Invalid port number")?;
            return Ok(port.clone())
        },
        1 => {
            let port = &midi_ports[0];
            return Ok(port.clone())
        },
        _ => {
            return Err(Box::new(StrError("Missing Port")))
        },
    }
}


// function to choose a program/voice
fn select_program_voice(syn: &fluidsynth::synth::Synth,
        channel: i32,
        sfont_id: u32,
        bank_num: u32
    ) -> u32 {
    println!("Enter Program/Voice number");
    let mut input_str = String::new();
    let mut voice: u32 = 0;
    while input_str != "y" && input_str != "Y" {
        stdin()
            .read_line(&mut input_str)
            .expect("failed to read in sound font number");
        match input_str.trim().parse::<u32>() {
            Ok(n) => voice = n,
            Err(_e) => voice = 0,
        }

        syn.program_select(channel, sfont_id, bank_num, voice);

        // confirm voice is the one wanted by playing some random notes
        for _x in 0..12 {
            let num: i32 = thread_rng().gen_range(0..12);
            let key = 60 + num;
            syn.noteon(channel, key, 80);
            thread::sleep(Duration::from_millis(100));
            syn.noteoff(channel, key);
        }

        input_str = String::new();
        println!("Is this the correct Program/Voice? (y)");
        stdin()
            .read_line(&mut input_str)
            .expect("failed to read in sound font number");
    }

    voice
}
