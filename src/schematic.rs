use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::{File, read};
use std::io::{Cursor, Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use color_eyre::eyre::{bail, ContextCompat, eyre, WrapErr};
use nbt::{from_gzip_reader, from_reader, to_gzip_writer, to_writer, Value};
use perpendicular::Vector3;
use serde::{Serialize, Deserialize};
use tracing::info;

#[derive(Serialize, Deserialize)]
#[serde(rename_all="PascalCase")]
struct SchemBlockEntity {
    id: String,

    #[serde(serialize_with="nbt::i32_array")]
    pos: Vec<i32>,

    #[serde(flatten)]
    props: HashMap<String, Value>
}


#[derive(Serialize, Deserialize, Copy, Clone)]
struct Metadata {
    #[serde(rename="WEOffsetX")]
    offset_x: i32,
    #[serde(rename="WEOffsetY")]
    offset_y: i32,
    #[serde(rename="WEOffsetZ")]
    offset_z: i32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all="PascalCase")]
struct SchemFormat {
    #[serde(serialize_with="nbt::i8_array")]
    block_data: Vec<i8>,
    block_entities: Vec<SchemBlockEntity>,
    data_version: i32,
    height: i16,
    length: i16,
    metadata: Metadata,
    #[serde(serialize_with="nbt::i32_array")]
    offset: Vec<i32>,
    palette: HashMap<String, i32>,
    palette_max: i32,
    version: i32,
    width: i16,
}

#[derive(Debug, Clone)]
pub struct BlockEntity {
    id: String,
    props: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct BlockState {
    id: String,
    props: HashMap<String, String>,
}

macro_rules! define_standard_block_states {
    ($($ident: ident = $literal: literal),* $(,)?) => {
        $(
            pub fn $ident() -> Rc<Self> {
                Self::new($literal)
            }
        )*
    };
}

impl BlockState {
    define_standard_block_states!(
        air = "minecraft:air",
        stone = "minecraft:stone",
    );

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn same_props_new_id(&self, id: impl AsRef<str>) -> Self {
        Self { id: id.as_ref().to_string(), props: self.props.clone() }
    }

    pub fn new(name: impl AsRef<str>) -> Rc<BlockState> {
        Self::with_props(name, HashMap::new())
    }

    pub fn with_props(name: impl AsRef<str>, props: HashMap<String, String>) -> Rc<BlockState> {
        Rc::new(Self {
            id: name.as_ref().to_string(),
            props: props,
        })
    }
}

impl Display for BlockState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let props = &self.props;
        write!(f, "{}", self.id)?;
        if !props.is_empty() {
            write!(f, "[")?;
            let len = props.len();
            for (idx, (k, v)) in props.iter().enumerate() {
                write!(f, "{k}={v}")?;
                if idx < len - 1 {
                    write!(f, ",")?;
                }
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}

impl FromStr for BlockState {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((id, props_data)) = s.split_once('[') else {
            return Ok(Self {
                id: s.to_string(),
                props: Default::default(),
            });
        };

        let mut props = HashMap::new();
        for i in props_data.trim_end_matches(']').split(',') {
            let (l, r) = i.split_once('=').ok_or(eyre!("no equal in palette prop"))?;
            props.insert(l.to_string(), r.to_string());
        }

        Ok(Self {
            id: id.to_string(),
            props,
        })
    }
}

#[derive(Clone)]
pub struct Schematic {
    pub original_width: usize,
    pub original_length: usize,
    pub original_height: usize,
    pub original_offset: [i32; 3],
    pub original_data_version: i32,
    original_metadata: Metadata,
    block_data: HashMap<Vector3<i64>, Rc<BlockState>>,
    block_entities: HashMap<Vector3<i64>, BlockEntity>,
}

impl Schematic {
    fn block_at(&self, loc: Vector3<i64>) -> Option<Rc<BlockState>> {
        self.block_data.get(&loc).cloned()
    }

    fn encode_block_data(&self) -> color_eyre::Result<(
        Vec<i8>,
        HashMap<String, i32>,
    )> {
        let x_min = self.min_x();
        let y_min = self.min_y();
        let z_min = self.min_z();

        let height = self.height();
        let length = self.length();
        let width = self.width();

        let mut block_data = Vec::new();
        let mut palette = HashMap::new();
        let mut id = 0;

        for y in 0..height {
            for z in 0..length {
                for x in 0..width {
                    let y0 = y_min + y as i64;
                    let z0 = z_min + z as i64;
                    let x0 = x_min + x as i64;

                    let block_at = self.block_at(Vector3::new3(x0, y0, z0));
                    let block = match block_at.as_deref() {
                        None => {
                            "minecraft:air".to_string()
                        }
                        Some(b) => {
                            b.to_string()
                        }
                    };

                    let mut id: i32 = *palette.entry(block).or_insert_with(|| {
                        id += 1;
                        id - 1
                    });

                    while (id & -128) != 0 {
                        block_data.push((id & 127 | 128) as i8);
                        id = ((id as u32) >> 7) as i32;
                    }
                    block_data.push(id as i8);
                }
            }
        }

        Ok((
            block_data,
            palette,
        ))
    }

    pub fn to_writer(&self, mut w: impl Write) -> color_eyre::Result<()> {
        let offset = self.original_offset;
        let (block_data, palette) = self.encode_block_data()?;
        println!("{:?}", offset);

        let block_entities = self.block_entities
            .iter()
            .map(|(k, v)| SchemBlockEntity {
                id: v.id.clone(),
                pos: vec![*k.x() as i32, *k.y() as i32, *k.z() as i32],
                props: v.props.clone(),
            })
            .collect();

        let format = SchemFormat {
            width: self.len_x() as i16,
            length: self.len_z() as i16,
            height: self.len_y() as i16,
            block_data,
            palette_max: palette.len() as i32,
            palette,
            offset: offset.to_vec(),
            block_entities,
            data_version: self.original_data_version,
            metadata: self.original_metadata,
            version: 2,
        };

        to_gzip_writer(&mut w, &format, Some("Schematic"))?;

        Ok(())
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> color_eyre::Result<()> {
        let mut f = File::create(path)?;
        self.to_writer(f)?;

        Ok(())
    }

    pub fn to_bytes(&self) -> color_eyre::Result<Vec<u8>> {
        let mut res = Vec::new();
        self.to_writer(Cursor::new(&mut res))?;

        Ok(res)
    }

    pub fn from_reader(reader: impl Read) -> color_eyre::Result<Self> {
        let format: SchemFormat = from_gzip_reader(reader)
            .wrap_err("read and decode nbt")?;

        let decoded_palette = Self::decode_palette(&format)?;
        let mut decoded_block_data = Self::decode_block_data(&format, &decoded_palette)?;
        let mut block_entities = HashMap::new();

        info!("{}", format.palette.len());
        info!("{}", format.palette_max);

        for i in format.block_entities {
            block_entities.insert(
                Vector3::new3(
                    i.pos[0] as i64,
                    i.pos[1] as i64,
                    i.pos[2] as i64,
                ),
                BlockEntity {
                    id: i.id,
                    props: i.props,
                }
            );
        }

        Ok(Self {
            original_width: format.width as usize,
            original_length: format.length as usize,
            original_height: format.height as usize,
            original_offset: [format.offset[0], format.offset[1], format.offset[2]],
            original_data_version: format.data_version,
            original_metadata: format.metadata,
            block_data: decoded_block_data,
            block_entities,
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> color_eyre::Result<Self> {
        let file = File::open(path)
            .wrap_err("open file")?;

        Self::from_reader(file)
    }

    pub fn from_bytes(data: impl AsRef<[u8]>) -> color_eyre::Result<Self> {
        Self::from_reader(Cursor::new(data.as_ref()))
    }

    fn decode_palette(format: &SchemFormat) -> color_eyre::Result<Vec<Option<Rc<BlockState>>>> {
        let mut res = Vec::new();
        res.resize(format.palette.len(), None);

        for (name, i) in &format.palette {
            if *i as usize > res.len() {
                res.resize(*i as usize, None);
            }
            res[*i as usize] = Some(Rc::new(name.parse()?));
        }

        Ok(res)
    }

    fn decode_block_data(format: &SchemFormat, palette: &[Option<Rc<BlockState>>]) -> color_eyre::Result<HashMap<Vector3<i64>, Rc<BlockState>>> {
        let mut buffer = HashMap::new();
        let ref block_data = format.block_data;

        let mut index: i64 = 0;
        let mut i = 0;
        let mut value: usize = 0;
        let mut varint_length = 0;
        while i < block_data.len() {
            value = 0;
            varint_length = 0;

            loop {
                value |= ((block_data[i] as u8 & 127) as usize) << (varint_length * 7);
                varint_length += 1;
                if varint_length > 5 {
                    bail!("varint length too big (data probably corrupted)")
                }
                if (block_data[i] as u8) & 128 != 128 {
                    i += 1;
                    break;
                }
                i += 1;
            }

            let y = (index /  (format.width as i64 * format.length as i64)) as i64;
            let z = ((index % (format.width as i64 * format.length as i64)) / format.width as i64) as i64;
            let x = ((index % (format.width as i64 * format.length as i64)) % format.width as i64) as i64;
            let state = palette.get(value)
                .ok_or_else(|| eyre!("invalid palette index"))?
                .clone()
                .ok_or_else(|| eyre!("missing palette index"))?;
            buffer.insert(
                Vector3::new3(
                    x,
                    y,
                    z,
                ),
                state
            );

            index += 1;
        }

        Ok(buffer)
    }

    pub fn blocks(&self) -> impl Iterator<Item=(&Vector3<i64>, &Rc<BlockState>)> {
        self.block_data.iter()
    }

    pub fn blocks_mut(&mut self) -> impl Iterator<Item=(&Vector3<i64>, &mut Rc<BlockState>)> {
        self.block_data.iter_mut()
    }

    pub fn len_x(&self) -> usize {
        (self.max_x() - self.min_x()) as usize
    }
    pub fn len_y(&self) -> usize {
        (self.max_y() - self.min_y()) as usize
    }
    pub fn len_z(&self) -> usize {
        (self.max_z() - self.min_z()) as usize
    }

    pub fn height(&self) -> usize {self.len_y()}
    pub fn width(&self) -> usize {self.len_x()}
    pub fn length(&self) -> usize {self.len_z()}

    fn min(&self, f: impl Fn(&Vector3<i64>) -> i64) -> Option<i64> {
        self.block_data.keys()
            .min_by_key(|i| f(i))
            .map(f)
    }

    fn max(&self, f: impl Fn(&Vector3<i64>) -> i64) -> Option<i64> {
        self.block_data.keys()
            .max_by_key(|i| f(i))
            .map(f)
    }

    pub fn min_x(&self) -> i64 {
        self.min(|i| *i.x()).unwrap_or(0)
    }

    pub fn max_x(&self) -> i64 {
        self.max(|i| *i.x()).map(|i| i + 1).unwrap_or(0)
    }

    pub fn min_y(&self) -> i64 {
        self.min(|i| *i.y()).unwrap_or(0)
    }

    pub fn max_y(&self) -> i64 {
        self.max(|i| *i.y()).map(|i| i + 1).unwrap_or(0)
    }

    pub fn min_z(&self) -> i64 {
        self.min(|i| *i.z()).unwrap_or(0)
    }

    pub fn max_z(&self) -> i64 {
        self.max(|i| *i.z()).map(|i| i + 1).unwrap_or(0)
    }
}

pub struct Block {
    x: usize,
    y: usize,
    z: usize,
    data: u8,
}

