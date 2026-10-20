use super::*;

const MAX_TREE_ENTRIES: usize = 8192;
const MAX_TREE_DEPTH: usize = 16;
impl WorkspaceKnowledgeCatalogue {
    pub(super) fn normal_path(&self, name: &str) -> Result<String, String> {
        if name.is_empty() || Path::new(name).is_absolute() || !name.ends_with(".eal") {
            return Err("path must be a workspace-relative .eal file".into());
        }
        let path = crate::acquisition::bounded_path(self.service.workspace(), name)
            .map_err(|e| e.to_string())?;
        let relative = path
            .strip_prefix(self.service.workspace())
            .map_err(|e| e.to_string())?;
        relative
            .to_str()
            .map(|name| name.replace(std::path::MAIN_SEPARATOR, "/"))
            .ok_or("entry_id must be valid UTF-8".into())
    }
    pub(super) fn read_source(&self, relative: &str) -> Result<String, String> {
        let path = crate::acquisition::bounded_path(self.service.workspace(), relative)
            .map_err(|e| e.to_string())?;
        if !std::fs::metadata(&path)
            .map_err(|e| e.to_string())?
            .is_file()
        {
            return Err("Registered source must be a regular EAL file".into());
        }
        let bytes = crate::acquisition::read_regular(
            &path,
            self.service.limits().source_bytes,
            "Registered source must be a regular EAL file",
        )
        .map_err(|e| e.to_string())?;
        if bytes.len() > self.service.limits().source_bytes {
            return Err("Registered source exceeds the EAL source limit".into());
        }
        String::from_utf8(bytes).map_err(|_| "Registered source must be UTF-8".into())
    }
    pub fn register_tree(
        &self,
        directory: &str,
        context: Option<&Value>,
        limit: usize,
    ) -> Result<Value, String> {
        if !(1..=MAX_TREE_FILES).contains(&limit) {
            return Err(format!(
                "limit must be an integer from 1 through {MAX_TREE_FILES}"
            ));
        }
        if Path::new(directory).is_absolute() {
            return Err("directory must be workspace-relative".into());
        }
        let root = crate::acquisition::bounded_path(self.service.workspace(), directory)
            .map_err(|e| e.to_string())?;
        if !root.is_dir() {
            return Err("directory must resolve to a workspace directory".into());
        }
        let mut paths = Vec::new();
        let mut visited = 0;
        walk(&root, 0, limit, &mut visited, &mut paths)?;
        paths.sort();
        let mut registered = Vec::new();
        let mut rejected = Vec::new();
        for path in paths {
            let relative = path
                .strip_prefix(self.service.workspace())
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("Catalogue paths must be UTF-8")?
                .replace(std::path::MAIN_SEPARATOR, "/");
            match self.register(
                &relative,
                RegisterOptions {
                    context,
                    ..RegisterOptions::default()
                },
            ) {
                Ok(entry) => registered.push(entry),
                Err(reason) => rejected.push(json!({"path":relative,"reason":reason})),
            }
        }
        Ok(json!({"registered":registered,"rejected":rejected}))
    }
}
fn walk(
    directory: &Path,
    depth: usize,
    limit: usize,
    visited: &mut usize,
    paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut directories = Vec::new();
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() || (kind.is_symlink() && entry.path().is_dir()) {
            directories.push((entry.path(), kind.is_symlink()));
        } else {
            files.push(entry.path());
        }
    }
    directories.sort();
    files.sort();
    *visited += directories.len() + files.len();
    if *visited > MAX_TREE_ENTRIES {
        return Err("Directory exceeds the discovery scan limit".into());
    }
    if depth >= MAX_TREE_DEPTH && !directories.is_empty() {
        return Err("Directory exceeds the discovery depth limit".into());
    }
    for path in files {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".eal"))
        {
            paths.push(path);
            if paths.len() > limit {
                return Err("Directory contains more EAL files than the discovery limit".into());
            }
        }
    }
    for (path, symlink) in directories {
        if !symlink {
            walk(&path, depth + 1, limit, visited, paths)?;
        }
    }
    Ok(())
}
