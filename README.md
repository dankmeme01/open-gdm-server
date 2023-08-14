# OpenGDM server

A server for [Geometry Dash Multiplayer](https://github.com/AlizerUncaged/GDM-Windows), written in Rust and reverse engineered from scratch.

The server does can be easily self-hosted, and does not implement VIP checks, VIP features like rainbow colors, or private rooms. It also doesn't implement icon generation, and the `getIcon.php` endpoint acts as a proxy to the actual GDM server.

However, it still allows you to host your own server and connect to it with a GDM client. Note that GDM does not officially support custom server endpoints, you will have to modify the source code and compile it yourself with the proper endpoints.

By default, the server is bound to `0.0.0.0`, port 53789 for HTTP, and 53790 for the GDM protocol. The environment variable `GDM_PORT` can be used to change the GDM port, and the `Rocket.toml` file can be edited to change the HTTP port.