
# Fast Flag Procedural Macro

A simple procedural macro for rust that loads FFlag values from Roblox Studio builds into your code.

Code is a bit scuffed but idrc.

## IMPORTANT
Some FFlag values are not initialized in the Studio binary and will panic when loaded by the macro.
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
```rust
use fflag_proc_macro::include_fflags;

include_fflags! {
    "version": "latest", // or "version-e2728ac197f84660"
    "flags": {
        "DebugStudioAssertsAlwaysBreak": "DEBUG_STUDIO_ASSERTS_ALWAYS_BREAK",
        "HttpPointsReporterUrl": "HTTP_POINTS_REPORTER_URL"
    }
}

fn main() {
    assert_eq!(DEBUG_STUDIO_ASSERTS_ALWAYS_BREAK, false);
    assert_eq!(tHTTP_POINTS_REPORTER_URL, "https://client-telemetry.roblox.com");
}
```
