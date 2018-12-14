# gabeln.jetzt

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
