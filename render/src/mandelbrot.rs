use std::ops::{Add, Mul};
use rayon::prelude::*;
use num_bigfloat::BigFloat;

const PRECISION: usize = 100;  // Number of decimal places for precision

#[derive(Clone)]
pub struct HighPrecComplex {
    real: BigFloat,
    imag: BigFloat,
}

impl HighPrecComplex {
    fn new(real: f64, imag: f64) -> Self {
        HighPrecComplex {
            real: BigFloat::from(real),
            imag: BigFloat::from(imag),
        }
    }

    fn magnitude_squared(&self) -> BigFloat {
        let r = self.real.clone();
        let i = self.imag.clone();
        r * r + i * i
    }

    fn to_complex(&self) -> Complex {
        Complex::new(
            self.real.to_f64(),
            self.imag.to_f64()
        )
    }

    fn mul(&self, other: &HighPrecComplex) -> HighPrecComplex {
        let r1 = self.real.clone();
        let i1 = self.imag.clone();
        let r2 = other.real.clone();
        let i2 = other.imag.clone();

        HighPrecComplex {
            real: r1.clone() * r2.clone() - i1.clone() * i2.clone(),
            imag: r1 * i2 + i1 * r2,
        }
    }

    fn add(&self, other: &HighPrecComplex) -> HighPrecComplex {
        HighPrecComplex {
            real: self.real.clone() + other.real.clone(),
            imag: self.imag.clone() + other.imag.clone(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Complex {
    real: f64,
    imag: f64,
    error: f64,  // Track accumulated error
}

impl Complex {
    pub fn new(real: f64, imag: f64) -> Self {
        Complex { 
            real, 
            imag, 
            error: 0.0 
        }
    }

    pub fn with_error(real: f64, imag: f64, error: f64) -> Self {
        Complex { real, imag, error }
    }

    pub fn magnitude_squared(&self) -> f64 {
        self.real * self.real + self.imag * self.imag
    }

    pub fn scale(&self, factor: f64) -> Complex {
        Complex {
            real: self.real * factor,
            imag: self.imag * factor,
            error: self.error,
        }
    }
}

impl Add for Complex {
    type Output = Complex;

    fn add(self, other: Complex) -> Complex {
        Complex {
            real: self.real + other.real,
            imag: self.imag + other.imag,
            error: self.error + other.error,
        }
    }
}

impl Mul for Complex {
    type Output = Complex;

    fn mul(self, other: Complex) -> Complex {
        Complex {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.real * other.imag + self.imag * other.real,
            error: self.error + other.error,
        }
    }
}

// Add this new struct for matrix-based calculations
#[derive(Clone, Copy)]
struct Matrix2x2 {
    a11: f64, a12: f64,
    a21: f64, a22: f64,
}

impl Matrix2x2 {
    fn new(a11: f64, a12: f64, a21: f64, a22: f64) -> Self {
        Matrix2x2 { a11, a12, a21, a22 }
    }

    fn mul_complex(&self, z: &Complex) -> Complex {
        Complex::new(
            self.a11 * z.real + self.a12 * z.imag,
            self.a21 * z.real + self.a22 * z.imag
        )
    }
}

pub struct MandelbrotFrame {
    pub width: u32,
    pub height: u32,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub max_iterations: u32,
    reference_point: Complex,
    reference_orbit: Vec<Complex>,
    high_prec_reference: Option<HighPrecComplex>,
    high_prec_orbit: Vec<HighPrecComplex>,
}

impl MandelbrotFrame {
    pub fn new(width: u32, height: u32) -> Self {
        MandelbrotFrame {
            width,
            height,
            x_min: -2.0,
            x_max: 1.0,
            y_min: -1.5,
            y_max: 1.5,
            max_iterations: 100,
            reference_point: Complex::new(0.0, 0.0),
            reference_orbit: Vec::new(),
            high_prec_reference: None,
            high_prec_orbit: Vec::new(),
        }
    }

    pub fn calculate(&mut self) -> Vec<u32> {
        let zoom_level = 1.0 / (self.x_max - self.x_min).abs();
        
        // Use high precision for deep zooms
        let use_high_precision = zoom_level > 1e14;
        
        if use_high_precision {
            self.calculate_high_precision()
        } else {
            self.calculate_standard()
        }
    }

    fn calculate_high_precision(&mut self) -> Vec<u32> {
        let mut result = vec![0; (self.width * self.height) as usize];
        
        // Calculate center point
        let center_x = (self.x_min + self.x_max) / 2.0;
        let center_y = (self.y_min + self.y_max) / 2.0;
        
        self.high_prec_reference = Some(HighPrecComplex::new(center_x, center_y));
        self.calculate_high_precision_orbit();

        result.chunks_mut(self.width as usize)
            .enumerate()
            .par_bridge()
            .for_each(|(y, row)| {
                for x in 0..self.width {
                    let x_coord = self.x_min + (x as f64 / self.width as f64) * (self.x_max - self.x_min);
                    let y_coord = self.y_min + (y as f64 / self.height as f64) * (self.y_max - self.y_min);
                    
                    let c = HighPrecComplex::new(x_coord, y_coord);
                    row[x as usize] = self.iterate_high_precision(&c);
                }
            });

        result
    }

    fn calculate_high_precision_orbit(&mut self) {
        self.high_prec_orbit.clear();
        let mut z = HighPrecComplex::new(0.0, 0.0);
        let c = self.high_prec_reference.as_ref().unwrap();
        
        self.high_prec_orbit.reserve(self.max_iterations as usize);
        
        for _ in 0..self.max_iterations {
            if z.magnitude_squared() > BigFloat::from(4.0) {
                break;
            }
            self.high_prec_orbit.push(z.clone());
            
            // z = z^2 + c
            z = z.mul(&z).add(c);
        }
    }

    fn iterate_high_precision(&self, c: &HighPrecComplex) -> u32 {
        let mut z = HighPrecComplex::new(0.0, 0.0);
        let mut n = 0;

        while z.magnitude_squared() <= BigFloat::from(4.0) && n < self.max_iterations as usize {
            z = z.mul(&z).add(c);
            n += 1;
        }

        if n < self.max_iterations as usize {
            let mag = z.magnitude_squared().to_f64();
            n as u32 + 1 - (mag.ln().ln() / 2.0_f64.ln()).floor() as u32
        } else {
            self.max_iterations
        }
    }

    fn calculate_standard(&mut self) -> Vec<u32> {
        // Calculate center point for reference orbit
        let center_x = (self.x_min + self.x_max) / 2.0;
        let center_y = (self.y_min + self.y_max) / 2.0;
        self.reference_point = Complex::new(center_x, center_y);
        
        // Calculate reference orbit
        self.calculate_reference_orbit();
        
        let mut result = vec![0; (self.width * self.height) as usize];
        
        result.par_chunks_mut(self.width as usize)
            .enumerate()
            .for_each(|(y, row)| {
                for x in 0..self.width {
                    let x_coord = self.x_min + (x as f64 / self.width as f64) * (self.x_max - self.x_min);
                    let y_coord = self.y_min + (y as f64 / self.height as f64) * (self.y_max - self.y_min);
                    
                    let c = Complex::new(x_coord, y_coord);
                    row[x as usize] = self.iterate_standard(c);
                }
            });
        
        result
    }

    fn calculate_reference_orbit(&mut self) {
        self.reference_orbit.clear();
        let mut z = Complex::new(0.0, 0.0);
        let c = self.reference_point;
        
        // Pre-calculate transformation matrices for better numerical stability
        let scale = 1.0 / (self.x_max - self.x_min).abs().max((self.y_max - self.y_min).abs());
        let transform = Matrix2x2::new(
            scale, 0.0,
            0.0, scale
        );
        
        self.reference_orbit.reserve(self.max_iterations as usize);
        
        // Use scaled coordinates for better precision
        let scaled_c = transform.mul_complex(&c);
        
        for _ in 0..self.max_iterations {
            if z.magnitude_squared() > 4.0 {
                break;
            }
            self.reference_orbit.push(z);
            
            // Calculate with error tracking
            let r2 = z.real * z.real;
            let i2 = z.imag * z.imag;
            let ri = z.real * z.imag;
            
            // Track numerical errors
            let error = (r2.abs() + i2.abs()) * f64::EPSILON;
            
            z = Complex::with_error(
                r2 - i2 + scaled_c.real,
                2.0 * ri + scaled_c.imag,
                z.error + error
            );
            
            // If error gets too large, break early
            if z.error > 1e-6 {
                break;
            }
        }
    }

    fn iterate_standard(&self, c: Complex) -> u32 {
        let mut z = Complex::new(0.0, 0.0);
        let mut n = 0;

        while z.magnitude_squared() <= 4.0 && n < self.max_iterations as usize {
            let r2 = z.real * z.real;
            let i2 = z.imag * z.imag;
            z.imag = 2.0 * z.real * z.imag + c.imag;
            z.real = r2 - i2 + c.real;
            n += 1;
        }

        self.smooth_color(z, n)
    }

    fn smooth_color(&self, z: Complex, n: usize) -> u32 {
        if n < self.max_iterations as usize {
            n as u32 + 1 - (z.magnitude_squared().ln().ln() / 2.0_f64.ln()).floor() as u32
        } else {
            self.max_iterations
        }
    }
} 