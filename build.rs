fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/redeye-active.ico");
    println!("cargo:rerun-if-env-changed=RC_PATH");
    println!("cargo:rerun-if-env-changed=WindowsSdkVerBinPath");

    if std::env::var("CARGO_CFG_TARGET_OS")?.as_str() != "windows"
        || std::env::var("CARGO_CFG_TARGET_ENV")?.as_str() != "msvc"
    {
        return Err("RedEye requires a native Windows MSVC resource build.".into());
    }

    // Build-time only: rc.exe embeds every ICO frame as native PE resources.
    // The helper discovers the installed SDK; no runtime file is required.
    winresource::WindowsResource::new()
        .set_icon("assets/redeye-active.ico")
        .compile()
        .map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!("Cannot embed the RedEye executable icon: {error}. Ensure the Windows SDK resource compiler (rc.exe) is installed; RC_PATH can specify its full path."),
            )
        })?;
    Ok(())
}
