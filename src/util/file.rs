use core::convert::TryFrom;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, io};

use lnpbp::rgb::prelude::*;
use lnpbp::strict_encoding::{Error, StrictDecode, StrictEncode};

use super::MagicNumber;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub enum FileMode {
    Read,
    Write,
    Create,
}

#[inline]
pub fn file(filename: PathBuf, mode: FileMode) -> Result<fs::File, io::Error> {
    match mode {
        FileMode::Read => fs::File::open(filename),
        FileMode::Write => fs::OpenOptions::new().write(true).open(filename),
        FileMode::Create => fs::File::create(filename),
    }
}

pub fn read_file(filename: PathBuf) -> Result<(u32, Vec<u8>), io::Error> {
    let mut data = vec![];
    let mut file = file(filename, FileMode::Read)?;
    let mut magic_buf = [0u8; 4];
    file.read_exact(&mut magic_buf)?;
    let magic = u32::from_be_bytes(magic_buf);
    file.read_to_end(&mut data)?;
    Ok((magic, data))
}

pub fn read_dir_filenames(
    dir: PathBuf,
    filter_extensions: Option<&str>,
) -> Result<Vec<String>, io::Error> {
    let mut list = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = filter_extensions {
            if ext
                != path.extension().map(|s| s.to_str().unwrap()).unwrap_or("")
            {
                continue;
            }
        }
        if !path.is_dir() {
            if let Some(name) =
                path.file_name().map(|s| s.to_str().unwrap().to_string())
            {
                list.push(name);
            }
        }
    }
    Ok(list)
}

pub trait ReadWrite
where
    Self: Sized,
{
    fn read_file(filename: PathBuf) -> Result<Self, Error>;
    fn write_file(&self, filename: PathBuf) -> Result<usize, Error>;
}

impl ReadWrite for Schema {
    fn read_file(filename: PathBuf) -> Result<Self, Error> {
        let mut file = file(filename, FileMode::Read)?;
        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        let magic = u32::from_be_bytes(magic_buf);
        let magic = MagicNumber::try_from(magic).map_err(|detected| {
            Error::DataIntegrityError(format!(
                "Wrong file type: expected schema file, got unknown magic number {}",
                detected
            ))
        })?;
        if magic != MagicNumber::Schema {
            Err(Error::DataIntegrityError(format!(
                "Wrong file type: expected schema file, got {}",
                magic
            )))?
        }
        Schema::strict_decode(file)
    }

    fn write_file(&self, filename: PathBuf) -> Result<usize, Error> {
        let mut file = file(filename, FileMode::Create)?;
        file.write(&MagicNumber::Schema.to_u32().to_be_bytes())?;
        self.strict_encode(file)
    }
}

impl ReadWrite for Genesis {
    fn read_file(filename: PathBuf) -> Result<Self, Error> {
        let mut file = file(filename, FileMode::Read)?;
        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        let magic = u32::from_be_bytes(magic_buf);
        let magic = MagicNumber::try_from(magic).map_err(|detected| {
            Error::DataIntegrityError(format!(
                "Wrong file type: expected genesis file, got unknown magic number {}",
                detected
            ))
        })?;
        if magic != MagicNumber::Genesis {
            Err(Error::DataIntegrityError(format!(
                "Wrong file type: expected genesis file, got {}",
                magic
            )))?
        }
        Genesis::strict_decode(file)
    }

    fn write_file(&self, filename: PathBuf) -> Result<usize, Error> {
        let mut file = file(filename, FileMode::Create)?;
        file.write(&MagicNumber::Genesis.to_u32().to_be_bytes())?;
        self.strict_encode(file)
    }
}

impl ReadWrite for Anchor {
    fn read_file(filename: PathBuf) -> Result<Self, Error> {
        let mut file = file(filename, FileMode::Read)?;
        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        let magic = u32::from_be_bytes(magic_buf);
        let magic = MagicNumber::try_from(magic).map_err(|detected| {
            Error::DataIntegrityError(format!(
                "Wrong file type: expected anchor file, got unknown magic number {}",
                detected
            ))
        })?;
        if magic != MagicNumber::Anchor {
            Err(Error::DataIntegrityError(format!(
                "Wrong file type: expected anchor file, got {}",
                magic
            )))?
        }
        Anchor::strict_decode(file)
    }

    fn write_file(&self, filename: PathBuf) -> Result<usize, Error> {
        let mut file = file(filename, FileMode::Create)?;
        file.write(&MagicNumber::Anchor.to_u32().to_be_bytes())?;
        self.strict_encode(file)
    }
}

impl ReadWrite for Transition {
    fn read_file(filename: PathBuf) -> Result<Self, Error> {
        let mut file = file(filename, FileMode::Read)?;
        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        let magic = u32::from_be_bytes(magic_buf);
        let magic = MagicNumber::try_from(magic).map_err(|detected| {
            Error::DataIntegrityError(format!(
                "Wrong file type: expected state transition file, got unknown magic number {}",
                detected
            ))
        })?;
        if magic != MagicNumber::Transition {
            Err(Error::DataIntegrityError(format!(
                "Wrong file type: expected state transition file, got {}",
                magic
            )))?
        }
        Transition::strict_decode(file)
    }

    fn write_file(&self, filename: PathBuf) -> Result<usize, Error> {
        let mut file = file(filename, FileMode::Create)?;
        file.write(&MagicNumber::Transition.to_u32().to_be_bytes())?;
        self.strict_encode(file)
    }
}

impl ReadWrite for Extension {
    fn read_file(filename: PathBuf) -> Result<Self, Error> {
        let mut file = file(filename, FileMode::Read)?;
        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        let magic = u32::from_be_bytes(magic_buf);
        let magic = MagicNumber::try_from(magic).map_err(|detected| {
            Error::DataIntegrityError(format!(
                "Wrong file type: expected state extension file, got unknown magic number {}",
                detected
            ))
        })?;
        if magic != MagicNumber::Extension {
            Err(Error::DataIntegrityError(format!(
                "Wrong file type: expected state extension file, got {}",
                magic
            )))?
        }
        Extension::strict_decode(file)
    }

    fn write_file(&self, filename: PathBuf) -> Result<usize, Error> {
        let mut file = file(filename, FileMode::Create)?;
        file.write(&MagicNumber::Extension.to_u32().to_be_bytes())?;
        self.strict_encode(file)
    }
}

impl ReadWrite for Consignment {
    fn read_file(filename: PathBuf) -> Result<Self, Error> {
        let mut file = file(filename, FileMode::Read)?;
        let mut magic_buf = [0u8; 4];
        file.read_exact(&mut magic_buf)?;
        let magic = u32::from_be_bytes(magic_buf);
        let magic = MagicNumber::try_from(magic).map_err(|detected| {
            Error::DataIntegrityError(format!(
                "Wrong file type: expected consignment file, got unknown magic number {}",
                detected
            ))
        })?;
        if magic != MagicNumber::Consignment {
            Err(Error::DataIntegrityError(format!(
                "Wrong file type: expected consignment file, got {}",
                magic
            )))?
        }
        Consignment::strict_decode(file)
    }

    fn write_file(&self, filename: PathBuf) -> Result<usize, Error> {
        let mut file = file(filename, FileMode::Create)?;
        file.write(&MagicNumber::Consignment.to_u32().to_be_bytes())?;
        self.strict_encode(file)
    }
}
