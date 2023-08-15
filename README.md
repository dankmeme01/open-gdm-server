# OpenGDM server

A server for [Geometry Dash Multiplayer](https://github.com/AlizerUncaged/GDM-Windows), written in Rust and reverse engineered from scratch.

The server can be easily self-hosted, and does not implement VIP checks, VIP features like rainbow colors, or private rooms. It also doesn't implement icon generation, and the `getIcon.php` endpoint acts as a proxy to the actual GDM server. However, the rest of the functionality is intact.

By default, the server is bound to `0.0.0.0` (all addresses), port 53789 for HTTP, and 53790 for the GDM protocol. Those all can be changed with environment variables `BIND_ADDRESS`, `HTTP_PORT` and `GDM_PORT`

Also, I love how I got to do this project not 2 years ago, but 2 months before 2.2 comes out and this becomes completely useless just as everything else I ever do :D

## How to connect

GDM does not officially support custom server endpoints, and you will have to modify the source code and compile it yourself with the proper endpoints.

For each of those places, change the IP address or the hostname to `127.0.0.1:53789`, for example `http://1.1.1.1/gdm/getIcon.php` or `http://example.com/gdm/getIcon.php` becomes `http://127.0.0.1:53789/gdm/getIcon.php`. Make sure to replace `127.0.0.1` to the server IP and `53789` to the HTTP port of the OpenGDM server:

* GDM/Client/Client.cs line 343
* GDM/Client/Client.cs line 395
* GDM/Globals/Endpoints.cs line 16
* GDM/Globals/Global Data.cs line 37 (remove the 2nd server, keep just 1 with your IP)
* GDM/Globals/Global Data.cs line 45
* GDM/Initialize.cs line 167
* GDM/Initialize.cs line 738
* Utilities/TCP.cs line 68
* Utilities/TCP.cs line 71
* Utilities/TCP.cs line 94
* Utilities/TCP.cs line 146

Then, change the file at GDM/Globals/Global Data.cs line 35, set the `StandardPort` variable to your GDM port of the OpenGDM server.

Additionally, I would recommend to switch the .NET version to 4.8, as I've had issues with both 4.6.1 and 4.6.2.

After those changes, you should be able to compile GDM successfully, and it should point to your server.

## Potential future plans

* Implement `getIcon.php` ourselves
* Figure out why the level list is broken on the client