use std::{fs};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, SeekFrom, Write};
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
    let mut file_order: Vec<String> = Vec::new();

    for _i in 0..files{
        let var00 = reader.read_i32().unwrap();
        let var04 = reader.read_i32().unwrap();
        let var08 = reader.read_u64().unwrap();
        let var10 = reader.read_u64().unwrap();
        let var18 = reader.read_u64().unwrap();
        let var20 = reader.read_bytes(224).unwrap();
        let filename = String::from_utf8(var20).unwrap().replace("\0", "");
        file_order.push(filename.clone());

        entries.push(Entry {
            field00: var00,
            index: var04,
            offset: var08,
            compressed_size: var10,
            uncompressed_size: var18,
            file_path: filename.clone()
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

    let mut fo_f = File::create(Path::new(output_dir).join("_fileorder.txt"))?;
    for line in &file_order {
        let l = line.clone() + "\n";
        fo_f.write_all(l.as_bytes()).unwrap();
    }

    Ok(())
}

pub fn build(input_dir: &str) -> Result<(), std::io::Error> {
    let files_count = fs::read_dir(&input_dir).unwrap().count() - 1;
    assert_ne!(files_count, 0);

    let f = match File::open(Path::new(input_dir).join("_fileorder.txt")) {
        Err(e) => panic!("_fileorder.txt: {}", e),
        Ok(f) => f
    };
    let reader = BufReader::new(f);
    let files: Vec<_> = reader.lines().collect();

    let mpk_name = input_dir.to_string() + ".mpk";
    let mut data_start = 0x40 + (0x100 * files_count);
    if data_start % 0x800 != 0 {
        data_start += 0x800 - data_start % 0x800
    };
    let mut entries = Vec::<Entry>::new();

    let mpk_f = match File::create(&mpk_name) {
        Err(e) => panic!("{:?}", e),
        Ok(out_f) => out_f,
    };
    let mut writer = BinaryWriter::new(mpk_f, Options::default());

    let _ = writer.write_u32(0x4B504D);
    let _ = writer.write_u32(0x20000);
    let _ = writer.write_u64(files_count as u64);
    let _ = writer.seek(SeekFrom::Start(data_start as u64));

    let mut index = 0;
    for file in files {
        let path = Path::new(input_dir).join(file.unwrap());
        println!("Import: {}", &path.display());
        let mut f = match File::open(&path) {
            Err(e) => panic!("{}", e),
            Ok(f) => f,
        };
        let metadata = fs::metadata(&path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        let fname = path.file_name().unwrap();
        entries.push(Entry{
            field00: 0,
            index,
            offset: writer.stream_position().unwrap(),
            compressed_size: metadata.len(),
            uncompressed_size: metadata.len(),
            file_path: String::from(fname.to_str().unwrap())
        });

        let _ = writer.write_bytes(&buffer);

        let pos = writer.stream_position().unwrap();
        if pos % 0x800 != 0 {
            let _ = writer.seek(SeekFrom::Current((0x800 - pos % 0x800) as i64));
        }

        index += 1;
    }

    let _ = writer.seek(SeekFrom::Start(0x40));
    for entry in entries.iter(){
        let _ = writer.write_i32(&entry.field00);
        let _ = writer.write_i32(&entry.index);
        let _ = writer.write_u64(&entry.offset);
        let _ = writer.write_u64(&entry.compressed_size);
        let _ = writer.write_u64(&entry.uncompressed_size);
        let mut buf = vec![0u8; 224];
        let str_buf = &entry.file_path.as_bytes();
        buf[..str_buf.len()].copy_from_slice(&str_buf);
        let _ = writer.write_bytes(&buf);
    }

    Ok(())
}