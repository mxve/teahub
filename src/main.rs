use serde_derive::Deserialize;
use std::path::PathBuf;
mod config;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GHOwner {
    login: String,
    r#type: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GHRepo {
    private: bool,
    fork: bool,
    clone_url: String,
    name: String,
    full_name: String,
    owner: GHOwner,
}

fn req_github(token: String, endpoint: String) -> String {
    let endpoint = format!("https://api.github.com{}", endpoint);
    let mut buffer: Vec<u8> = Vec::new();
    match http_req::request::Request::new(&endpoint.as_str().try_into().unwrap())
        .header("User-Agent", "teahub")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", &format!("Bearer {}", &token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send(&mut buffer)
    {
        Ok(_) => {
            let response = String::from_utf8(buffer).unwrap();
            response
        }
        Err(error) => panic!("Error: {:?}", error),
    }
}

fn req_github_repos(token: String, endpoint: String) -> Vec<GHRepo> {
    let mut page: u8 = 1;
    let mut repos: Vec<GHRepo> = Vec::new();

    loop {
        let endpoint = if endpoint.contains("?") {
            format!("{}&per_page=100&page={}", endpoint, page)
        } else {
            format!("{}?per_page=100&page={}", endpoint, page)
        };
        let response = req_github(token.clone(), endpoint);
        let mut page_repos: Vec<GHRepo> = serde_json::from_str(&response).unwrap();
        let page_repos_cnt = page_repos.len(); // store repo count, value will be moved

        println!("Page {} has {} repos", page, page_repos_cnt);
        repos.append(&mut page_repos);
        if page_repos_cnt < 100 {
            break;
        }
        page += 1;
    }

    repos
}

fn get_github_repos(token: String, include_private: bool, include_starred: bool) -> Vec<GHRepo> {
    let endpoint = if include_private {
        "/user/repos"
    } else {
        "/user/repos?visibility=public"
    };
    let mut repos = req_github_repos(token.clone(), endpoint.to_string());

    if include_starred {
        let mut starred = req_github_repos(token.clone(), "/user/starred".to_string());
        repos.append(&mut starred);
    }

    repos
}

fn main() {
    let config = config::load_config(PathBuf::from("config.toml"));
    let github_repos = get_github_repos(
        config.github.token,
        config.github.include_private,
        config.github.include_starred,
    );
    println!("{:?}", github_repos);
}
