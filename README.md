# gabeln.jetzt

[![](https://img.shields.io/travis/verkehrsministerium/gabeln.jetzt.svg)](https://travis-ci.org/verkehrsministerium/gabeln.jetzt)
[![](https://img.shields.io/docker/automated/fin1ger/gabeln.jetzt.svg)](https://cloud.docker.com/repository/docker/fin1ger/gabeln.jetzt)
[![](https://img.shields.io/docker/build/fin1ger/gabeln.jetzt.svg)](https://cloud.docker.com/repository/docker/fin1ger/gabeln.jetzt/tags)
![](https://img.shields.io/microbadger/image-size/fin1ger%2Fgabeln.jetzt.svg)
[![](https://img.shields.io/github/tag/verkehrsministerium/gabeln.jetzt.svg)](https://github.com/verkehrsministerium/gabeln.jetzt/releases)
[![Built with Spacemacs](https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg)](http://spacemacs.org)

---

This repository hosts a webserver that displays github fork events and sends events via a telegram bot.

## How to build?

```
$ cargo build
```

## How to run?
```
$ USERS="github-user1,github-user2" \
    GITHUB_OAUTH_TOKEN="<github-oauth-app-token>" \
    GIPHY_API_KEY="<giphy-api-key>" \
    TELEGRAM_BOT_TOKEN="<telegram-bot-token>" \
    cargo run
```

## How to use docker image?

### Environment variables

| Name                 | Function                                                                                                                             |
|----------------------|--------------------------------------------------------------------------------------------------------------------------------------|
| `USERS`              | A comma separated list of github usernames                                                                                           |
| `GITHUB_OAUTH_TOKEN` | The github OAuth API Token. Can be created [like this](https://developer.github.com/apps/building-oauth-apps/creating-an-oauth-app/) |
| `TELEGRAM_BOT_TOKEN` | The telegram bot token. Can be created [like this](https://core.telegram.org/bots#creating-a-new-bot)                                |
| `GIPHY_API_KEY`      | The Giphy API key. Can be created [here](https://developers.giphy.com/)                                                              |

### Run the image

```
$ docker run \
    -e USERS="..." \
    -e GITHUB_OAUTH_TOKEN="..." \
    -e GIPHY_API_KEY="..." \
    -e TELEGRAM_BOT_TOKEN="..." \
    -p 80:8000 \
    fin1ger/gabeln.jetzt
```
