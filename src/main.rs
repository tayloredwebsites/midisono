// rustedmusic 
extern crate midir;

use fluidsynth::*;
use rand::{thread_rng, Rng};
use std::{thread, time::Duration};
use std::io::{stdin, stdout, Write};
use std::error::Error;

use midir::{MidiInput, Ignore};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    // create the default synthesizer settings
    let mut settings = settings::Settings::new();

    // create the synthesizer using the settings
    let mut syn = synth::Synth::new(&mut settings);

    // set the audio driver to PulseAudio
    settings.setstr("audio.driver", "pulseaudio");
    let _adriver = audio::AudioDriver::new(&mut settings, &mut syn);
    syn.sfload("/home/dave/Lib/SoundFonts/FluidR3_GM.sf2", 1);

    let mut input = String::new();
    
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);
    
    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!("Choosing the only available input port: {}", midi_in.port_name(&in_ports[0]).unwrap());
            &in_ports[0]
        },
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports.get(input.trim().parse::<usize>()?)
                     .ok_or("invalid input port selected")?
        }
    };
    
    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(in_port, "midir-read-input", move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
    }, ())?;
    
    println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing MIDI connection");

    for _x in 0..12 {
        let num: i32 = thread_rng().gen_range(0..12);
        let key = 60 + num;
        syn.noteon(0, key, 80);
        thread::sleep(Duration::from_secs(1));
        syn.noteoff(0, key);
    }

    Ok(())
}
