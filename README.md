# The All Seeing Crab

is a raytracer written in Rust.

## Prereqs

* Rust, obviously
* eframe dependencies: `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

If you are running this under WSL with an X11 server like vcxsrv, make sure:

* Hardware acceleration is enabled in vcxsrv & you run with `env LIBGL_ALWAYS_INDIRECT=1 <command>`
* If you have 2 graphics cards (e.g. Nvidia Optimus), make sure your X11 executable (e.g. vcxsrv.exe) is set to run with your proper graphics card, not your low power card (which probably doesn't support the things that eframe needs it to support).

## Running

Run `./run.sh output.png` and the output of the render will be saved to `output.png`

## Developing

1. Run `./watch.sh`
2. Open `target/output.png` in an editor split, so you can see it auto-update

To run tests, run `cargo test` (or `cargo watch`) as usual.
