mod plotter;
mod circle;
mod types;
mod camera;

fn main() {
    let mut plt = plotter::Plot::new();

    let x : Vec<f32> = (-1..1).map(|f| f as f32).collect();
    let y = x.clone();
    let color = [1., 0., 0., 1.];
    let radius = 0.3;

    plt.scatter(&x, &y, radius, color).show();
}
