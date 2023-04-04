# Calling Your Video Game With Your Phone: Part 1

"Calling Your Video Game With Your Phone" is a 3-part series describing how you can use Twilio, Deepgram, and other technologies
to make phone calls which patch into your video game via a websocket server.

In this first part of the series, we will describe how to use Twilio to stream audio to a server, forward that audio
to Deepgram for transcription, and forward the transcription results to a game instance. An example game written
in Godot is shared, although the client code is simple enough to adapt to your needs, whether you use Unity,
or even a non-game front-end application. In fact, the server we will describe is fully test-able with
utilities such as `websocat`.

The server we will be walking through is written in Rust, but understanding the core architecture of the system
should allow you to build analogous servers in other languages.

## Pre-requisites

You will need the following:
* a Deepgram API Key
* a Twilio phone number

## Spinning Up the Server

First, clone the repository containing the server code:

```
git clone git@github.com:nikolawhallon/game-twilio-deepgram-distributor.git
```

Next, checkout the `stt` branch - this is the branch containing the code for "Part 1" in the "Calling Your Video Game With Your Phone" series:

```
cd game-twilio-deepgram-distributor
git checkout stt
```

Now, you can spin up the server by simply running `cargo run`. However, you will need the following environment variables set:

* `DEEPGRAM_API_KEY`: a Deepgram API Key to enable transcription
* `TWILIO_PHONE_NUMBER`: your Twilio phone number

When a client connects to the server, the server will send two text websocket messages to the client in a row. The first
will be the Twilio phone number, and the second will be a number between 0 and 100 which represents a unique code for patching
phone calls into client (game) sessions. What do we mean by this? Well, in order for a single Twilio phone number to serve multiple
ongoing client sessions, we need a way to associate a particular phone call with a particle client session. One way to do this is
to have each client session display a code unique to it - if you are playing a game and it displays the code "42," then no other
game session would also be displaying the code "42." Then, if you say this code on the phone, the server can tell that
that phonecall is associated with the game whose code is "42," and the server can start to pass transcription from your phonecall
into that particular game session.

## Setting up Twilio

Spin up the server locally and use `ngrok` to expose it. Then, in your Twilio Console, create a TwiML Bin like the following:

```
<?xml version="1.0" encoding="UTF-8"?>
<Response>
  <Say>This call may be monitored or recordered. Now, say the code you see in the game.</Say>
  <Connect>
    <Stream url="wss://8e8a-97-113-39-114.ngrok.io/twilio" />
  </Connect>
</Response>
```

Attach this TwiML Bin to your Twilio phone number. Check the Twilio documentation for more info.


## Testing With a Client

Testing with websocat is fairly easy. If you spin up the server locally on its default port (5000), just connect via:

```
websocat ws://127.0.0.1:5000/game
```

Call the phone number that websocat displays, and on the phone say the unique code that websocat also displays.
After that, you should start seeing Deepgram ASR responses stream into your websocat session.

A simple Godot game has been prepared for you to try this out in a game/game engine. Clone the following repository:

```
git@github.com:nikolawhallon/GodotPhonecall.git
```

And checkout the `stt` branch:

```
cd GodotPhonecall
git checkout stt
```

Then import the game with Godot 3.5, edit the file under `GodotPhonecall/Scenes/Game.gd`, and replace the url on line 16 with your server's url
(if you are running both the game and the server locally, and the server is listening on port 5000, then the url is probably already correct).
