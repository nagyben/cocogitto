use self::WriterMode::*;
use crate::commit::{Commit, CommitType};
use crate::COMMITS_METADATA;
use anyhow::Result;
use git2::Oid;
use std::fs;
use std::path::PathBuf;

pub enum WriterMode {
    Replace,
    Prepend,
    Append,
}

pub(crate) struct Changelog {
    pub from: Oid,
    pub to: Oid,
    pub date: String,
    pub commits: Vec<Commit>,
}

pub(crate) struct ChangelogWriter {
    pub(crate) changelog: Changelog,
    pub(crate) path: PathBuf,
    pub(crate) mode: WriterMode,
}

impl ChangelogWriter {
    pub(crate) fn write(&mut self) -> Result<()> {
        match &self.mode {
            Append => self.insert(),
            Prepend => self.insert(),
            Replace => self.replace(),
        }
    }

    fn insert(&mut self) -> Result<()> {
        let mut changelog_content = fs::read_to_string(&self.path)?;

        let separator_idx = match self.mode {
            Append => changelog_content.rfind("- - -"),
            Prepend => changelog_content.find("- - -"),
            _ => unreachable!(),
        };

        if let Some(idx) = separator_idx {
            let markdown_changelog = self.changelog.markdown(false);
            changelog_content.insert_str(idx + 5, &markdown_changelog);
            changelog_content.insert_str(idx + 5 + markdown_changelog.len(), "\n- - -");
            fs::write(&self.path, changelog_content)?;

            Ok(())
        } else {
            Err(anyhow!(
                "Cannot find default separator '- - -' in {}",
                self.path.display()
            ))
        }
    }

    fn replace(&mut self) -> Result<()> {
        let mut content = Changelog::header();
        content.push_str(&self.changelog.markdown(false));
        content.push_str(Changelog::footer());

        fs::write(&self.path, content).map_err(|err| anyhow!(err))
    }
}

impl Changelog {
    pub(crate) fn markdown(&mut self, colored: bool) -> String {
        let mut out = String::new();

        let short_to = &self.to.to_string()[0..6];
        let short_from = &self.from.to_string()[0..6];
        out.push_str(&format!(
            "\n## {}..{} - {}\n\n",
            short_from, short_to, self.date
        ));

        let add_commit_section = |commit_type: &CommitType| {
            let commits: Vec<Commit> = self
                .commits
                .drain_filter(|commit| &commit.message.commit_type == commit_type)
                .collect();

            let metadata = COMMITS_METADATA.get(&commit_type).unwrap();
            if !commits.is_empty() {
                out.push_str(&format!("\n### {}\n\n", metadata.changelog_title));
                commits.iter().for_each(|commit| {
                    out.push_str(&commit.to_markdown(colored));
                    out.push('\n');
                });
            }
        };

        COMMITS_METADATA
            .iter()
            .map(|(commit_type, _)| commit_type)
            .for_each(add_commit_section);

        out
    }

    fn header() -> String {
        let title = "# Changelog";
        let link = "[conventional commits]";
        format!(
            "{}\nAll notable changes to this project will be documented in this file. \
        See {}(https://www.conventionalcommits.org/) for commit guidelines.\n\n- - -\n",
            title, link
        )
    }

    fn footer() -> &'static str {
        "- - -\n\nThis changelog was generated by [cocogitto](https://github.com/oknozor/cocogitto)."
    }
}
