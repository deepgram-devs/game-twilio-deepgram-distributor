# game-twilio-deepgram-distributor

This is a websockets server which communicates with Twilio, Deepgram, and game clients (well any client really).
When a client connects to the `/game` endpoint, the server will send two text messages. The first one contains a string
which should be interpretted as a phone number. The second one contains a unique code for the client. If you then
call the phone number and say the unique code, the server will start to proxy transcriptions of your speech on the phone
to the client, using Deepgram's ASR.

The intention, then, is to be able to have, say, a video game connect to the server, display the phone number and
unique code to the player, and then the player can call in and start issuing commands to the game via their phone.
This version of the server has been extended to handle text-to-speech as well, allowing players to call into the game
and engage with NPCs via some kind of conversational AI flow or something.

## Setting up Twilio

I would spin up this server locally and use `ngrok` to expose it. Then create a TwiML Bin like the following:

```
<?xml version="1.0" encoding="UTF-8"?>
<Response>
  <Say>Say the code you see in the game.</Say>
  <Connect>
    <Stream url="wss://8e8a-97-113-39-114.ngrok.io/twilio" />
  </Connect>
</Response>
```

Attach this TwiML Bin to one of your Twilio phone numbers. Check the Twilio documentation for more info.

## Spinning Up the Server

You can spin up the server by simply running `cargo run`. However, you will need the following environment variables set:

* `DEEPGRAM_API_KEY`: a Deepgram API Key to enable transcription
* `TWILIO_PHONE_NUMBER`: your Twilio phone number using the TwiML Bin described in a previous section
* `AWS_REGION`: the AWS region to use for Polly (`us-west-2` should be fine)
* `AWS_ACCESS_KEY_ID`: AWS Key ID for Polly
* `AWS_SECRET_ACCESS_KEY`: AWS Secret Access Key for Polly


## Testing With a Client

Testing with websocat is fairly easy. If you spin up the server locally, just connect via:

```
websocat ws://127.0.0.1:5000/game
```

Call the phone number that websocat spits out, and on the phone say the unique code that websocat also spits out.
After that, you should start seeing Deepgram ASR responses stream into your websocat session.

If you want to try this out in a game/game engine, see the simple demo here:

https://github.com/nikolawhallon/GodotPhonecall

## Notes From Deployment

I deployed following my typical procedure (link(s) to come).

My file `/etc/nginx/sites-available/robotdreams.deepgram.com` had the following contents:

```
server {
    root /var/www/robotdreams.deepgram.com/html;
    index index.html index.htm index.nginx-debian.html;

    server_name robotdreams.deepgram.com;

    location / {
        proxy_pass http://0.0.0.0:5000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "Upgrade";
        proxy_set_header Connection "keep-alive";
        proxy_set_header Host $host;
    }

    listen [::]:443 ssl ipv6only=on; # managed by Certbot
    listen 443 ssl; # managed by Certbot
    ssl_certificate /etc/letsencrypt/live/robotdreams.deepgram.com/fullchain.pem; # managed by Certbot
    ssl_certificate_key /etc/letsencrypt/live/robotdreams.deepgram.com/privkey.pem; # managed by Certbot
    include /etc/letsencrypt/options-ssl-nginx.conf; # managed by Certbot
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem; # managed by Certbot
}
server {
    if ($host = robotdreams.deepgram.com) {
        return 301 https://$host$request_uri;
    } # managed by Certbot

    listen 80;
    listen [::]:80;

    server_name robotdreams.deepgram.com;
    return 404; # managed by Certbot
}
```

My `docker-compose.yml` file had the following contents:

```
version: '3.7'
services:
  web:
    image: browncanstudios/robot-dreams-server:0.1.0
    ports:
      - "5000:5000"
    volumes:
      - /home/ubuntu/config.json:/config.json:ro
    environment:
      - PROXY_URL=0.0.0.0:5000
      - DEEPGRAM_URL=wss://api.deepgram.com/v1/listen?encoding=mulaw&sample_rate=8000&numerals=true&tier=enhanced&interim_results=true
      - AWS_REGION=us-west-2
      - AWS_ACCESS_KEY_ID=SECRET
      - AWS_SECRET_ACCESS_KEY=SECRET
      - TWILIO_PHONE_NUMBER=SECRET
      - DEEPGRAM_API_KEY=SECRET
    command: --config=config.json
```

The `config.json` file had the following contents:

```
{
    "game_codes":["apple"]
}
```

Finally, I was able to connect with the following:

```
websocat wss://robotdreams.deepgram.com:443/game
```

Things I understand enough:
* The `PROXY_URL` needs to be `0.0.0.0`, not `127.0.0.1` or `localhost` inside the docker container.

Things I don't understand well:
* Why did I need to specify `443` in the websockets URL?
* Why didn't I need to provide my Rust server `CERT_PEM` or `KEY_PEM`?
* What all is going on with the `nginx` configuration?
* I had to remove the `CMD` block from my `Dockerfile` - why was it populating a command with `''`? (Maybe this makes sense actually.)
