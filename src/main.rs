use color_eyre::Result;
mod audio;
mod fft;
mod tui;

fn main() -> Result<()> {
    std::env::set_var(
        "PIPEWIRE_ALSA",
        "{ node.name=\"rust-vis\" stream.capture.sink=true }",
    );

    color_eyre::install()?;

    let terminal = ratatui::init();

    let app_result = tui::App::new().run(terminal);

    ratatui::restore();

    app_result
}
