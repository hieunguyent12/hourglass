use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::{
    self,
    header::{ACCEPT, USER_AGENT},
};
use serde::Deserialize;
use std::env;
use std::process::Command;
use url::Url;

struct GitRepo {
    name: String,
    owner: String,
    url: String,
    repo_type: String,
}
#[derive(Deserialize, Debug)]
pub struct GitUser {
    pub login: String,
    pub id: u32,
    pub node_id: String,
}

#[derive(Deserialize, Debug)]
pub struct RepoIssue {
    pub id: u32,
    pub node_id: String,
    pub html_url: String,
    pub number: u32,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user: GitUser,
}

pub fn get_issues() -> Option<Vec<RepoIssue>> {
    let output = Command::new("git")
        .arg("remote")
        .arg("-v")
        .output()
        .expect("Failed to get git remotes");

    let data = String::from_utf8_lossy(&output.stdout).to_string();

    let remotes = get_lines(&data);

    let fetch_remote = remotes[0];

    let push_remote = remotes[1];

    let re = Regex::new(r"(.+)\s+(.+)\s+\((push|fetch)\)").unwrap();

    let capture = re.captures_iter(fetch_remote).next();

    if let Some(capture) = capture {
        let url = &capture[2];
        let repo_type = &capture[3];

        let url = Url::parse(url).expect("Unable to parse Git url");

        let (owner, name) = parse_git_url(url.path());

        let request_url = format!(
            "https://api.github.com/repos/{owner}/{repo}/issues",
            owner = owner,
            repo = name
        );

        let client = reqwest::blocking::Client::new();

        let access_token =
            env::var("GITHUB_ACCESS_TOKEN").expect("unable to get github access token");

        let res = client
            .get(request_url)
            .bearer_auth(access_token)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, owner)
            .send()
            .expect("Unable to get issues");

        let issues: Vec<RepoIssue> = res.json().expect("Unable to parse json resposne");

        return Some(issues);
    } else {
        return None;
    }
}

fn parse_git_url(url: &str) -> (&str, &str) {
    let repo_info: Vec<&str> = url.split("/").collect();
    let owner = repo_info.get(1).expect("Unable to get owner of repo");
    let name = repo_info
        .get(2)
        .expect("Unable to get name of repo")
        .trim_end_matches(".git");

    (owner, name)
}

fn get_lines(input: &str) -> Vec<&str> {
    let lines: Vec<&str> = input.split("\n").collect();

    lines
}
