mod plotter;
mod circle;
mod types;
mod camera;
mod renderer;
mod window_surface;
mod plot;
mod line;
mod rectangle;

use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let mut plt = plotter::Plotter::new();

    let x : Vec<f32> = (-1..=1).map(|f| f as f32).collect();
    let y = x.clone();
    let color = [1., 0., 0., 1.];
    let radius = 0.3;

    plt.scatter(x, y, radius, color)
       .show();
}
