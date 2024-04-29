extern crate fflag_proc_macro;
use fflag_proc_macro::include_fflags;

include_fflags! {
    "version": "latest",
    "flags": {
        "DebugStudioAssertsAlwaysBreak": "DEBUG_STUDIO_ASSERTS_ALWAYS_BREAK",
        "HttpPointsReporterUrl": "HTTP_POINTS_REPORTER_URL"
    }
}

fn main() {
    println!("{}", DEBUG_STUDIO_ASSERTS_ALWAYS_BREAK);
    println!("{}", HTTP_POINTS_REPORTER_URL);
}
