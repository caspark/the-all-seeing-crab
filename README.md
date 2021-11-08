# The All Seeing Crab

is a raytracer written in Rust, with the core ray tracing mechanics adhering fairly closely to [Ray Tracing in One Weekend](https://raytracing.github.io/).

![GIF showing incremental rendering in action](/../media/incremental-rendering.gif?raw=true "Demo of incremental rendering for terminal and UI")

The primary differences from the "original" are:

1. It's written in Rust (obviously) instead of C++
2. Support for various debug rendering modes is added (e.g. render normals, depth testing, etc)
3. A few more rendering scenes have been added
4. Terminal progress display: the scene is incrementally rendered in ascii-art-lite in your terminal as it is computed
5. A GUI is provided for tweaking various camera, scene, and other configuration settings (using [egui](https://github.com/emilk/egui)), with the resulting image being displayed in the GUI too.

Here's a screenshot showing most of the fancy features:

![Screenshot showing UI with rendered scene](/../media/ui-screenshot.png?raw=true "Demo scene shown with UI")

## Prereqs

* Rust, obviously
* eframe dependencies: `sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

If you are running this under WSL with an X11 server like vcxsrv, make sure:

* Hardware acceleration is enabled in vcxsrv & you run with `env LIBGL_ALWAYS_INDIRECT=1 <command>`
* If you have 2 graphics cards (e.g. Nvidia Optimus), make sure your X11 executable (e.g. vcxsrv.exe) is set to run with your proper graphics card, not your low power card (which probably doesn't support the things that eframe needs it to support).

## Running

Run `./run.sh` and the output of the render will be displayed & saved to `target/output.png`.

## Developing

1. Run `./watch.sh`
2. Optionally, open `target/output.png` in an editor split, so you can see it auto-update

To run (very very minimal) tests, run `cargo test` (or `cargo watch`) as usual.
