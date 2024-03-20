use std::f64::consts::PI;

use num::complex::Complex;

fn expi<T>(theta : T) -> Complex<T>
where T: num::Float,
{
    Complex::new(theta.cos(), theta.sin())
}


//https://numpy.org/doc/stable/reference/routines.fft.html
//raw definition of dft.
//o(n^2)
pub fn dtf(values: &Vec<f64>) -> Vec<Complex<f64>>
{
    let n = values.len();
    (0..n).map(|k| values.iter()
             .enumerate()
             .map(|(m, a)| a * expi(-2.0 * PI * (m * k) as f64 / n as f64))
             .sum()
    )
    .collect()
}

pub fn my_fft(x: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    if x.len() <= 1 {
        x.clone()
    } else {
        let n = x.len();
        let x_odd: Vec<Complex<f64>> = (1..n).step_by(2).map(|i| x[i]).collect();
        let x_even: Vec<Complex<f64>> = (0..n).step_by(2).map(|i| x[i]).collect();
        let factors: Vec<Complex<f64>> = (0..(n / 2))
            .map(|i| Complex::new(0.0, -2.0 * PI * i as f64 / n as f64).exp())
            .collect();
        let x_odd_fft = my_fft(&x_odd);
        let x_even_fft = my_fft(&x_even);
        let mut result = Vec::with_capacity(n);
        for i in 0..(n / 2) {
            result.push(x_even_fft[i] + factors[i] * x_odd_fft[i]);
        }
        for i in 0..(n / 2) {
            result.push(x_even_fft[i] - factors[i] * x_odd_fft[i]);
        }
        result
    }
}

pub fn fftfreq(n : usize, d : f64) -> Vec<f64> {
    let f = if n % 2 == 0 { 0..=(n / 2 - 1) } else { 0..=(n - 1) / 2 };
    let g = if n % 2 == 0 { 1..=(n / 2) } else { 1..=(n - 1) / 2};

    f.map(|v| v as f64 / (d * n as f64))
     .chain(g.rev()
             .map(|v| -(v as f64) / (d * n as f64))
     )
     .collect()
}
