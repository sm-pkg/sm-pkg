use crate::BoxResult;
use auth_git2::GitAuthenticator;
use git2::{AnnotatedCommit, FetchOptions, Repository, build::CheckoutBuilder};
use std::path::Path;

pub const DEFAULT_BRANCH: &str = "master";
pub const DEFAULT_REMOTE: &str = "origin";
pub const DEFAULT_REPO_URL: &str = "https://github.com/sm-pkg/plugins.git";

pub struct Git {
    repo: Repository,
    branch: String,
}

impl Git {
    pub fn open_or_create(path: &Path, url: &str, branch: &str) -> Result<Self, git2::Error> {
        if path.exists() {
            debug!("Using existing repo");
            return match Repository::open(path) {
                Ok(repo) => Ok(Self {
                    repo,
                    branch: branch.to_string(),
                }),
                Err(err) => Err(err),
            };
        };

        // let repo = match Repository::init(path) {
        //     Ok(r) => r,
        //     Err(err) => return Err(err),
        // };
        // repo.remote(DEFAULT_REMOTE, url)?;

        let auth = GitAuthenticator::default();
        let git_config = git2::Config::open_default()?;
        let mut repo_builder = git2::build::RepoBuilder::new();
        let mut fetch_options = git2::FetchOptions::new();
        let mut remote_callbacks = git2::RemoteCallbacks::new();
        remote_callbacks.credentials(auth.credentials(&git_config));
        fetch_options.remote_callbacks(remote_callbacks);
        repo_builder.fetch_options(fetch_options);

        let repo = repo_builder.clone(url, path)?;

        Ok(Self {
            repo,
            branch: branch.to_string(),
        })
    }

    pub fn pull(&self) -> BoxResult {
        self.reset()?;
        let mut remote = self.repo.find_remote(DEFAULT_REMOTE)?;
        let mut fetch_options = FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);

        remote.fetch(
            &["+refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )?;

        drop(remote);
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let annotated_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        let commit_id = annotated_commit.id();
        debug!("Successfully fetched latest changes, merging...");
        self.git_merge(&self.branch, annotated_commit)?;

        info!("Successfully merged {} into {}", commit_id, self.branch);

        Ok(())
    }

    fn git_merge(&self, remote_branch: &str, fetch_commit: AnnotatedCommit<'_>) -> BoxResult<()> {
        // First perform a merge analysis to understand how to proceed
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;

        // Handle fast-forward merges
        if analysis.0.is_fast_forward() {
            debug!(
                "Performing fast forward merge from branch '{}'",
                remote_branch
            );
            let refname = format!("refs/heads/{remote_branch}");
            // This code will return early with an error if pulling into an empty repository.
            // That *should* never happen, so that handling was omitted, but if it's needed,
            // an example can be found at:
            // https://github.com/rust-lang/git2-rs/blob/master/examples/pull.rs#L160
            let mut reference = self.repo.find_reference(&refname)?;
            self.fast_forward(&mut reference, &fetch_commit)?;
        }
        // Handle normal merges
        else if analysis.0.is_normal() {
            debug!("Performing normal merge from branch '{}'", remote_branch);
            let head_commit = self
                .repo
                .reference_to_annotated_commit(&self.repo.head()?)?;
            self.normal_merge(&fetch_commit, &head_commit)?;
        }
        // If no merging is needed
        else {
            debug!("No work needed to merge from branch '{}'", remote_branch);
        }

        Ok(())
    }

    /// This is a helper function called by [`Self::git_merge`], you probably don't want to call this
    /// directly.
    ///
    /// Merge the the `source` reference commit into on top of the reference `destination` commit.
    /// This is considered a "normal merge", as opposed to a fast forward merge. See [`Self::fast_forward`]
    /// for more info.
    fn normal_merge(&self, source: &AnnotatedCommit, destination: &AnnotatedCommit) -> BoxResult {
        let source_tree = self.repo.find_commit(source.id())?.tree()?;
        let destination_tree = self.repo.find_commit(destination.id())?.tree()?;
        // The ancestor is the most recent commit that the source and destination share.
        let ancestor = self
            .repo
            .find_commit(self.repo.merge_base(source.id(), destination.id())?)?
            .tree()?;
        // A git index (or staging area) is where changes are written before they're committed.
        let mut idx = self
            .repo
            .merge_trees(&ancestor, &source_tree, &destination_tree, None)?;
        if idx.has_conflicts() {
            // bail!(
            //     "Unable to merge changes from {:?} into {:?} because there are merge conflicts and method is currently implemented to handle merge conflicts.",
            //     source.refname().unwrap(),
            //     destination.refname().unwrap()
            // );

            return Err("Merge conflicts detected".into());
        }
        // Write the changes to disk, then create and attach a merge commit to that tree then update the working tree to the latest commit.
        let result_tree = self.repo.find_tree(idx.write_tree()?)?;
        let _merge_commit = {
            let msg = format!("Merge: {} into {}", source.id(), destination.id());
            let sig = self.repo.signature()?;
            let destination_commit_parent = self.repo.find_commit(destination.id())?;
            let source_commit_parent = self.repo.find_commit(source.id())?;
            self.repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &msg,
                &result_tree,
                &[&destination_commit_parent, &source_commit_parent],
            )?
        };
        // Now update the working tree
        self.repo.checkout_head(None)?;

        debug!(
            "normal merged latest changes {} -> {}",
            source.id(),
            destination.id()
        );

        Ok(())
    }

    /// This is a helper function used by [`Self::git_merge`], you probably don't want to call it
    /// directly.
    ///
    /// In some cases, a merge can be simplified by just moving the `HEAD` pointer forwards if the new
    /// commits are direct ancestors of the old `HEAD`.
    fn fast_forward(
        &self,
        local_branch: &mut git2::Reference,
        remote_commit: &AnnotatedCommit,
    ) -> BoxResult {
        let lb_name = local_branch
            .name()
            .expect("invalid local branch name")
            .to_string();
        let msg = format!(
            "Fast forwarding: Setting {lb_name} to id: {}",
            remote_commit.id()
        );
        local_branch.set_target(remote_commit.id(), &msg)?;
        self.repo.set_head(&lb_name)?;
        self.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

        debug!(
            "fast forwarding latest changes {} -> {}",
            remote_commit.id(),
            local_branch.name().unwrap_or("unknown")
        );

        Ok(())
    }

    fn reset(&self) -> BoxResult {
        // Get the current status of the repository
        let status = self.repo.statuses(None)?;

        // Log the status of each file
        for entry in status.iter() {
            debug!("File: {:?}, Status: {:?}", entry.path(), entry.status());
        }

        // Check for uncommitted changes
        if status.iter().any(|s| s.status() != git2::Status::CURRENT) {
            warn!("Uncommitted changes found. Discarding changes before pulling.");

            // Create a checkout builder to discard changes
            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder.force();

            // Checkout HEAD to discard uncommitted changes
            self.repo
                .checkout_head(Some(&mut checkout_builder))
                .expect("Failed to checkout HEAD and discard uncommitted changes");
            info!("Discarded uncommitted changes and reset to the last commit.");
        }

        Ok(())
    }
}
