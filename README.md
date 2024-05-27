
# Fast Flag Procedural Macro

A "simple" procedural macro for rust that loads FFlag values from Roblox Studio builds into your code.

Code is a bit scuffed but idrc.

## Usage/Examples
The syntax for the macro is just a simple JSON structure
```javascript
{
    "version": "" // either specific version or latest,
    "flags": {
        // FFlagName = name in the binary
        // FFLAG_VAR_NAME = name in code
        "FFlagName": "FFLAG_VAR_NAME" 
    }
}
```
### Static Flags
Static FFlags have values which are not in the binary.
They must be loaded via the include_fflags_runtime macro.
Their values can be viewed at: https://clientsettingscdn.roblox.com/v2/settings/application/PCStudioApp
```rust
extern crate fflag_macro;
use fflag_macro::{ include_fflags, include_fflags_runtime };

include_fflags! {
    "version": "latest", // or "version-e2728ac197f84660"
    "flags": {
        "DebugStudioAssertsAlwaysBreak": "DEBUG_STUDIO_ASSERTS_ALWAYS_BREAK",
        "HttpPointsReporterUrl": "HTTP_POINTS_REPORTER_URL"
    }
}

include_fflags_runtime! {
    "LuaGcStatsEphemeralCooldownSec": "LUA_GC_STATS_EPHEMERAL_COOLDOWN_SEC"
}

fn main() {
    assert_eq!(*LUA_GC_STATS_EPHEMERAL_COOLDOWN_SEC, 120);

    assert_eq!(DEBUG_STUDIO_ASSERTS_ALWAYS_BREAK, false);
    assert_eq!(HTTP_POINTS_REPORTER_URL, "https://client-telemetry.roblox.com");
}
```
