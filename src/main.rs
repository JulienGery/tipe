mod plotter;
mod plot;
mod window_surface;
mod circles;
mod circle_manadger;
mod renderer;

use std::env;

fn main() {

    env::set_var("RUST_BACKTRACE", "1");
    let mut plt = plotter::Plotter::new();

    let x1 : Vec<f32> = (-1..=1).map(|f| f as f32).collect();
    let y1 = x1.clone();
    let color = [1., 0., 0., 1.];
    let radius = 0.3;

    let x2 = vec![0., 0.];
    let y2 = vec![0., 1.];

    plt.scatter(x1, y1, radius, color)
       .new_plot()
       .scatter(x2, y2, radius, [1., 1., 0., 1.])
       .show();
}
