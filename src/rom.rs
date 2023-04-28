use perpendicular::{Vector2, Vector3};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::schematic::{BlockState, Schematic};

pub fn find_soul_torches(schematic: &Schematic) -> Vec<Vector3<i64>> {
    let mut res = Vec::new();

    for (loc, blk) in schematic.blocks() {
        if blk.id() == "minecraft:soul_wall_torch" {
            res.push(loc.clone());
        }
    }

    res
}

pub fn find_program_lines(schematic: &Schematic) -> HashMap<Vector2<i64>, Vec<Vector3<i64>>> {
    let torch_locations = find_soul_torches(schematic);
    let mut lines = HashMap::new();

    for i in torch_locations {
        let line_id = Vector2::new2(*i.y(), *i.z());
        lines.entry(line_id).or_insert_with(Vec::new).push(i);
    }

    for i in lines.values_mut() {
        i.sort_by_key(|x| *x.x());
    }

    lines
}

pub fn order_lines(mut lines: HashMap<Vector2<i64>, Vec<Vector3<i64>>>) -> Vec<Vec<Vector3<i64>>> {
    let mut ordered_lines = vec![Vec::new(); 128];
    for i in 0..8 {
        // find the lowest line left
        let (id, bits) = lines
            .iter()
            .min_by_key(|(k, _)| k[0])
            .unwrap();

        // save its z
        let mut last_z = id[1];

        ordered_lines[i * 16 + 0] = bits.clone();

        // remove it
        lines.remove(&id.clone());

        for j in 1..16 {
            // find the smallest-y line
            // whose z is bigger than the last
            let (id, bits) = lines.iter()
                .filter(|(k, _)| k[1] > last_z)
                .min_by_key(|(k, _)| k[0])
                .unwrap();

            last_z = id[1];
            ordered_lines[i * 16 + j] = bits.clone();

            // remove it too
            lines.remove(&id.clone());
        }
    }

    ordered_lines
}

pub fn program_rom(mut schematic: Schematic, program: Vec<u16>) -> Schematic {
    let mut lines = find_program_lines(&schematic);
    // check if we have all bits
    assert_eq!(lines.len(), 128);
    for i in lines.values() {
        assert_eq!(i.len(), 16, "{:?}", i);
    }

    let ordered_lines = order_lines(lines);

    let mut set_bits = HashSet::new();

    for (bits, value) in ordered_lines.into_iter().zip(program) {
        for (idx, bit) in bits.into_iter().enumerate() {
            if (value >> idx) & 1 == 1 {
                set_bits.insert(bit);
            }
        }
    }

    for (pos, blk) in schematic.blocks_mut() {
        if blk.id() == "minecraft:soul_wall_torch" {
            if set_bits.contains(pos) {
                let mut redstone_torch = Rc::new(blk.same_props_new_id("minecraft:redstone_wall_torch"));
                *blk = redstone_torch;
            }
        } else {
            *blk = BlockState::air();
        }
    }

    schematic
}
