use std::{
    env,
    error::Error,
    fs,
    io::Write,
    process,
};


fn main() -> Result<(), Box<Error>> {
    let filename = match env::args().nth(1) {
        Some(name) => name,
        None => {
            eprintln!("Missing filename. Usage:");
            eprintln!("");
            eprintln!("   ./binary <filename>");
            process::exit(1);
        }
    };

    let data = fs::read_to_string(&filename)?
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| {
            let mut split = line.split(':');
            let wavelength = split.next().unwrap().trim().parse::<u32>().unwrap();
            let power = split.next().unwrap().trim().parse::<f64>().unwrap();

            // Calculate the energy of a single photon in eV. The factor 1240
            // can be derived from the formula E[J] = hc / λ[m] (where h is the
            // planck constant and c is the speed of light).
            let photon_energy = 1240.0 / (wavelength as f64);

            // The received energy per second per m² per nm wavelength in eV
            // (the factor is just the ratio between eV and J).
            let incoming_ev = 6.242e+18 * power;

            // Calculate the number of photons
            let num_photons = incoming_ev / photon_energy;

            DataPoint {
                wavelength,
                num_photons,
                photon_energy,
            }
        })
        .collect::<Vec<_>>();


    let total_sun_energy: f64 = data.iter()
        .map(|dp| dp.num_photons * dp.photon_energy)
        .sum();

    let mut max_threshold_idx = 0;
    let mut max_energy = 0.0;

    // Also write out some data
    let mut file = fs::File::create("efficiency.csv")?;

    for threshold_idx in 0..data.len() {
        // Calculate the total power we can generate. Only photons with
        // wavelengths below our threshold (thus, energies below our
        // threshold's photons' energies) contribute to the total power.

        let threshold_energy = data[threshold_idx].photon_energy;
        let total_photons: f64 = data[0..threshold_idx + 1].iter().map(|dp| dp.num_photons).sum();
        let sum = threshold_energy * total_photons;

        writeln!(
            file,
            "{}, {}, {}, {}",
            threshold_energy,
            data[threshold_idx].num_photons,
            total_photons,
            sum / total_sun_energy,
        )?;

        if sum > max_energy {
            max_energy = sum;
            max_threshold_idx = threshold_idx;
        }
    }

    // Print results
    println!("total sun power: {} W/m²", total_sun_energy / 6.242e+18);
    println!(
        "max energy @ {}nm ({:.2}eV) => {:.2} W/m² ({:.1}%)",
        data[max_threshold_idx].wavelength,
        data[max_threshold_idx].photon_energy,
        max_energy / 6.242e+18,
        (max_energy / total_sun_energy) * 100.0,
    );


    Ok(())
}

struct DataPoint {
    /// The wavelength in nm.
    wavelength: u32,

    /// The number of photons per second per square meter per nm wavelength.
    /// [1/s/m²/nm]
    num_photons: f64,

    /// The energy of a single photon in eV.
    photon_energy: f64,
}
