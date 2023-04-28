use std::collections::{HashMap, HashSet};
use std::fs::{File, read};
use std::iter;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use perpendicular::{Vector, Vector2, Vector3};
use tracing::info;
use crate::instruction::Instruction;
use crate::schematic::{BlockState, Schematic};
use crate::server::ServerConfig;

mod server;
mod schematic;
#[macro_use]
mod instruction;
mod rom;

fn main() -> color_eyre::Result<()> {
    color_eyre::install().ok();
    tracing_subscriber::fmt::init();

    let fili = ServerConfig::fili();

    fili.download_schematic("jona-diag-rom-fixed", "input.schem")?;
    let mut rom = Schematic::from_file("input.schem")?;


    let mut program = program! {
        nop;
        nop;
        nop;
        nop;
        jmp 0;
    };


    let programmed_rom = rom::program_rom(
        rom,
        program,
    );


    programmed_rom.to_file("generated.schem")?;
    fili.upload_schematic("generated.schem", "generated")?;


    Ok(())
}
