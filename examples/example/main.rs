extern crate fflag_macro;
use fflag_macro::{ include_fflags, include_fflags_runtime };

include_fflags! {
    "version": "latest",
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