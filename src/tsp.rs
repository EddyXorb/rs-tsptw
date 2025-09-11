use std::{fmt, path::PathBuf};

pub struct TSPInstance {
    num_cities: usize,
    distances: Vec<Vec<f64>>,
    time_windows: Vec<(f64, f64)>,
}

impl TSPInstance {
    fn new(num_cities: usize, distances: Vec<Vec<f64>>, time_windows: Vec<(f64, f64)>) -> Self {
        TSPInstance {
            num_cities,
            distances,
            time_windows,
        }
    }

    // First line of each file contains the number of cities.
    // The next num_cities lines contain the distance matrix, with each line containing num_cities floating-point numbers.
    // The next num_cities lines contain the time windows, with each line containing two floating-point numbers.
    pub fn create_from_file(path: PathBuf) -> Self {
        let content = std::fs::read_to_string(&path)
            .expect(&format!("Failed to read file {}", path.display()));

        assert!(content.len() > 0);

        let mut lines = content.lines();

        let num_cities_str = lines.next().unwrap();
        let num_cities: usize = num_cities_str
            .parse()
            .expect(&format!("Could not convert {} to usize!", num_cities_str));

        let distance_lines_str: Vec<&str> = lines.by_ref().take(num_cities).collect();
        assert!(distance_lines_str.len() == num_cities);

        let mut distances: Vec<Vec<f64>> = Vec::with_capacity(num_cities);
        for line in distance_lines_str {
            let row_entries: Vec<f64> = line.split(" ").map(|x| x.parse().unwrap()).collect();
            distances.push(row_entries);
        }

        let time_windows_str: Vec<&str> = lines.take(num_cities).collect();
        let time_windows: Vec<(f64, f64)> = time_windows_str
            .into_iter()
            .map(|line| {
                let split: Vec<f64> = line
                    .split_whitespace()
                    .map(|x| x.parse().unwrap())
                    .collect();
                (split[0], split[1])
            })
            .collect();
        
        TSPInstance {
            num_cities: num_cities,
            distances: distances,
            time_windows: time_windows,
        }
    }
}

impl fmt::Display for TSPInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cities: {}\nDistances:{:?}\nTime_windows:{:?}",
            self.num_cities, self.distances, self.time_windows
        )
    }
}