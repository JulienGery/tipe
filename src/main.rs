mod plotter;
mod plot;
mod window_surface;
mod circles;

use std::env;

fn main() {

    env::set_var("RUST_BACKTRACE", "1");
    let mut plt = plotter::Plotter::new();

    let x : Vec<f32> = (-1..=1).map(|f| f as f32).collect();
    let y = x.clone();
    let color = [1., 0., 0., 1.];
    let radius = 0.3;

    plt.scatter(x.clone(), y.clone(), radius, color)
       .new_plot()
       .scatter(x.clone(), y.clone(), radius, color)
       .show();
}
