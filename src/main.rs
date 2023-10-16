use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
mod config;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct RepoOwner {
    login: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Repo {
    private: bool,
    fork: bool,
    clone_url: String,
    name: String,
    full_name: String,
    owner: RepoOwner,
}

#[derive(PartialEq)]
enum RepoType {
    Gitea,
    GitHub,
}

fn get_github(token: String, endpoint: String) -> String {
    let endpoint = format!("https://api.github.com{}", endpoint);
    let mut buffer: Vec<u8> = Vec::new();
    match http_req::request::Request::new(&endpoint.as_str().try_into().unwrap())
        .header("User-Agent", "teahub")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", &format!("Bearer {}", &token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send(&mut buffer)
    {
        Ok(_) => String::from_utf8(buffer).unwrap(),
        Err(error) => panic!("Error: {:?}", error),
    }
}

fn get_gitea(config: &config::CGitea, endpoint: &String) -> String {
    let endpoint = format!("{}/api/v1{}", config.url, endpoint);
    let mut buffer: Vec<u8> = Vec::new();
    match http_req::request::Request::new(&endpoint.as_str().try_into().unwrap())
        .header("User-Agent", "teahub")
        .header("Accept", "application/json")
        .header("Authorization", &format!("token {}", &config.token))
        .send(&mut buffer)
    {
        Ok(_) => String::from_utf8(buffer).unwrap(),
        Err(error) => panic!("Error: {:?}", error),
    }
}

fn get_repos(config: &config::Config, endpoint: &String, repo_type: RepoType) -> Vec<Repo> {
    let mut page: u8 = 1;
    let mut repos: Vec<Repo> = Vec::new();

    loop {
        let response = if repo_type == RepoType::GitHub {
            let endpoint = if endpoint.contains('?') {
                format!("{}&per_page=100&page={}", endpoint, page)
            } else {
                format!("{}?per_page=100&page={}", endpoint, page)
            };
            get_github(config.github.token.clone(), endpoint)
        } else {
            get_gitea(&config.gitea, endpoint)
        };

        let mut page_repos: Vec<Repo> = serde_json::from_str(&response).unwrap();
        let page_repos_cnt = page_repos.len(); // store repo count, value will be moved

        repos.append(&mut page_repos);
        if page_repos_cnt < 100 {
            break;
        }
        page += 1;
    }

    repos
}

fn collect_github_repos(config: &config::Config) -> Vec<Repo> {
    let endpoint = if config.github.include_private {
        "/user/repos"
    } else {
        "/user/repos?visibility=public"
    };
    let mut repos = get_repos(config, &endpoint.to_string(), RepoType::GitHub);

    if config.github.include_starred {
        let mut starred = get_repos(config, &"/user/starred".to_string(), RepoType::GitHub);
        repos.append(&mut starred);
    }

    repos
}

#[derive(Serialize)]
struct MigrateRepoOptions {
    auth_token: String,
    clone_addr: String,
    description: String,
    issues: bool,
    lfs: bool,
    mirror: bool,
    mirror_interval: String,
    private: bool,
    pull_requests: bool,
    releases: bool,
    repo_name: String,
    repo_owner: String,
    wiki: bool,
}

fn mirror_repo(repo: &Repo, config: &config::Config) {
    let mut payload = MigrateRepoOptions {
        auth_token: config.github.token.clone(),
        clone_addr: repo.clone_url.clone(),
        description: format!("Mirror of {}", repo.clone_url),
        issues: true,
        lfs: true,
        mirror: true,
        mirror_interval: "2h".to_string(),
        private: true,
        pull_requests: true,
        releases: true,
        repo_name: repo.name.clone(),
        repo_owner: config.gitea.user.clone(),
        wiki: true,
    };
    if !config.gitea.keep_private {
        payload.private = repo.private;
    }

    // prefix repo name with owner name if owner is not the github user
    if repo.owner.login != config.github.user {
        payload.repo_name = format!("{}_{}", repo.owner.login, repo.name);
    }

    println!(
        "Mirroring {} to {}/{}",
        repo.full_name, payload.repo_owner, payload.repo_name
    );

    let payload = serde_json::to_string(&payload).unwrap();
    let endpoint = format!("{}/api/v1/repos/migrate", config.gitea.url);

    let mut buffer: Vec<u8> = Vec::new();
    match http_req::request::Request::new(&endpoint.as_str().try_into().unwrap())
        .method(http_req::request::Method::POST)
        .body(payload.as_bytes())
        .header("User-Agent", "teahub")
        .header("Content-Length", &payload.len())
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("token {}", &config.gitea.token))
        .send(&mut buffer)
    {
        Ok(req) => {
            if req.status_code() == http_req::response::StatusCode::new(409) {
                println!("{} already exists", repo.name);
            } else if req.status_code() != http_req::response::StatusCode::new(201) {
                panic!("Error: {:?}", req);
            }
        }
        Err(error) => panic!("Error: {:?}", error),
    }
}

fn main() {
    let config = config::load_config(PathBuf::from("config.toml"));
    let github_repos = collect_github_repos(&config);
    let gitea_repos = get_repos(&config, &"/user/repos".to_string(), RepoType::Gitea);

    let gitea_repos_names: Vec<String> = gitea_repos.iter().map(|r| r.name.clone()).collect();
    for repo in github_repos {
        let prefixed_name = format!("{}_{}", repo.owner.login, repo.name);
        if gitea_repos_names.contains(&repo.name) || gitea_repos_names.contains(&prefixed_name) {
            println!("{} already exists", repo.name);
            continue;
        }

        mirror_repo(&repo, &config);
    }
}
