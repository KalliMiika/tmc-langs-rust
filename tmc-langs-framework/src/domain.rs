mod meta_syntax;

use super::LanguagePlugin;
use lazy_static::lazy_static;
use log::{debug, info};
use meta_syntax::{MetaString, MetaSyntaxParser};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

lazy_static! {
    static ref FILES_TO_SKIP_ALWAYS: Regex =
        Regex::new("\\.tmcrc|metadata\\.yml|(.*)Hidden(.*)").unwrap();
    static ref NON_TEXT_TYPES: Regex =
        Regex::new("class|jar|exe|jpg|jpeg|gif|png|zip|tar|gz|db|bin|csv|tsv|^$").unwrap();
}

pub struct TestDesc {}

pub struct ExerciseDesc {}

pub struct RunResult {}

pub struct ValidationResult {}

pub struct ExercisePackagingConfiguration {}

fn is_hidden_dir(entry: &DirEntry) -> bool {
    let skip = entry.metadata().map(|e| e.is_dir()).unwrap_or(false)
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false);
    if skip {
        debug!("is hidden dir: {:?}", entry.path());
    }
    skip
}

fn on_skip_list(entry: &DirEntry) -> bool {
    let skip = entry
        .file_name()
        .to_str()
        .map(|s| FILES_TO_SKIP_ALWAYS.is_match(s) || s == "private")
        .unwrap_or(false);
    if skip {
        debug!("on skip list: {:?}", entry.path());
    }
    skip
}

fn contains_tmcignore(entry: &DirEntry) -> bool {
    for entry in WalkDir::new(entry.path())
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let is_file = entry.metadata().map(|e| e.is_file()).unwrap_or(false);
        if is_file {
            let is_tmcignore = entry
                .file_name()
                .to_str()
                .map(|s| s == ".tmcignore")
                .unwrap_or(false);
            if is_tmcignore {
                debug!("contains .tmcignore: {:?}", entry.path());
                return true;
            }
        }
    }
    false
}

fn copy_file<F: Fn(&MetaString) -> bool>(
    entry: &DirEntry,
    skip_parts: usize,
    dest_root: &Path,
    filter: &mut F,
) -> io::Result<()> {
    let is_dir = entry.metadata().map(|e| e.is_dir()).unwrap_or(false);
    if is_dir {
        return Ok(());
    }
    // get relative path
    let relative_path = entry
        .path()
        .into_iter()
        .skip(skip_parts)
        .collect::<PathBuf>();
    let dest_path = dest_root.join(&relative_path);
    dest_path
        .parent()
        .map_or(Ok(()), |p| fs::create_dir_all(p))?;
    let extension = entry
        .path()
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let is_binary = NON_TEXT_TYPES.is_match(extension);
    if is_binary {
        // copy binary files
        debug!(
            "copying binary file from {:?} to {:?}",
            entry.path(),
            dest_path
        );
        fs::copy(entry.path(), dest_path)?;
    } else {
        // filter text files
        debug!(
            "filtering text file from {:?} to {:?}",
            entry.path(),
            dest_path
        );
        let source_file = File::open(entry.path())?;
        let mut target_file = File::create(dest_path)?;
        let parser = MetaSyntaxParser::new(source_file, extension);
        for line in parser {
            let line = line?;
            if filter(&line) {
                debug!("write: {:?}", line);
                target_file.write_all(line.as_str().as_bytes())?;
            } else {
                debug!("skip: {:?}", line);
            }
        }
    }
    Ok(())
}

fn process_files<F: Fn(&MetaString) -> bool>(
    path: &Path,
    dest_root: &Path,
    mut filter: F,
) -> io::Result<()> {
    info!("Project: {:?}", path);

    let skip_parts = path.components().count(); // used to get the relative path of files
    let walker = WalkDir::new(path).into_iter();
    // silently skips over errors, for example when there's a directory we don't have permissions for
    for entry in walker
        .filter_entry(|e| !is_hidden_dir(e) && !on_skip_list(e) && !contains_tmcignore(e))
        .filter_map(|e| e.ok())
    {
        copy_file(&entry, skip_parts, dest_root, &mut filter)?;
    }
    Ok(())
}

/// Walks through each path in ```exercise_map```, processing files and copying them into ```dest_path```.
/// Skips hidden directories, directories that contain a ```.tmcignore``` file in their root, as well as files matching patterns defined in ```FILES_TO_SKIP_ALWAYS``` and directories and files named ```private```.
/// Binary files are copied without extra processing, while text files have solution tags and stubs removed.
pub fn prepare_solutions<'a, I: IntoIterator<Item = &'a PathBuf>>(
    exercise_paths: I,
    dest_root: &Path,
) -> io::Result<()> {
    for path in exercise_paths {
        process_files(path, dest_root, |meta| match meta {
            MetaString::Stub(_) => false,
            _ => true,
        })?;
    }
    Ok(())
}

/// Walks through each path in ```exercise_map```, processing files and copying them into ```dest_path```.
/// Skips hidden directories, directories that contain a ```.tmcignore``` file in their root, as well as files matching patterns defined in ```FILES_TO_SKIP_ALWAYS``` and directories and files named ```private```.
/// Binary files are copied without extra processing, while text files have solution tags and stubs removed.
pub fn prepare_stubs(
    exercise_map: HashMap<PathBuf, Box<dyn LanguagePlugin>>,
    repo_path: &Path,
    dest_root: &Path,
) -> io::Result<()> {
    for (path, plugin) in exercise_map {
        process_files(&path, dest_root, |meta| match meta {
            MetaString::Solution(_) => false,
            _ => true,
        })?;

        let relative_path = if repo_path.components().count() < path.iter().count() {
            let skip_count = repo_path.components().count();
            path.components().into_iter().skip(skip_count).collect()
        } else {
            PathBuf::from("")
        };
        plugin.maybe_copy_shared_stuff(&dest_root.join(relative_path));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use std::io::Read;
    use tempdir::TempDir;

    const TESTDATA_ROOT: &str = "testdata";
    const BINARY_REL: &str = "dir/inner/binary.bin";
    const TEXT_REL: &str = "dir/nonbinary.java";

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn prepare_solutions_preserves_structure() {
        init();

        let mut exercise_set = HashSet::new();
        exercise_set.insert(TESTDATA_ROOT.into());
        let temp = TempDir::new("prepare_solutions_preserves_structure").unwrap();
        let temp_path = temp.path();

        prepare_solutions(&exercise_set, temp_path).unwrap();

        let mut dest_files = HashSet::new();
        for entry in walkdir::WalkDir::new(temp_path) {
            let entry = entry.unwrap();
            dest_files.insert(entry.into_path());
        }

        let exp = &temp_path.join(BINARY_REL);
        assert!(
            dest_files.contains(exp),
            "{:?} did not contain {:?}",
            dest_files,
            exp
        );
        let exp = &temp_path.join(TEXT_REL);
        assert!(
            dest_files.contains(exp),
            "{:?} did not contain {:?}",
            dest_files,
            exp
        );
    }

    #[test]
    fn prepare_solutions_filters_text_files() {
        init();

        let mut exercise_set = HashSet::new();
        exercise_set.insert(TESTDATA_ROOT.into());
        let temp = TempDir::new("prepare_solutions_filters_text_files").unwrap();
        let temp_path = temp.path();

        prepare_solutions(&exercise_set, temp_path).unwrap();

        let exp = &temp_path.join(TEXT_REL);
        let mut file = File::open(exp).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        let expected = r#"public class JavaTestCase {
    public int foo() {
        return 3;
    }

    public void bar() {
        System.out.println("hello");
    }

    public int xoo() {
        return 3;
    }
}
"#;
        assert_eq!(s, expected, "expected:\n{:#}\nfound:\n{:#}", expected, s);
    }

    #[test]
    fn prepare_solutions_does_not_filter_binary_files() {
        init();

        let mut exercise_set = HashSet::new();
        exercise_set.insert(TESTDATA_ROOT.into());
        let temp = TempDir::new("prepare_solutions_does_not_filter_binary_files").unwrap();
        let temp_path = temp.path();

        prepare_solutions(&exercise_set, temp_path).unwrap();

        let original: PathBuf = [TESTDATA_ROOT, BINARY_REL].iter().collect();
        let mut original = File::open(original).unwrap();
        let mut original_s = String::new();
        original.read_to_string(&mut original_s).unwrap();

        let copied = &temp_path.join(BINARY_REL);
        let mut copied = File::open(copied).unwrap();
        let mut copied_s = String::new();
        copied.read_to_string(&mut copied_s).unwrap();

        assert_eq!(
            original_s, copied_s,
            "expected:\n{:#}\nfound:\n{:#}",
            copied_s, original_s
        );
    }

    struct MockPlugin {}

    impl LanguagePlugin for MockPlugin {
        fn maybe_copy_shared_stuff(&self, _path: &Path) {}
    }

    #[test]
    fn prepares_stubs() {
        init();

        let mut exercise_map = HashMap::new();
        exercise_map.insert(
            TESTDATA_ROOT.into(),
            Box::new(MockPlugin {}) as Box<dyn LanguagePlugin>,
        );
        let temp = TempDir::new("prepares_stubs").unwrap();
        let temp_path = temp.path();

        let repo_path: PathBuf = "".into();
        prepare_stubs(exercise_map, &repo_path, &temp_path).unwrap();

        let exp = &temp_path.join(TEXT_REL);
        let mut file = File::open(exp).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        let expected = r#"public class JavaTestCase {

    public void bar() {
    }

    public int xoo() {
        return 0;
    }
}
"#;

        assert_eq!(s, expected, "expected:\n{:#}\nfound:\n{:#}", expected, s);
    }
}
