use std::error::Error;

use serde::Deserialize;
use time::{macros::format_description, PrimitiveDateTime};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Notification {
	pub id: String,
	pub repository: Repository,
	pub subject: Subject,
	pub reason: String,
	pub unread: bool,
	pub updated_at: String,
	pub last_read_at: Option<String>,
	pub url: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Repository {
	pub id: u64,

	pub node_id: Option<String>,
	pub name: String,

	pub full_name: Option<String>,

	pub owner: Option<serde_json::Value>,

	pub private: Option<bool>,

	pub html_url: Option<String>,

	pub description: Option<String>,

	pub fork: Option<bool>,
	pub url: String,

	pub archive_url: Option<String>,

	pub assignees_url: Option<String>,

	pub blobs_url: Option<String>,

	pub branches_url: Option<String>,

	pub collaborators_url: Option<String>,

	pub comments_url: Option<String>,

	pub commits_url: Option<String>,

	pub compare_url: Option<String>,

	pub contents_url: Option<String>,

	pub contributors_url: Option<String>,

	pub deployments_url: Option<String>,

	pub downloads_url: Option<String>,

	pub events_url: Option<String>,

	pub forks_url: Option<String>,

	pub git_commits_url: Option<String>,

	pub git_refs_url: Option<String>,

	pub git_tags_url: Option<String>,

	pub git_url: Option<String>,

	pub issue_comment_url: Option<String>,

	pub issue_events_url: Option<String>,

	pub issues_url: Option<String>,

	pub keys_url: Option<String>,

	pub labels_url: Option<String>,

	pub languages_url: Option<String>,

	pub merges_url: Option<String>,

	pub milestones_url: Option<String>,

	pub notifications_url: Option<String>,

	pub pulls_url: Option<String>,

	pub releases_url: Option<String>,

	pub ssh_url: Option<String>,

	pub stargazers_url: Option<String>,

	pub statuses_url: Option<String>,

	pub subscribers_url: Option<String>,

	pub subscription_url: Option<String>,

	pub tags_url: Option<String>,

	pub teams_url: Option<String>,

	pub trees_url: Option<String>,

	pub clone_url: Option<String>,

	pub mirror_url: Option<String>,

	pub hooks_url: Option<String>,

	pub svn_url: Option<String>,

	pub homepage: Option<String>,

	pub language: Option<::serde_json::Value>,

	pub forks_count: Option<u32>,

	pub stargazers_count: Option<u32>,

	pub watchers_count: Option<u32>,

	pub size: Option<u32>,

	pub default_branch: Option<String>,

	pub open_issues_count: Option<u32>,

	pub is_template: Option<bool>,

	pub topics: Option<Vec<String>>,

	pub has_issues: Option<bool>,

	pub has_projects: Option<bool>,

	pub has_wiki: Option<bool>,

	pub has_pages: Option<bool>,

	pub has_downloads: Option<bool>,

	pub archived: Option<bool>,

	pub disabled: Option<bool>,

	pub visibility: Option<String>,
	pub pushed_at: Option<String>,
	pub created_at: Option<String>,
	pub updated_at: Option<String>,

	pub permissions: Option<serde_json::Value>,

	pub allow_rebase_merge: Option<bool>,

	pub template_repository: Option<Box<Repository>>,

	pub allow_squash_merge: Option<bool>,

	pub allow_merge_commit: Option<bool>,

	pub allow_update_branch: Option<bool>,

	pub allow_forking: Option<bool>,

	pub subscribers_count: Option<i64>,

	pub network_count: Option<i64>,

	pub license: Option<serde_json::Value>,

	pub allow_auto_merge: Option<bool>,

	pub delete_branch_on_merge: Option<bool>,

	pub parent: Option<Box<Repository>>,

	pub source: Option<Box<Repository>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[non_exhaustive]
pub struct Subject {
	pub title: String,
	pub url: Option<String>,
	pub latest_comment_url: Option<String>,
	pub r#type: String,
}

/// Get new notifications.
/// Returns: notifications and new last-modified value.
pub fn get_new_notifications(
	pat: &str,
	last_modified: Option<&str>,
) -> Result<(Vec<Notification>, Option<String>), Box<dyn Error>> {
	let mut resp = ureq::get("https://api.github.com/notifications").header("Authorization", &format!("Bearer {pat}"));
	if let Some(val) = last_modified {
		resp = resp.header("If-Modified-Since", val);
	}
	let json = resp.call()?.into_body().read_to_string()?;
	let items: Vec<Notification> = serde_json::from_str(&json)?;
	let new_last_modified = items.get(0).map(|x| x.updated_at.clone());
	let last_modified = if let Some(lm) = new_last_modified {
		// parse and increase by five seconds
		let format = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
		let mut dt = PrimitiveDateTime::parse(&lm, format)?;
		dt += time::Duration::seconds(5);
		Some(dt.format(&format)?)
	} else {
		last_modified.map(|x| x.to_owned())
	};
	Ok((items, last_modified))
}
