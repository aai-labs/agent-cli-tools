use std::{
    env, fs,
    io::{self, Seek, Write},
    path::{Path, PathBuf},
};

use zip::{write::SimpleFileOptions, ZipWriter};

fn main() {
    println!("cargo:rerun-if-changed=bundled/skills");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let archive_path = out_dir.join("aai-skills.zip");

    if let Err(err) = write_archive(Path::new("bundled/skills"), &archive_path) {
        panic!("failed to package bundled skills: {err}");
    }
}

fn write_archive(source_dir: &Path, archive_path: &Path) -> io::Result<()> {
    let archive = fs::File::create(archive_path)?;
    let mut writer = ZipWriter::new(archive);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for path in list_files(source_dir)? {
        let relative = path
            .strip_prefix(source_dir)
            .expect("listed files are under source dir");
        let archive_name = relative.to_string_lossy().replace('\\', "/");
        writer.start_file(archive_name, options)?;
        let mut file = fs::File::open(path)?;
        io::copy(&mut file, &mut writer)?;
    }

    finish_zip(writer)?;
    Ok(())
}

fn list_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
            files.push(path);
        }
    }
    Ok(())
}

fn finish_zip<W: Write + Seek>(writer: ZipWriter<W>) -> io::Result<()> {
    writer.finish().map(|_| ()).map_err(io::Error::other)
}
