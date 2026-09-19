# Connect your own YouTube channel

These steps are for channel connection and publishing. Downloading a permitted
source video through **New project → YouTube** does not require this setup.

1. Open [Google Cloud Console](https://console.cloud.google.com/) and create or
   select a project you control.
2. Enable the [YouTube Data API v3](https://console.cloud.google.com/apis/library/youtube.googleapis.com)
   in that project.
3. Open **Google Auth platform** and configure its branding and audience. For a
   personal development app, use the testing audience and add the Google account
   that owns your YouTube channel as a test user.
4. Under **Clients**, create an OAuth client with application type **Desktop app**.
   A web-application client or API key is not a substitute for desktop sign-in.
5. Download the client JSON file to your computer. Select it through ClipMill's
   channel setup in **Settings → YouTube**. Do not paste access tokens,
   refresh tokens or the client JSON into a chat, issue or source-control file.
6. Use **Connect channel** and complete Google's consent flow in your system
   browser. Check the actual channel name and ID shown after connection before
   choosing an export to upload.

Channel connection currently uses macOS Keychain; other platforms show it as
unavailable. The app uses the desktop authorization flow with a loopback callback, random
state and PKCE. Google account passwords remain in the browser; channel tokens
belong in the operating system credential store. Google's authoritative
[desktop OAuth guide](https://developers.google.com/identity/protocols/oauth2/native-app)
describes client setup and consent requirements. A testing OAuth application can
require periodic reauthorization; reconnecting is preferable to copying tokens.

Uploads should begin as **private**. Review the finished video in YouTube, then
use the separate explicit Publish action when you want it public. Google
restricts uploads from unverified API projects to private visibility until the
project passes the relevant audit. A working sign-in or successful private upload
does not prove public publishing is available for your project. See Google's
[upload API requirements](https://developers.google.com/youtube/v3/docs/videos/insert).

ClipMill implementation and protocol tests can be completed without your account.
A real channel connection and private-upload check require your own client
configuration and browser consent; no account verification should be reported
until those steps have actually happened.
