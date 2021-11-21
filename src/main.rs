// rustedmusic 
extern crate midir;
use midir::{MidiInput, MidiIO, Ignore as MidiIgnore};  // midi input library

// use fluidsynth::*;  // fluid synth rust interface
use fluidsynth::settings::Settings as FluidSettings;
use fluidsynth::synth::Synth as FluidSynth;
use fluidsynth::audio::AudioDriver as FluidAudio;

// use rand::{thread_rng, Rng};
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

    let (tx_from_midi, rx_to_run) = mpsc::channel::<[u8; 3]>();

    // create the default synthesizer settings
    let mut settings = FluidSettings::new();

    // create the synthesizer using the settings
    let mut syn = FluidSynth::new(&mut settings);

    // set the audio driver to PulseAudio
    settings.setstr("audio.driver", "pulseaudio");
    let _adriver = FluidAudio::new(&mut settings, &mut syn);
    syn.sfload("/home/dave/Lib/SoundFonts/FluidR3_GM.sf2", 1);

    let mut input = String::new();
    
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(MidiIgnore::None);
    
    let in_port = select_port(&midi_in, "input")?;
    
    println!("\nOpening connection");
    // let in_port_name = (midi_in.port_name(&in_port)).clone().unwrap();

    let in_port_name = "Midi Import Port Name";

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope

    // let _conn_in = thread::spawn(move || {
    //     midi_in.connect(&in_port, &in_port_name, move |stamp, message, _| {
        let _conn_in = midi_in.connect(&in_port, &in_port_name, move |stamp, message, _| {
            const NOTE_ON: u8 = 149;
            const NOTE_OFF: u8 = 133;
            let note: i32 = message[1] as i32;
            let force: i32 = message[2] as i32;
            if message[0] == NOTE_ON {
                // syn.noteon(0, note, force);
                println!("Note on for {} with force {}", note, force)
            } else if message[0] == NOTE_OFF {
                // syn.noteoff(0, note);
                println!("Note off for {}", force)
            } else {
                println!("message else");
            }
            println!("{}: {:?} (len = {})", stamp, message, message.len());
            let msg = [message[0].clone(), message[1].clone(), message[2].clone()];
            // tx_from_midi.send(msg).unwrap_or_else(|_error) => | println!("Error when forwarding message ..."));
            // task::sleep(Duration::from_millis(1)).await;   // async sleep function to unblock loop
            thread::sleep(Duration::from_millis(1));   // blocking sleep function
        }, ());  // close midi_in.connect  Note: no ? cannot use in a closure that returns '()'
    // });  // close thread::spawn

    // }, ())?;
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", &in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    // for message in rx_to_run {
    //     println!("Got: {:#?}", message);
    // }


    println!("Closing MIDI connection");

    // // confirm synthesizer plays notes
    // for _x in 0..12 {
    //     let num: i32 = thread_rng().gen_range(0..12);
    //     let key = 60 + num;
    //     syn.noteon(0, key, 80);
    //     thread::sleep(Duration::from_secs(1));
    //     syn.noteoff(0, key);
    // }

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