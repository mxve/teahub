# TeaHub
#### Mirror GitHub repositories to Gitea
###### Inspired by [Gickup](https://github.com/cooperspencer/gickup)

TeaHub makes use of Gitea's mirroring feature and uses the GitHub and Gitea API to automate the process of mirroring GitHub repositories to Gitea.

## Featues
- Mirror GitHub repositories to Gitea (duh)
- Prefix repo name with owner name if not owned by user
  - Mirroring your own repo would result in `<yourgiteauser>/<reponame>`
  - Mirroring this repo would result in `<yourgiteauser>/mxve_teahub`
- Option to include starred repos
- Option to include private repos
- Option to keep all repos private
- Configurable (Gitea) mirror interval

## Usage
- Setup config file (see [config.example.toml](config.example.toml))
- Run teahub
- ???
- Profit

## Config
```toml
[github]
token = "" # GitHub personal access token
user = "" # GitHub username
include_starred = true # Include starred repositories
include_private = true # Include private repositories

[gitea]
token = "" # Gitea access token
user = "" # Gitea username
url = "" # Gitea URL (without trailing slash)
keep_private = true # Set all repositories to private, otherwise use the same visibility as on GitHub
mirror_interval = "2h" # Gitea mirror interval
```