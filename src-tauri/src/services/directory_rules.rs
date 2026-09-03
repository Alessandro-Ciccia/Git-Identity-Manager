use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::process::{ProcessErrorKind, ProcessRunner};

use super::{
    git::{GitConfigScope, GitService},
    profiles::{GitProfile, ProfileError, ProfilesService},
    repositories::{RepositoriesService, RepositoryError},
};

const STORE_VERSION: u32 = 1;
const MAX_PATH_CHARS: usize = 32_768;
const MANAGED_BEGIN: &str = "# >>> Git Identity Manager directory rules >>>";
const MANAGED_END: &str = "# <<< Git Identity Manager directory rules <<<";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DirectoryRule {
    pub(crate) id: String,
    pub(crate) directory: String,
    pub(crate) profile_id: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DirectoryRuleInput {
    pub(crate) id: Option<String>,
    pub(crate) directory: String,
    pub(crate) profile_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResolvedDirectoryRule {
    pub(crate) id: String,
    pub(crate) directory: String,
    pub(crate) profile_id: String,
    pub(crate) state: DirectoryRuleState,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DirectoryRuleState {
    Active,
    NeedsApply,
    MissingDirectory,
    MissingProfile,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DirectoryRulePreview {
    pub(crate) rule_id: Option<String>,
    pub(crate) directory: String,
    pub(crate) profile: Option<GitProfile>,
    pub(crate) operation: DirectoryRuleOperation,
    pub(crate) condition: String,
    pub(crate) identity_file_path: String,
    pub(crate) global_config_path: String,
    pub(crate) backup_required: bool,
    pub(crate) conflicts: Vec<DirectoryRuleConflict>,
    pub(crate) can_apply: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DirectoryRuleOperation {
    Add,
    Update,
    Remove,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DirectoryRuleConflict {
    pub(crate) kind: DirectoryRuleConflictKind,
    pub(crate) message: String,
    pub(crate) blocking: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum DirectoryRuleConflictKind {
    OverlappingRule,
    ExistingConditionalInclude,
    RepositoryOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectoryRuleError {
    StorageUnavailable,
    StorageReadFailed,
    StorageWriteFailed,
    MalformedData,
    InvalidId,
    InvalidPath,
    PathNotFound,
    NotDirectory,
    Profile(ProfileError),
    Repository(RepositoryError),
    NotFound,
    GitMissing,
    GitCommandFailed,
    GitConfigMalformed,
    Conflict,
    WriteFailed,
    VerificationFailed,
    RollbackFailed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryRulesStore {
    version: u32,
    rules: Vec<DirectoryRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IncludeEntry {
    condition: String,
    target: String,
}

pub(crate) struct DirectoryRulesService<R> {
    store_path: Option<PathBuf>,
    identity_directory: Option<PathBuf>,
    backup_directory: Option<PathBuf>,
    global_config_candidates: Vec<PathBuf>,
    runner: R,
}

impl<R: ProcessRunner + Clone> DirectoryRulesService<R> {
    pub(crate) fn new(
        store_path: Option<PathBuf>,
        identity_directory: Option<PathBuf>,
        backup_directory: Option<PathBuf>,
        global_config_candidates: Vec<PathBuf>,
        runner: R,
    ) -> Self {
        Self {
            store_path,
            identity_directory,
            backup_directory,
            global_config_candidates,
            runner,
        }
    }

    pub(crate) fn list(
        &self,
        profiles: &ProfilesService,
    ) -> Result<Vec<ResolvedDirectoryRule>, DirectoryRuleError> {
        let store = self.load()?;
        Ok(store
            .rules
            .into_iter()
            .map(|rule| self.resolve(rule, profiles))
            .collect())
    }

    pub(crate) fn preview(
        &self,
        input: DirectoryRuleInput,
        profiles: &ProfilesService,
        repositories: &RepositoriesService,
    ) -> Result<DirectoryRulePreview, DirectoryRuleError> {
        let rule_id = match input.id.as_deref() {
            Some(id) => {
                validate_id(id)?;
                Some(id.to_owned())
            }
            None => None,
        };
        let directory = canonical_directory(&input.directory)?;
        validate_id(&input.profile_id)?;
        let profile = profiles
            .find(&input.profile_id)
            .map_err(DirectoryRuleError::Profile)?;
        let store = self.load()?;
        if let Some(id) = &rule_id {
            if !store.rules.iter().any(|rule| &rule.id == id) {
                return Err(DirectoryRuleError::NotFound);
            }
        }

        let global_config = self.global_config_path()?;
        let identity_file = self.identity_file_path(&profile.id)?;
        let condition = gitdir_condition(&directory);
        let mut conflicts = self.rule_conflicts(
            &store.rules,
            rule_id.as_deref(),
            &directory,
            &profile.id,
            &global_config,
        )?;
        conflicts.extend(self.repository_conflicts(&directory, repositories)?);
        let can_apply = conflicts.iter().all(|conflict| !conflict.blocking);

        Ok(DirectoryRulePreview {
            rule_id: rule_id.clone(),
            directory,
            profile: Some(profile),
            operation: if rule_id.is_some() {
                DirectoryRuleOperation::Update
            } else {
                DirectoryRuleOperation::Add
            },
            condition,
            identity_file_path: path_text(&identity_file)?,
            global_config_path: path_text(&global_config)?,
            backup_required: global_config.exists() || identity_file.exists(),
            conflicts,
            can_apply,
        })
    }

    pub(crate) fn apply(
        &self,
        input: DirectoryRuleInput,
        profiles: &ProfilesService,
        repositories: &RepositoriesService,
    ) -> Result<ResolvedDirectoryRule, DirectoryRuleError> {
        let preview = self.preview(input, profiles, repositories)?;
        if !preview.can_apply {
            return Err(DirectoryRuleError::Conflict);
        }
        let profile = preview
            .profile
            .clone()
            .ok_or(DirectoryRuleError::Profile(ProfileError::NotFound))?;

        let mut store = self.load()?;
        let store_path = self.store_path()?;
        let store_before = read_optional(store_path)?;
        let rule = match preview.rule_id.as_deref() {
            Some(id) => {
                let existing = store
                    .rules
                    .iter_mut()
                    .find(|rule| rule.id == id)
                    .ok_or(DirectoryRuleError::NotFound)?;
                existing.directory = preview.directory.clone();
                existing.profile_id = profile.id.clone();
                existing.clone()
            }
            None => {
                let rule = DirectoryRule {
                    id: Uuid::new_v4().to_string(),
                    directory: preview.directory.clone(),
                    profile_id: profile.id.clone(),
                };
                store.rules.push(rule.clone());
                rule
            }
        };

        let global_path = self.global_config_path()?;
        let identity_path = self.identity_file_path(&profile.id)?;
        let global_before = read_optional(&global_path)?;
        let identity_before = read_optional(&identity_path)?;
        let global_text = String::from_utf8(global_before.clone().unwrap_or_default())
            .map_err(|_| DirectoryRuleError::GitConfigMalformed)?;
        let managed_block = render_managed_block(&store.rules, self.identity_directory()?)?;
        let updated_global = replace_managed_block(&global_text, &managed_block)?;
        let identity_text = render_identity(&profile);

        self.backup_existing(&global_path, "global-gitconfig")?;
        self.backup_existing(&identity_path, "identity")?;

        if safe_write(&identity_path, identity_text.as_bytes()).is_err()
            || safe_write(&global_path, updated_global.as_bytes()).is_err()
        {
            return Err(self.restore_after_failure(
                &global_path,
                global_before.as_deref(),
                &identity_path,
                identity_before.as_deref(),
                store_path,
                store_before.as_deref(),
                DirectoryRuleError::WriteFailed,
            ));
        }

        if self.save(&store).is_err() {
            return Err(self.restore_after_failure(
                &global_path,
                global_before.as_deref(),
                &identity_path,
                identity_before.as_deref(),
                store_path,
                store_before.as_deref(),
                DirectoryRuleError::StorageWriteFailed,
            ));
        }

        if self.verify_rule(&rule, &profile).is_err() {
            return Err(self.restore_after_failure(
                &global_path,
                global_before.as_deref(),
                &identity_path,
                identity_before.as_deref(),
                store_path,
                store_before.as_deref(),
                DirectoryRuleError::VerificationFailed,
            ));
        }

        Ok(self.resolve(rule, profiles))
    }

    pub(crate) fn preview_remove(
        &self,
        id: &str,
        profiles: &ProfilesService,
    ) -> Result<DirectoryRulePreview, DirectoryRuleError> {
        validate_id(id)?;
        let rule = self
            .load()?
            .rules
            .into_iter()
            .find(|rule| rule.id == id)
            .ok_or(DirectoryRuleError::NotFound)?;
        let profile = match profiles.find(&rule.profile_id) {
            Ok(profile) => Some(profile),
            Err(ProfileError::NotFound) => None,
            Err(error) => return Err(DirectoryRuleError::Profile(error)),
        };
        let global = self.global_config_path()?;
        let identity = self.identity_file_path(&rule.profile_id)?;
        Ok(DirectoryRulePreview {
            rule_id: Some(rule.id),
            directory: rule.directory.clone(),
            profile,
            operation: DirectoryRuleOperation::Remove,
            condition: gitdir_condition(&rule.directory),
            identity_file_path: path_text(&identity)?,
            global_config_path: path_text(&global)?,
            backup_required: global.exists() || identity.exists(),
            conflicts: Vec::new(),
            can_apply: true,
        })
    }

    pub(crate) fn remove(&self, id: &str) -> Result<(), DirectoryRuleError> {
        validate_id(id)?;
        let mut store = self.load()?;
        let store_path = self.store_path()?;
        let store_before = read_optional(store_path)?;
        let position = store
            .rules
            .iter()
            .position(|rule| rule.id == id)
            .ok_or(DirectoryRuleError::NotFound)?;
        let removed = store.rules.remove(position);
        let global_path = self.global_config_path()?;
        let identity_path = self.identity_file_path(&removed.profile_id)?;
        let global_before = read_optional(&global_path)?;
        let identity_before = read_optional(&identity_path)?;
        let global_text = String::from_utf8(global_before.clone().unwrap_or_default())
            .map_err(|_| DirectoryRuleError::GitConfigMalformed)?;
        let managed_block = render_managed_block(&store.rules, self.identity_directory()?)?;
        let updated_global = replace_managed_block(&global_text, &managed_block)?;
        let identity_still_used = store
            .rules
            .iter()
            .any(|rule| rule.profile_id == removed.profile_id);

        self.backup_existing(&global_path, "global-gitconfig")?;
        if !identity_still_used {
            self.backup_existing(&identity_path, "identity")?;
        }

        if safe_write(&global_path, updated_global.as_bytes()).is_err()
            || (!identity_still_used
                && identity_path.exists()
                && fs::remove_file(&identity_path).is_err())
        {
            return Err(self.restore_after_failure(
                &global_path,
                global_before.as_deref(),
                &identity_path,
                identity_before.as_deref(),
                store_path,
                store_before.as_deref(),
                DirectoryRuleError::WriteFailed,
            ));
        }

        if self.save(&store).is_err() {
            return Err(self.restore_after_failure(
                &global_path,
                global_before.as_deref(),
                &identity_path,
                identity_before.as_deref(),
                store_path,
                store_before.as_deref(),
                DirectoryRuleError::StorageWriteFailed,
            ));
        }

        let remaining_global = match fs::read_to_string(&global_path) {
            Ok(contents) => contents,
            Err(_) => {
                return Err(self.restore_after_failure(
                    &global_path,
                    global_before.as_deref(),
                    &identity_path,
                    identity_before.as_deref(),
                    store_path,
                    store_before.as_deref(),
                    DirectoryRuleError::VerificationFailed,
                ));
            }
        };
        if remaining_global.contains(&gitdir_condition(&removed.directory))
            || (!identity_still_used && identity_path.exists())
        {
            return Err(self.restore_after_failure(
                &global_path,
                global_before.as_deref(),
                &identity_path,
                identity_before.as_deref(),
                store_path,
                store_before.as_deref(),
                DirectoryRuleError::VerificationFailed,
            ));
        }
        Ok(())
    }

    fn resolve(&self, rule: DirectoryRule, profiles: &ProfilesService) -> ResolvedDirectoryRule {
        let (state, message) = if !Path::new(&rule.directory).is_dir() {
            (
                DirectoryRuleState::MissingDirectory,
                Some("The configured directory is missing or unavailable.".to_owned()),
            )
        } else {
            match profiles.find(&rule.profile_id) {
                Err(ProfileError::NotFound) => (
                    DirectoryRuleState::MissingProfile,
                    Some("The configured profile no longer exists.".to_owned()),
                ),
                Err(_) => (
                    DirectoryRuleState::Unavailable,
                    Some("The rule could not be checked safely.".to_owned()),
                ),
                Ok(profile) => match self.verify_rule(&rule, &profile) {
                    Ok(()) => (DirectoryRuleState::Active, None),
                    Err(_) => (
                        DirectoryRuleState::NeedsApply,
                        Some("The generated Git configuration differs from this rule. Preview and reapply it.".to_owned()),
                    ),
                },
            }
        };
        ResolvedDirectoryRule {
            id: rule.id,
            directory: rule.directory,
            profile_id: rule.profile_id,
            state,
            message,
        }
    }

    fn rule_conflicts(
        &self,
        rules: &[DirectoryRule],
        current_id: Option<&str>,
        directory: &str,
        profile_id: &str,
        _global_config: &Path,
    ) -> Result<Vec<DirectoryRuleConflict>, DirectoryRuleError> {
        let mut conflicts = Vec::new();
        for rule in rules
            .iter()
            .filter(|rule| Some(rule.id.as_str()) != current_id)
        {
            if paths_overlap(&rule.directory, directory) {
                conflicts.push(DirectoryRuleConflict {
                    kind: DirectoryRuleConflictKind::OverlappingRule,
                    message: if rule.profile_id == profile_id {
                        format!(
                            "Another rule already covers {} with the same profile.",
                            rule.directory
                        )
                    } else {
                        format!(
                            "Another rule for {} overlaps this directory with a different profile.",
                            rule.directory
                        )
                    },
                    blocking: true,
                });
            }
        }

        let managed: HashSet<_> = rules
            .iter()
            .map(|rule| {
                Ok((
                    gitdir_condition(&rule.directory).to_ascii_lowercase(),
                    normalized_git_path(&path_text(&self.identity_file_path(&rule.profile_id)?)?),
                ))
            })
            .collect::<Result<_, DirectoryRuleError>>()?;
        for entry in self.existing_include_entries()? {
            if managed.contains(&(entry.condition.to_ascii_lowercase(), entry.target.clone())) {
                continue;
            }
            if condition_may_overlap(&entry.condition, directory) {
                conflicts.push(DirectoryRuleConflict {
                    kind: DirectoryRuleConflictKind::ExistingConditionalInclude,
                    message: format!(
                        "An existing conditional include ({}) may also apply to this directory.",
                        entry.condition
                    ),
                    blocking: true,
                });
            }
        }
        Ok(conflicts)
    }

    fn repository_conflicts(
        &self,
        directory: &str,
        repositories: &RepositoriesService,
    ) -> Result<Vec<DirectoryRuleConflict>, DirectoryRuleError> {
        let registrations = repositories
            .list()
            .map_err(DirectoryRuleError::Repository)?;
        let root = Path::new(directory);
        let mut conflicts = Vec::new();
        for repository in registrations
            .iter()
            .filter(|repository| Path::new(&repository.path).starts_with(root))
        {
            if let Ok(inspection) = GitService::new(self.runner.clone()).inspect(&repository.path) {
                let overridden = [&inspection.identity.name, &inspection.identity.email]
                    .iter()
                    .filter_map(|value| value.source.as_ref())
                    .any(|source| {
                        matches!(
                            source.scope,
                            GitConfigScope::Local | GitConfigScope::Worktree
                        )
                    });
                if overridden {
                    conflicts.push(DirectoryRuleConflict {
                        kind: DirectoryRuleConflictKind::RepositoryOverride,
                        message: format!(
                            "Registered repository {} has local or worktree identity values that would override this rule.",
                            repository.path
                        ),
                        blocking: true,
                    });
                }
            }
        }
        Ok(conflicts)
    }

    fn existing_include_entries(&self) -> Result<Vec<IncludeEntry>, DirectoryRuleError> {
        let mut entries = Vec::new();
        for path in self
            .global_config_candidates
            .iter()
            .filter(|path| path.exists())
        {
            let text = path_text(path)?;
            let output = self
                .runner
                .run(
                    "git",
                    &[
                        "config",
                        "--file",
                        &text,
                        "--null",
                        "--get-regexp",
                        "^includeIf\\..*\\.path$",
                    ],
                )
                .map_err(|error| match error.kind {
                    ProcessErrorKind::NotFound => DirectoryRuleError::GitMissing,
                    ProcessErrorKind::PermissionDenied | ProcessErrorKind::Other => {
                        DirectoryRuleError::GitCommandFailed
                    }
                })?;
            if !output.success {
                if output.exit_code == Some(1) && output.stdout.is_empty() {
                    continue;
                }
                return Err(DirectoryRuleError::GitConfigMalformed);
            }
            entries.extend(parse_include_entries(&output.stdout)?);
        }
        Ok(entries)
    }

    fn verify_rule(
        &self,
        rule: &DirectoryRule,
        profile: &GitProfile,
    ) -> Result<(), DirectoryRuleError> {
        let identity_path = self.identity_file_path(&profile.id)?;
        let identity = fs::read_to_string(&identity_path)
            .map_err(|_| DirectoryRuleError::VerificationFailed)?;
        if identity != render_identity(profile) {
            return Err(DirectoryRuleError::VerificationFailed);
        }
        let expected = IncludeEntry {
            condition: gitdir_condition(&rule.directory),
            target: normalized_git_path(&path_text(&identity_path)?),
        };
        if self.existing_include_entries()?.contains(&expected) {
            Ok(())
        } else {
            Err(DirectoryRuleError::VerificationFailed)
        }
    }

    fn restore_after_failure(
        &self,
        global_path: &Path,
        global_before: Option<&[u8]>,
        identity_path: &Path,
        identity_before: Option<&[u8]>,
        store_path: &Path,
        store_before: Option<&[u8]>,
        original: DirectoryRuleError,
    ) -> DirectoryRuleError {
        if restore_optional(global_path, global_before).is_ok()
            && restore_optional(identity_path, identity_before).is_ok()
            && restore_optional(store_path, store_before).is_ok()
        {
            original
        } else {
            DirectoryRuleError::RollbackFailed
        }
    }

    fn store_path(&self) -> Result<&Path, DirectoryRuleError> {
        self.store_path
            .as_deref()
            .ok_or(DirectoryRuleError::StorageUnavailable)
    }

    fn global_config_path(&self) -> Result<PathBuf, DirectoryRuleError> {
        let mut existing_paths = Vec::new();
        let mut managed_path = None;
        for existing in self
            .global_config_candidates
            .iter()
            .filter(|path| path.exists())
        {
            let canonical =
                dunce::canonicalize(existing).map_err(|_| DirectoryRuleError::StorageReadFailed)?;
            if !canonical.is_file() {
                return Err(DirectoryRuleError::GitConfigMalformed);
            }
            if existing_paths.contains(&canonical) {
                continue;
            }
            let contents = fs::read_to_string(&canonical)
                .map_err(|_| DirectoryRuleError::GitConfigMalformed)?;
            if contents.contains(MANAGED_BEGIN) || contents.contains(MANAGED_END) {
                if managed_path.is_some() {
                    return Err(DirectoryRuleError::GitConfigMalformed);
                }
                managed_path = Some(canonical.clone());
            }
            existing_paths.push(canonical);
        }
        if let Some(managed) = managed_path {
            return Ok(managed);
        }
        if let Some(existing) = existing_paths.into_iter().next() {
            return Ok(existing);
        }
        self.global_config_candidates
            .first()
            .cloned()
            .ok_or(DirectoryRuleError::StorageUnavailable)
    }

    fn identity_directory(&self) -> Result<&Path, DirectoryRuleError> {
        self.identity_directory
            .as_deref()
            .ok_or(DirectoryRuleError::StorageUnavailable)
    }

    fn identity_file_path(&self, profile_id: &str) -> Result<PathBuf, DirectoryRuleError> {
        validate_id(profile_id)?;
        Ok(self
            .identity_directory()?
            .join(format!("{profile_id}.gitconfig")))
    }

    fn backup_existing(&self, path: &Path, label: &str) -> Result<(), DirectoryRuleError> {
        if !path.exists() {
            return Ok(());
        }
        let backup_directory = self
            .backup_directory
            .as_deref()
            .ok_or(DirectoryRuleError::StorageUnavailable)?;
        fs::create_dir_all(backup_directory).map_err(|_| DirectoryRuleError::WriteFailed)?;
        let backup = backup_directory.join(format!("{label}-{}.bak", Uuid::new_v4()));
        fs::copy(path, backup).map_err(|_| DirectoryRuleError::WriteFailed)?;
        Ok(())
    }

    fn load(&self) -> Result<DirectoryRulesStore, DirectoryRuleError> {
        let path = self.store_path()?;
        recover_backup_if_needed(path)?;
        if !path.exists() {
            return Ok(DirectoryRulesStore {
                version: STORE_VERSION,
                rules: Vec::new(),
            });
        }
        let bytes = fs::read(path).map_err(|_| DirectoryRuleError::StorageReadFailed)?;
        let store: DirectoryRulesStore =
            serde_json::from_slice(&bytes).map_err(|_| DirectoryRuleError::MalformedData)?;
        validate_store(&store)?;
        Ok(store)
    }

    fn save(&self, store: &DirectoryRulesStore) -> Result<(), DirectoryRuleError> {
        let path = self.store_path()?;
        let bytes =
            serde_json::to_vec_pretty(store).map_err(|_| DirectoryRuleError::StorageWriteFailed)?;
        safe_store_write(path, &bytes)?;
        let loaded = self.load()?;
        if loaded.rules == store.rules {
            Ok(())
        } else {
            Err(DirectoryRuleError::StorageWriteFailed)
        }
    }
}

fn canonical_directory(path: &str) -> Result<String, DirectoryRuleError> {
    validate_path(path)?;
    let canonical = dunce::canonicalize(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => DirectoryRuleError::PathNotFound,
        _ => DirectoryRuleError::InvalidPath,
    })?;
    if !canonical.is_dir() {
        return Err(DirectoryRuleError::NotDirectory);
    }
    path_text(&canonical)
}

fn validate_store(store: &DirectoryRulesStore) -> Result<(), DirectoryRuleError> {
    if store.version != STORE_VERSION {
        return Err(DirectoryRuleError::MalformedData);
    }
    let mut ids = HashSet::new();
    let mut directories = HashSet::new();
    for rule in &store.rules {
        validate_id(&rule.id).map_err(|_| DirectoryRuleError::MalformedData)?;
        validate_id(&rule.profile_id).map_err(|_| DirectoryRuleError::MalformedData)?;
        validate_path(&rule.directory).map_err(|_| DirectoryRuleError::MalformedData)?;
        if !Path::new(&rule.directory).is_absolute()
            || !ids.insert(&rule.id)
            || !directories.insert(&rule.directory)
        {
            return Err(DirectoryRuleError::MalformedData);
        }
    }
    Ok(())
}

fn validate_id(id: &str) -> Result<(), DirectoryRuleError> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| DirectoryRuleError::InvalidId)
}

fn validate_path(path: &str) -> Result<(), DirectoryRuleError> {
    if path.is_empty()
        || path.chars().count() > MAX_PATH_CHARS
        || path
            .chars()
            .any(|character| character == '\0' || character.is_control())
    {
        Err(DirectoryRuleError::InvalidPath)
    } else {
        Ok(())
    }
}

fn path_text(path: &Path) -> Result<String, DirectoryRuleError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or(DirectoryRuleError::InvalidPath)
}

fn normalized_git_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn gitdir_condition(directory: &str) -> String {
    format!(
        "gitdir:{}/",
        normalized_git_path(directory).trim_end_matches('/')
    )
}

fn paths_overlap(left: &str, right: &str) -> bool {
    Path::new(left).starts_with(right) || Path::new(right).starts_with(left)
}

fn condition_may_overlap(condition: &str, directory: &str) -> bool {
    let lower = condition.to_ascii_lowercase();
    let Some(value) = lower
        .strip_prefix("gitdir:")
        .or_else(|| lower.strip_prefix("gitdir/i:"))
    else {
        return false;
    };
    if value.contains(['*', '?', '[']) || value.starts_with('~') || !Path::new(value).is_absolute()
    {
        return true;
    }
    let condition_path = value.trim_end_matches('/');
    paths_overlap(condition_path, &normalized_git_path(directory))
}

fn render_identity(profile: &GitProfile) -> String {
    format!(
        "# Generated by Git Identity Manager. Do not edit.\n[user]\n\tname = {}\n\temail = {}\n",
        quote_config_value(&profile.git_name),
        quote_config_value(&profile.git_email)
    )
}

fn render_managed_block(
    rules: &[DirectoryRule],
    identity_directory: &Path,
) -> Result<String, DirectoryRuleError> {
    if rules.is_empty() {
        return Ok(String::new());
    }
    let mut sorted = rules.to_vec();
    sorted.sort_by(|left, right| left.directory.cmp(&right.directory));
    let mut output = format!("{MANAGED_BEGIN}\n");
    for rule in sorted {
        let condition = gitdir_condition(&rule.directory);
        let identity = identity_directory.join(format!("{}.gitconfig", rule.profile_id));
        output.push_str(&format!(
            "[includeIf {}]\n\tpath = {}\n",
            quote_config_value(&condition),
            quote_config_value(&normalized_git_path(&path_text(&identity)?))
        ));
    }
    output.push_str(MANAGED_END);
    Ok(output)
}

fn quote_config_value(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

fn replace_managed_block(existing: &str, replacement: &str) -> Result<String, DirectoryRuleError> {
    let begins: Vec<_> = existing.match_indices(MANAGED_BEGIN).collect();
    let ends: Vec<_> = existing.match_indices(MANAGED_END).collect();
    if begins.len() > 1 || ends.len() > 1 || begins.len() != ends.len() {
        return Err(DirectoryRuleError::GitConfigMalformed);
    }
    if let (Some((begin, _)), Some((end, _))) = (begins.first(), ends.first()) {
        if end < begin {
            return Err(DirectoryRuleError::GitConfigMalformed);
        }
        let end = end + MANAGED_END.len();
        let mut updated = String::with_capacity(existing.len() + replacement.len());
        updated.push_str(&existing[..*begin]);
        updated.push_str(replacement);
        updated.push_str(&existing[end..]);
        return Ok(updated);
    }
    if replacement.is_empty() {
        return Ok(existing.to_owned());
    }
    let mut updated = existing.to_owned();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    if !updated.is_empty() {
        updated.push('\n');
    }
    updated.push_str(replacement);
    updated.push('\n');
    Ok(updated)
}

fn parse_include_entries(output: &str) -> Result<Vec<IncludeEntry>, DirectoryRuleError> {
    let mut entries = Vec::new();
    for field in output.split('\0').filter(|field| !field.is_empty()) {
        let (key, target) = field
            .split_once('\n')
            .ok_or(DirectoryRuleError::GitConfigMalformed)?;
        let lower = key.to_ascii_lowercase();
        if !lower.starts_with("includeif.") || !lower.ends_with(".path") || target.is_empty() {
            return Err(DirectoryRuleError::GitConfigMalformed);
        }
        let condition = key["includeif.".len()..key.len() - ".path".len()].to_owned();
        entries.push(IncludeEntry {
            condition,
            target: target.to_owned(),
        });
    }
    Ok(entries)
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, DirectoryRuleError> {
    if path.exists() {
        fs::read(path)
            .map(Some)
            .map_err(|_| DirectoryRuleError::StorageReadFailed)
    } else {
        Ok(None)
    }
}

fn safe_write(path: &Path, contents: &[u8]) -> Result<(), ()> {
    let parent = path.parent().ok_or(())?;
    fs::create_dir_all(parent).map_err(|_| ())?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config"),
        Uuid::new_v4()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| ())?;
    if file
        .write_all(contents)
        .and_then(|_| file.sync_all())
        .is_err()
    {
        let _ = fs::remove_file(&temporary);
        return Err(());
    }
    if path.exists() && fs::remove_file(path).is_err() {
        let _ = fs::remove_file(&temporary);
        return Err(());
    }
    fs::rename(&temporary, path).map_err(|_| ())
}

fn restore_optional(path: &Path, contents: Option<&[u8]>) -> Result<(), ()> {
    match contents {
        Some(contents) => safe_write(path, contents),
        None if path.exists() => fs::remove_file(path).map_err(|_| ()),
        None => Ok(()),
    }
}

fn store_backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.backup")
}

fn safe_store_write(path: &Path, contents: &[u8]) -> Result<(), DirectoryRuleError> {
    let parent = path
        .parent()
        .ok_or(DirectoryRuleError::StorageUnavailable)?;
    fs::create_dir_all(parent).map_err(|_| DirectoryRuleError::StorageWriteFailed)?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("directory-rules"),
        Uuid::new_v4()
    ));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| DirectoryRuleError::StorageWriteFailed)?;
    if file
        .write_all(contents)
        .and_then(|_| file.sync_all())
        .is_err()
    {
        let _ = fs::remove_file(&temporary);
        return Err(DirectoryRuleError::StorageWriteFailed);
    }

    let backup = store_backup_path(path);
    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| DirectoryRuleError::StorageWriteFailed)?;
    }
    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup).map_err(|_| DirectoryRuleError::StorageWriteFailed)?;
    }
    if fs::rename(&temporary, path).is_err() {
        if had_existing {
            let _ = fs::rename(&backup, path);
        }
        return Err(DirectoryRuleError::StorageWriteFailed);
    }
    if had_existing {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

fn recover_backup_if_needed(path: &Path) -> Result<(), DirectoryRuleError> {
    let backup = store_backup_path(path);
    if !path.exists() && backup.exists() {
        fs::rename(backup, path).map_err(|_| DirectoryRuleError::StorageReadFailed)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    use crate::{
        process::{ProcessError, ProcessOutput, SystemProcessRunner},
        services::profiles::{GithubAccountReference, ProfileInput},
    };

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "git-identity-manager-rules-test-{}",
                Uuid::new_v4()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    struct IsolatedGitRunner;

    impl ProcessRunner for &IsolatedGitRunner {
        fn run(&self, program: &str, args: &[&str]) -> Result<ProcessOutput, ProcessError> {
            let output = std::process::Command::new(program)
                .args(args)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .map_err(ProcessError::from)?;
            Ok(ProcessOutput {
                success: output.status.success(),
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            })
        }

        fn start(&self, _program: &str, _args: &[&str]) -> Result<(), ProcessError> {
            panic!("directory rules never start detached processes")
        }
    }

    fn profile(profiles: &ProfilesService) -> GitProfile {
        profiles
            .create(ProfileInput {
                label: "Work".to_owned(),
                git_name: "Work Octo".to_owned(),
                git_email: "work@example.com".to_owned(),
                github_account: Some(GithubAccountReference {
                    hostname: "github.com".to_owned(),
                    username: "work-octo".to_owned(),
                }),
            })
            .unwrap()
    }

    fn services<'a>(
        directory: &'a TestDirectory,
        runner: &'a IsolatedGitRunner,
    ) -> (
        DirectoryRulesService<&'a IsolatedGitRunner>,
        ProfilesService,
        RepositoriesService,
    ) {
        let config = directory.0.join("config");
        (
            DirectoryRulesService::new(
                Some(directory.0.join("directory-rules.v1.json")),
                Some(config.join("identities")),
                Some(config.join("backups")),
                vec![directory.0.join("global.gitconfig")],
                runner,
            ),
            ProfilesService::new(Some(directory.0.join("profiles.v1.json"))),
            RepositoriesService::new(Some(directory.0.join("repositories.v1.json"))),
        )
    }

    #[test]
    fn managed_block_preserves_unrelated_configuration() {
        let existing = "[core]\n\teditor = vim\n";
        let rule = DirectoryRule {
            id: Uuid::new_v4().to_string(),
            directory: "/work".to_owned(),
            profile_id: Uuid::new_v4().to_string(),
        };
        let block = render_managed_block(&[rule], Path::new("/app/identities")).unwrap();
        let updated = replace_managed_block(existing, &block).unwrap();
        assert!(updated.starts_with(existing));
        assert!(updated.contains("[core]\n\teditor = vim"));
        assert!(updated.contains("includeIf \"gitdir:/work/\""));
        let removed = replace_managed_block(&updated, "").unwrap();
        assert!(removed.contains("[core]\n\teditor = vim"));
        assert!(!removed.contains("includeIf"));
    }

    #[test]
    fn apply_creates_backup_persists_and_resolves_identity_through_git() {
        let directory = TestDirectory::new();
        let runner = IsolatedGitRunner;
        let (rules, profiles, repositories) = services(&directory, &runner);
        let profile = profile(&profiles);
        let root = directory.0.join("work");
        let repository = root.join("project");
        fs::create_dir_all(&repository).unwrap();
        let status = std::process::Command::new("git")
            .args(["-C", repository.to_str().unwrap(), "init", "--quiet"])
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .status()
            .unwrap();
        assert!(status.success());
        let global = directory.0.join("global.gitconfig");
        fs::write(&global, "[core]\n\teditor = vim\n").unwrap();

        let input = DirectoryRuleInput {
            id: None,
            directory: root.to_string_lossy().into_owned(),
            profile_id: profile.id.clone(),
        };
        let preview = rules
            .preview(input.clone(), &profiles, &repositories)
            .unwrap();
        assert!(preview.can_apply);
        assert!(preview.backup_required);
        let applied = rules.apply(input, &profiles, &repositories).unwrap();
        assert_eq!(applied.state, DirectoryRuleState::Active);
        assert!(fs::read_to_string(&global)
            .unwrap()
            .contains("editor = vim"));
        assert_eq!(
            fs::read_dir(directory.0.join("config/backups"))
                .unwrap()
                .count(),
            1
        );

        let output = std::process::Command::new("git")
            .args([
                "-C",
                repository.to_str().unwrap(),
                "config",
                "--get",
                "user.email",
            ])
            .env("HOME", &directory.0)
            .env("GIT_CONFIG_GLOBAL", &global)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "work@example.com"
        );
        assert_eq!(
            rules.list(&profiles).unwrap()[0].state,
            DirectoryRuleState::Active
        );
    }

    #[test]
    fn preview_blocks_overlapping_app_and_external_rules() {
        let directory = TestDirectory::new();
        let runner = IsolatedGitRunner;
        let (rules, profiles, repositories) = services(&directory, &runner);
        let profile = profile(&profiles);
        let root = directory.0.join("work");
        let nested = root.join("client");
        fs::create_dir_all(&nested).unwrap();
        let first = rules
            .apply(
                DirectoryRuleInput {
                    id: None,
                    directory: root.to_string_lossy().into_owned(),
                    profile_id: profile.id.clone(),
                },
                &profiles,
                &repositories,
            )
            .unwrap();
        let preview = rules
            .preview(
                DirectoryRuleInput {
                    id: None,
                    directory: nested.to_string_lossy().into_owned(),
                    profile_id: profile.id.clone(),
                },
                &profiles,
                &repositories,
            )
            .unwrap();
        assert!(!preview.can_apply);
        assert!(preview
            .conflicts
            .iter()
            .any(|conflict| conflict.kind == DirectoryRuleConflictKind::OverlappingRule));

        rules.remove(&first.id).unwrap();
        let canonical_root = dunce::canonicalize(&root).unwrap();
        fs::write(
            directory.0.join("global.gitconfig"),
            format!(
                "[includeIf \"gitdir:{}/\"]\n\tpath = /external.inc\n",
                canonical_root.to_string_lossy()
            ),
        )
        .unwrap();
        let external = rules
            .preview(
                DirectoryRuleInput {
                    id: None,
                    directory: root.to_string_lossy().into_owned(),
                    profile_id: profile.id,
                },
                &profiles,
                &repositories,
            )
            .unwrap();
        assert!(external.conflicts.iter().any(|conflict| {
            conflict.kind == DirectoryRuleConflictKind::ExistingConditionalInclude
        }));
    }

    #[test]
    fn registered_local_identity_is_a_blocking_conflict() {
        let directory = TestDirectory::new();
        let runner = IsolatedGitRunner;
        let (rules, profiles, repositories) = services(&directory, &runner);
        let profile = profile(&profiles);
        let root = directory.0.join("work");
        let repository = root.join("project");
        fs::create_dir_all(&repository).unwrap();
        for args in [
            vec!["init", "--quiet"],
            vec!["config", "--local", "user.name", "Local Name"],
        ] {
            let status = std::process::Command::new("git")
                .arg("-C")
                .arg(&repository)
                .args(args)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .status()
                .unwrap();
            assert!(status.success());
        }
        repositories
            .register(dunce::canonicalize(&repository).unwrap().to_str().unwrap())
            .unwrap();
        let preview = rules
            .preview(
                DirectoryRuleInput {
                    id: None,
                    directory: root.to_string_lossy().into_owned(),
                    profile_id: profile.id,
                },
                &profiles,
                &repositories,
            )
            .unwrap();
        assert!(!preview.can_apply);
        assert!(preview
            .conflicts
            .iter()
            .any(|conflict| { conflict.kind == DirectoryRuleConflictKind::RepositoryOverride }));
    }

    #[test]
    fn removal_keeps_shared_identity_until_last_rule_and_preserves_unrelated_config() {
        let directory = TestDirectory::new();
        let runner = IsolatedGitRunner;
        let (rules, profiles, repositories) = services(&directory, &runner);
        let profile = profile(&profiles);
        let first_root = directory.0.join("personal");
        let second_root = directory.0.join("work");
        fs::create_dir(&first_root).unwrap();
        fs::create_dir(&second_root).unwrap();
        fs::write(
            directory.0.join("global.gitconfig"),
            "[core]\n\teditor = vim\n",
        )
        .unwrap();
        let first = rules
            .apply(
                DirectoryRuleInput {
                    id: None,
                    directory: first_root.to_string_lossy().into_owned(),
                    profile_id: profile.id.clone(),
                },
                &profiles,
                &repositories,
            )
            .unwrap();
        let second = rules
            .apply(
                DirectoryRuleInput {
                    id: None,
                    directory: second_root.to_string_lossy().into_owned(),
                    profile_id: profile.id.clone(),
                },
                &profiles,
                &repositories,
            )
            .unwrap();
        let identity = directory
            .0
            .join(format!("config/identities/{}.gitconfig", profile.id));
        rules.remove(&first.id).unwrap();
        assert!(identity.exists());
        rules.remove(&second.id).unwrap();
        assert!(!identity.exists());
        let global = fs::read_to_string(directory.0.join("global.gitconfig")).unwrap();
        assert!(global.contains("editor = vim"));
        assert!(!global.contains(MANAGED_BEGIN));
    }

    #[test]
    fn rejects_invalid_paths_before_running_git() {
        let service = DirectoryRulesService::new(None, None, None, Vec::new(), SystemProcessRunner);
        assert_eq!(
            canonical_directory("bad\npath"),
            Err(DirectoryRuleError::InvalidPath)
        );
        assert!(matches!(
            service.load(),
            Err(DirectoryRuleError::StorageUnavailable)
        ));
    }
}
