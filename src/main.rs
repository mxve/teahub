use serde_derive::Deserialize;
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
    let mut repos = get_repos(&config, &endpoint.to_string(), RepoType::GitHub);

    if config.github.include_starred {
        let mut starred = get_repos(&config, &"/user/starred".to_string(), RepoType::GitHub);
        repos.append(&mut starred);
    }

    repos
}

fn main() {
    let config = config::load_config(PathBuf::from("config.toml"));
    let github_repos = collect_github_repos(&config);
    println!("GH repos: {:?}", github_repos.len());
    let gitea_repos = get_repos(&config, &"/user/repos".to_string(), RepoType::Gitea);
    println!("GT repos: {:?}", gitea_repos.len());
}
