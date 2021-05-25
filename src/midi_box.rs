use midir::{Ignore, MidiInput, MidiInputConnection};
use std::convert::TryFrom;
use std::error::Error;
use std::sync::mpsc;

pub struct MidiBox {
    _connnection: MidiInputConnection<()>,
    pub rx: mpsc::Receiver<MidiMsg>,
}

impl MidiBox {
    pub fn connect() -> Result<Self, Box<dyn Error>> {
        let mut midi_in = MidiInput::new("Water")?;
        midi_in.ignore(Ignore::None);

        // Get an input port (read from console if multiple are available)
        let in_ports = midi_in.ports();
        let in_port = match in_ports.len() {
            0 => return Err("no input port found".into()),
            1 => {
                println!(
                    "Choosing the only available input port: {}",
                    midi_in.port_name(&in_ports[0]).unwrap()
                );
                &in_ports[0]
            }
            _ => {
                println!("\nAvailable input ports:");
                for (i, p) in in_ports.iter().enumerate() {
                    println!("{}: {}", i, midi_in.port_name(p).unwrap());
                }
                println!(
                    "\nSelecting port 0: {}",
                    midi_in.port_name(&in_ports[0]).unwrap()
                );
                &in_ports[0]
            }
        };

        println!("\nOpening connection");
        let in_port_name = midi_in.port_name(in_port)?;

        let (tx, rx) = mpsc::channel();

        // conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
        let conn_in = midi_in.connect(
            in_port,
            "midir-read-input",
            move |stamp, message, _| {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
                if let Ok(msg) = MidiMsg::try_from((stamp, message)) {
                    let _ = tx.send(msg);
                }
            },
            (),
        )?;

        println!("Connection open, reading input from '{}'...", in_port_name);

        Ok(MidiBox {
            _connnection: conn_in,
            rx,
        })
    }
}

pub struct MidiMsg {
    pub timestamp: u64,
    pub event: MidiEvent,
}

pub enum MidiEvent {
    KeyPress { key: u8, velocity: u8 },
    KeyRelease { key: u8 },
    Volume { level: u8 },
}

impl TryFrom<(u64, &[u8])> for MidiMsg {
    type Error = ();

    fn try_from((timestamp, raw_msg): (u64, &[u8])) -> Result<Self, Self::Error> {
        if raw_msg.len() != 3 {
            return Err(());
        }
        let event = match raw_msg[0] {
            128 => MidiEvent::KeyRelease { key: raw_msg[1] },
            144 => MidiEvent::KeyPress {
                key: raw_msg[1],
                velocity: raw_msg[2],
            },
            176 => MidiEvent::Volume { level: raw_msg[2] },
            _ => Err(())?,
        };
        Ok(Self { timestamp, event })
    }
}
