mod plotter;
mod plot;
mod window_surface;
mod circles;
mod circle_manadger;
mod renderer;
mod camera;
mod fft;

use std::env;

fn main() {

    env::set_var("RUST_BACKTRACE", "1");
    let mut plt = plotter::Plotter::new();

    let x1 : Vec<f32> = (-1..=1).map(|f| f as f32).collect();
    let y1 = x1.clone();
    let radius = 0.5;

    let x2 = vec![0.];
    let y2 = vec![0.];

    let mut x3 = x1.clone();
    x3.reverse();
    let y3 = y1.clone();

    plt.scatter(x1, y1, radius, [1., 0., 0., 1.])
        .scatter(x3, y3, radius, [1., 1., 1., 1.])
       .new_plot()
       .scatter(x2, y2, radius, [1., 1., 0., 1.])
       .show();
}
