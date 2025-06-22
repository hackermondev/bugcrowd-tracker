# Bugcrowd Tracker
[![GitHub stars](https://img.shields.io/github/stars/hackermondev/bugcrowd-tracker)](https://github.com/hackermondev/bugcrowd-tracker/stargazers)
[![License](https://img.shields.io/github/license/hackermondev/bugcrowd-tracker)](LICENSE)

## Overview
Monitor disclosed Crowstream reports and Hall of Fame updates from Bugcrowd, and receive a Discord notification when new reports are disclosed or HoF updates are made.

![demo leaderboard](https://ninja.dog/ydFERU.png)

![demo crowdstream](https://ninja.dog/d1QPhw.png)


## Installation (requires [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/))

Clone the repository:
```bash
git clone https://github.com/hackermondev/bugcrowd-tracker
```

### Update configuration
Edit the `.env.example` file in the root of the repository and save it as `.env`:
```bash
# Bugcrowd engagement handle
BUGCROWD_ENGAGEMENT=
# Discord webhook URL (ex. https://discord.com/api/webhooks/<id>/<token>)
DISCORD_WEBHOOK_URL=
# Bugcrowd session token, required for private engagements
#SESSION_TOKEN=
```

### Start the application
Run the following command in the root of the repository:
```bash
docker compose up -d
```
