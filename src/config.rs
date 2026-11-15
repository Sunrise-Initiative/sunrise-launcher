use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};

use serde::{Deserialize, Serialize};

const SCHEMA: u32 = 1;
const MAX_BYTES: u64 = 128 * 1024;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Modern,
    Aero,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Light,
    #[default]
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Cards,
    List,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    schema: u32,
    pub theme: Theme,
    pub mode: Mode,
    pub view: View,
    #[serde(default)]
    pub reduce_motion: bool,
    pub links: BTreeMap<String, PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            theme: Theme::default(),
            mode: Mode::default(),
            view: View::default(),
            reduce_motion: false,
            links: BTreeMap::new(),
        }
    }
}

pub fn read(path: &Path) -> io::Result<Settings> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Settings::default()),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "arquivo de preferências excede o limite",
        ));
    }
    let settings: Settings = serde_json::from_slice(&bytes)?;
    if settings.schema != SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "versão de preferências incompatível",
        ));
    }
    Ok(settings)
}

pub fn save(path: &Path, previous: &Settings, next: &Settings) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("pasta de preferências indisponível"))?;
    let bytes = serde_json::to_vec_pretty(next)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::other("arquivo de preferências excede o limite"));
    }
    fs::create_dir_all(parent)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path.with_extension("lock"))?;
    lock.try_lock()
        .map_err(|error| io::Error::other(format!("preferências em uso: {error}")))?;
    if read(path)? != *previous {
        return Err(io::Error::other(
            "outra instância salvou preferências. Reabra o launcher para editar",
        ));
    }
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}

fn validate_path(path: &Path) -> io::Result<()> {
    let local = path.is_absolute()
        && !path
            .components()
            .any(|component| component == Component::ParentDir);
    #[cfg(windows)]
    let local = local
        && matches!(path.components().next(), Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))
        && path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    if !local {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selecione um executável local com caminho absoluto",
        ));
    }
    Ok(())
}

pub fn executable(path: &Path) -> io::Result<PathBuf> {
    validate_path(path)?;
    let resolved = path.canonicalize()?;
    validate_path(&resolved)?;
    if !resolved.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "o caminho não é um arquivo",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if resolved.metadata()?.permissions().mode() & 0o111 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "o arquivo não possui permissão de execução",
            ));
        }
    }
    Ok(resolved)
}

pub fn open(path: &Path) -> io::Result<()> {
    let path = executable(path)?;
    let mut command = Command::new(&path);
    if let Some(parent) = path.parent() {
        command.current_dir(parent);
    }
    let mut child = command.spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_stale_writer_preserve_last_commit() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        fs::write(
            &path,
            br#"{"schema":1,"theme":"modern","mode":"dark","view":"cards","links":{}}"#,
        )
        .unwrap();
        let initial = read(&path).unwrap();
        assert!(!initial.reduce_motion);
        let changed = Settings {
            theme: Theme::Aero,
            view: View::List,
            reduce_motion: true,
            ..initial.clone()
        };
        save(&path, &initial, &changed).unwrap();
        assert_eq!(read(&path).unwrap(), changed);
        assert!(save(&path, &initial, &initial).is_err());
        assert_eq!(read(&path).unwrap(), changed);
    }

    #[test]
    fn invalid_settings_are_preserved() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("settings.json");
        for contents in [
            b"{malformed".as_slice(),
            br#"{"schema":2,"theme":"modern","mode":"dark","view":"cards","links":{}}"#,
        ] {
            fs::write(&path, contents).unwrap();
            assert!(read(&path).is_err());
            assert!(save(&path, &Settings::default(), &Settings::default()).is_err());
            assert_eq!(fs::read(&path).unwrap(), contents);
        }
    }

    #[test]
    fn executable_rejects_relative_missing_and_directories() {
        let folder = tempfile::tempdir().unwrap();
        assert!(executable(Path::new("app.exe")).is_err());
        assert!(executable(&folder.path().join("missing.exe")).is_err());
        let directory = folder.path().join("directory.exe");
        fs::create_dir(&directory).unwrap();
        assert!(executable(&directory).is_err());
        assert!(executable(&std::env::current_exe().unwrap()).is_ok());
        #[cfg(windows)]
        assert!(executable(Path::new(r"\\host\share\app.exe")).is_err());
    }
}
