use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub struct TSPInstance {
    num_cities: usize,
    distances: Vec<Vec<f64>>,
    time_windows: Vec<(f64, f64)>,
}

impl TSPInstance {
    pub fn new(num_cities: usize, distances: Vec<Vec<f64>>, time_windows: Vec<(f64, f64)>) -> Self {
        assert!(distances.len() == num_cities);
        for row in &distances {
            assert!(row.len() == num_cities);
        }
        assert!(time_windows.len() == num_cities);

        TSPInstance {
            num_cities,
            distances,
            time_windows,
        }
    }

    // First line of each file contains the number of cities.
    // The next num_cities lines contain the distance matrix, with each line containing num_cities floating-point numbers.
    // The next num_cities lines contain the time windows, with each line containing two floating-point numbers.
    pub fn from_file(path: PathBuf) -> Self {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read file {}", path.display()));

        assert!(!content.is_empty());

        let mut lines = content.lines();

        let num_cities_str = lines.next().unwrap();
        let num_cities: usize = num_cities_str
            .parse()
            .unwrap_or_else(|_| panic!("Could not convert {} to usize!", num_cities_str));

        let distance_lines_str: Vec<&str> = lines.by_ref().take(num_cities).collect();
        assert!(distance_lines_str.len() == num_cities);

        let mut distances: Vec<Vec<f64>> = Vec::with_capacity(num_cities);
        for line in distance_lines_str {
            let row_entries: Vec<f64> = line
                .split_whitespace()
                .map(|x| x.parse().unwrap())
                .collect();
            distances.push(row_entries);
        }

        let time_windows_str: Vec<&str> = lines.take(num_cities).collect();
        assert!(time_windows_str.len() == num_cities);

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
            num_cities,
            distances,
            time_windows,
        }
    }

    pub fn len(&self) -> usize {
        self.num_cities
    }

    pub fn dist_from_to(&self, from: usize, to: usize) -> f64 {
        assert!(from < self.num_cities && to < self.num_cities);
        self.distances[from][to]
    }

    pub fn window_of(&self, node: usize) -> (f64, f64) {
        assert!(node < self.num_cities);
        self.time_windows[node]
    }

    pub fn window_of_contains(&self, node: usize, time: f64) -> bool {
        assert!(node < self.num_cities);
        let (start, end) = self.window_of(node);
        (start..=end).contains(&time)
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
