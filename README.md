# Cup notifier

Small Rust service that polls a [Cup](https://github.com/sergi0g/cup) instance for monitored container images updates and sends notifications to a Discord webhook.

## Features
- Polls Cup's API periodically (configurable cron)
- Sends Discord webhook embeds for images with available updates
- Keeps an in-memory cache to avoid duplicate notifications across runs

## Configuration
The application expects a YAML config file (default `config.yaml`) describing at least:

```yaml
webhook_url: "https://discord.com/api/webhooks/..../.."
cup_base_url: "http://localhost:8000/"
cron: "0 0 * * *" # optional (default: "0 0 * * *")
```

All config values can be provided via environment variables with the `CUP_NOTIFIER_` prefix. For example `CUP_NOTIFIER_WEBHOOK_URL`.

## Running

Below is an example `docker-compose.yaml` you can adapt. It runs the built binary and shows two ways to provide configuration: mounting a YAML file or using environment variables.

```yaml
services:
  cup:
    image: ghcr.io/sergi0g/cup:latest
    restart: unless-stopped
    command: serve
    ports:
      - 8000:8000
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock

  cup-notifier:
    image: ghcr.io/ceski23/cup-notifier:latest
    restart: unless-stopped
    # 1) Option A: mount a config file
    volumes:
      - ./config.yaml:/config.yaml:ro
    # 2) Option B: or set environment variables
    # environment:
    #   CUP_NOTIFIER_WEBHOOK_URL: "https://discord.com/api/webhooks/..."
    #   CUP_NOTIFIER_CUP_BASE_URL: "https://your-cup.example.com/"
    #   CUP_NOTIFIER_CRON: "0 0 * * *"
```