use std::fs;
use std::fs::File;
use std::io::{SeekFrom};
use std::path::Path;
use binary_stream::{BinaryReader, BinaryWriter, Options};

pub struct Entry {
    pub field00: i32,
    pub index: i32,
    pub offset: u64,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub file_path: String
}

pub fn extract(mpk_path: &str) -> Result<(), std::io::Error> {
    let f = match File::open(mpk_path) {
        Err(e) => panic!("{}", e),
        Ok(f) => f,
    };
    let mut reader = BinaryReader::new(f, Options::default());

    let mut entries = Vec::<Entry>::new();

    let magic = reader.read_u32().unwrap();
    let field04 = reader.read_u32().unwrap();

    if magic != 0x4B504D {
        panic!("Unexpected magic.");
    }
    if field04 != 0x20000 {
        panic!("field00 error.");
    }

    let files = reader.read_u64().unwrap();
    let _ = reader.seek(SeekFrom::Current(0x30));

    for _i in 0..files{
        let var00 = reader.read_i32().unwrap();
        let var04 = reader.read_i32().unwrap();
        let var08 = reader.read_u64().unwrap();
        let var10 = reader.read_u64().unwrap();
        let var18 = reader.read_u64().unwrap();
        let var20 = reader.read_bytes(224).unwrap();

        entries.push(Entry {
            field00: var00,
            index: var04,
            offset: var08,
            compressed_size: var10,
            uncompressed_size: var18,
            file_path: String::from_utf8(var20).unwrap().replace("\0", "")
        });
    }
    let output_dir = Path::new(mpk_path).file_stem().unwrap();
    let _ = fs::create_dir_all(output_dir);

    for entry in entries.iter() {
        if entry.compressed_size != entry.uncompressed_size{
            println!("Compressed entry will be skipped");
            continue;
        }
        let _ = reader.seek(SeekFrom::Start(entry.offset));
        let buffer = reader.read_bytes(entry.uncompressed_size as usize);
        let path = Path::new(output_dir).join(&entry.file_path);
        println!("Extracting: {}", &path.display());
        let out_f = match File::create(&path) {
            Err(e) => panic!("{:?}", e),
            Ok(out_f) => out_f,
        };

        let mut writer = BinaryWriter::new(out_f, Options::default());
        let _ = writer.write_bytes(buffer.unwrap());
    }

    Ok(())
}