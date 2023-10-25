use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{arg, value_parser, ArgMatches, Command};
use deltas::patch::Patch;
use huffman_coding::HuffmanTree;

fn get_command() -> ArgMatches {
    Command::new("Diff")
        .args(&[
            arg!(<Source> "Path to the original version of the file")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
            arg!(<Target> "Path to the new version of the file")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
            arg!([Patch] "Path to save the patch file at")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
            arg!(-c --compress "Whether or not to huffman compress the patch file."),
        ])
        .get_matches()
}

fn get_patch(source: &Path, target: &Path) -> io::Result<Patch> {
    let source = fs::read(source)?;
    let target = fs::read(target)?;
    Ok(Patch::new(&source, &target))
}

fn compress(bytes: &[u8]) -> io::Result<(Vec<u8>, HuffmanTree)> {
    let tree = huffman_coding::HuffmanTree::from_data(bytes);
    let mut output = Vec::with_capacity(bytes.len());
    {
        let mut writer = huffman_coding::HuffmanWriter::new(&mut output, &tree);
        assert!(writer.write(&bytes).is_ok());
    }
    Ok((output, tree))
}

fn main() -> io::Result<()> {
    let matches = get_command();
    let source: &PathBuf = matches.get_one("Source").unwrap();
    let target: &PathBuf = matches.get_one("Target").unwrap();
    let patch = matches
        .get_one("Patch")
        .cloned()
        .unwrap_or(PathBuf::from("patch.diff"));
    let should_compress = matches.get_flag("compress");
    if should_compress {
        let (bytes, tree) = compress(&get_patch(&source, &target)?.to_bytes())?;
        fs::write(&patch, bytes)?;
        fs::write(&patch.with_extension("tree"), tree.to_table())?;
    } else {
        fs::write(patch, get_patch(&source, &target)?.to_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn patch_size() {
        let source = fs::read("files/source.txt").unwrap();
        let target = fs::read("files/target.txt").unwrap();
        let patch = fs::read("files/patch2.diff.bz2").unwrap();
        dbg!(source.len(), target.len(), patch.len());
    }
}
