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
