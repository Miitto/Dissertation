#![feature(duration_constructors, duration_millis_float)]

use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct Benchmark {
    pub name: String,
    pub finished: Vec<Rc<RefCell<Benchmark>>>,
    pub active: Option<Rc<RefCell<Benchmark>>>,
    pub start: std::time::Instant,
    pub ended: bool,
    pub durations: Vec<Vec<std::time::Duration>>,
    pub draw_calls: Vec<usize>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ActiveBenchmark {
    bench: Option<Rc<RefCell<Benchmark>>>,
}

impl ActiveBenchmark {
    pub fn new(bench: Rc<RefCell<Benchmark>>) -> Self {
        Self { bench: Some(bench) }
    }

    pub fn dummy() -> Self {
        ActiveBenchmark { bench: None }
    }

    pub fn end(self) {
        if let Some(bench) = &self.bench {
            bench.borrow_mut().end();
        }
    }
}

impl Drop for ActiveBenchmark {
    fn drop(&mut self) {
        if let Some(bench) = &self.bench {
            if bench.borrow_mut().end() != 0 {
                eprintln!("Active benchmark dropped, ending {}", bench.borrow().name);
            }
        }
    }
}

impl Benchmark {
    /// Create a new benchmark, only use for the Root, otherwise use Benchmark::start
    pub fn new(name: &'static str) -> Self {
        Self {
            name: name.to_string(),
            start: std::time::Instant::now(),
            finished: Vec::new(),
            active: None,
            ended: true,
            durations: vec![vec![]],
            enabled: false,
            draw_calls: vec![],
        }
    }

    /// Enable this benchmark and all children. Does not restart the children, only sets the start
    /// time to now
    pub fn enable(&mut self) {
        self.enabled = true;
        self.ended = false;
        self.start = std::time::Instant::now();

        for finished in self.finished.iter_mut() {
            finished.borrow_mut().enable();
        }
    }

    /// Disable this benchmark and all children
    pub fn disable(&mut self) {
        self.enabled = false;
        self.ended = true;

        if let Some(active) = self.active.as_mut() {
            active.borrow_mut().disable();
        }

        for finished in self.finished.iter_mut() {
            finished.borrow_mut().disable();
        }

        self.end();
    }

    /// Restart this benchmark, does not cascade
    pub fn restart(&mut self) {
        if !self.enabled {
            return;
        }

        self.start = std::time::Instant::now();
        self.ended = false;

        // New set of draw calls for us
        // Not children to keep an accurate count for each nest
        self.draw_calls.push(0);

        // Start a new set of durations for children
        for finished in self.finished.iter_mut() {
            finished.borrow_mut().durations.push(vec![]);
        }
    }

    /// Create a new nested benchmark at the bottom of the tree
    #[must_use]
    pub fn start(&mut self, name: &'static str) -> ActiveBenchmark {
        if !self.enabled {
            // Pass a new dummy benchmark
            return ActiveBenchmark::dummy();
        }

        if self.move_active() {
            // We have a free active slot, create a new child for us
            // First check if we have a stashed benchmark with the same name
            let previous = self.finished.iter().find(|b| b.borrow().name == name);
            if let Some(previous) = previous {
                // Restart the previous benchmark and set as active
                previous.borrow_mut().restart();
                self.active = Some(previous.clone());
                ActiveBenchmark::new(previous.clone())
            } else {
                // Otherwise create a new one
                let mut benchmark = Benchmark::new(name);
                benchmark.enable();

                self.finished.push(Rc::new(RefCell::new(benchmark)));

                let last = self.finished.last().unwrap();
                self.active = Some(last.clone());
                ActiveBenchmark::new(last.clone())
            }
        } else {
            // We have an active benchmark, pass new children to it
            let active = self.active.as_ref().unwrap();
            let mut mutable = active.borrow_mut();
            mutable.start(name)
        }
    }

    /// Update the active benchmark if it has ended, return if the active benchmark is now empty
    fn move_active(&mut self) -> bool {
        if let Some(active) = self.active.as_ref() {
            if active.borrow().ended {
                self.active = None;
                return true;
            }
            return false;
        }
        true
    }

    /// Ends the benchmark, including children
    ///
    /// # Returns
    /// Number of benchmarks ended by this call
    pub fn end(&mut self) -> usize {
        let mut count = 0;
        if !self.ended {
            // End ourself, pushing the duration to the current set
            let end = std::time::Instant::now();
            self.durations
                .last_mut()
                .unwrap()
                .push(end.duration_since(self.start));

            self.ended = true;

            count += 1;
        }

        // If we have an active benchmark, end it aswell. Cascades down the tree
        if let Some(active) = self.active.take() {
            count += active.borrow_mut().end();
            self.move_active();
        }

        count
    }

    /// Log a draw call
    pub fn draw(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.borrow_mut().draw();
        }

        if let Some(draws) = self.draw_calls.last_mut() {
            *draws += 1;
        } else {
            self.draw_calls.push(1);
        }
    }

    fn get_depth(&self, depth: usize) -> usize {
        let mut max = depth;
        if let Some(active) = self.active.as_ref() {
            let active_depth = active.borrow().get_depth(depth + 1);
            if active_depth > max {
                max = active_depth;
            }
        }

        for finished in self.finished.iter() {
            let finished_depth = finished.borrow().get_depth(depth + 1);
            if finished_depth > max {
                max = finished_depth;
            }
        }

        max
    }

    fn get_max_name_len(&self, max_name_len: usize) -> usize {
        let mut max_name_len = max_name_len;

        if self.name.len() > max_name_len {
            max_name_len = self.name.len();
        }

        for finished in self.finished.iter() {
            max_name_len = finished.borrow().get_max_name_len(max_name_len);
        }

        if let Some(active) = self.active.as_ref() {
            max_name_len = active.borrow().get_max_name_len(max_name_len)
        }

        max_name_len
    }

    /// Print the benchmark tree
    pub fn print(&self) {
        if self.durations.iter().flatten().next().is_none() {
            return;
        }

        let max_name_len = self.get_max_name_len(0);
        let max_depth = self.get_depth(0);
        println!(
            "\n┌{}┐Min Max Avg Draws {}┐Min Single (fps) {}┐Max Single (fps) {}┐Avg Single (fps) {}┐Min Set (Count) {}┐Max Set (Count) {}┐Avg Set (Count) {}┐",
            "─".repeat(max_name_len + max_depth + 7),
            "─".repeat(3),
            "─".repeat(2),
            "─".repeat(2),
            "─".repeat(2),
            "─".repeat(3),
            "─".repeat(3),
            "─".repeat(3),
        );
        self.print_internal(0, max_depth, max_name_len, true, true);
        println!();
    }

    fn print_internal(
        &self,
        depth: usize,
        max_depth: usize,
        max_name_len: usize,
        is_last: bool,
        parent_last: bool,
    ) {
        if self.ended {
            if self.durations.iter().flatten().next().is_none() {
                return;
            }

            let mut min_draw_calls = usize::MAX;
            let mut max_draw_calls = 0;
            let mut total_draw_calls = 0;

            for draw_calls in self.draw_calls.iter() {
                if *draw_calls < min_draw_calls {
                    min_draw_calls = *draw_calls;
                }
                if *draw_calls > max_draw_calls {
                    max_draw_calls = *draw_calls;
                }
                total_draw_calls += *draw_calls;
            }

            let avg_draw_calls = total_draw_calls / self.draw_calls.len();

            let mut lowest = std::time::Duration::MAX;
            let mut highest = std::time::Duration::from_secs(0);
            let mut total = std::time::Duration::from_secs(0);

            for duration in self.durations.iter().flatten() {
                if *duration < lowest {
                    lowest = *duration;
                }
                if *duration > highest {
                    highest = *duration;
                }
                total += *duration;
            }

            let mut lowest_set_total = std::time::Duration::MAX;
            let mut min_set_count = 0;
            let mut highest_set_total = std::time::Duration::from_secs(0);
            let mut max_set_count = 0;
            let mut avg_set_total = std::time::Duration::from_secs(0);
            let mut avg_set_count_total = 0;

            for set in self.durations.iter() {
                let mut total = std::time::Duration::from_secs(0);
                for duration in set {
                    total += *duration;
                }

                if total < lowest_set_total {
                    min_set_count = set.len();
                    lowest_set_total = total;
                }
                if total > highest_set_total {
                    max_set_count = set.len();
                    highest_set_total = total;
                }
                avg_set_count_total += set.len();
                avg_set_total += total;
            }

            let avg_set_count = avg_set_count_total / self.durations.len();

            let set_avgerage = avg_set_total / self.durations.len() as u32;

            let set_min = lowest_set_total.as_millis_f32();
            let set_max = highest_set_total.as_millis_f32();
            let set_avg = set_avgerage.as_millis_f32();

            fn fmt_dur(dur: f32) -> String {
                if dur >= 1000. {
                    format!(" {:6.4}s", dur / 1000.)
                } else {
                    format!("{:6.2}ms", dur)
                }
            }

            fn fmt_fps(fps: f32) -> String {
                if fps >= 1000. {
                    ">10000".to_string()
                } else {
                    format!("{:6.2}", fps)
                }
            }

            let set_min = fmt_dur(set_min);
            let set_max = fmt_dur(set_max);
            let set_avg = fmt_dur(set_avg);

            let average = total / self.durations.iter().flatten().count() as u32;

            let min = lowest.as_millis_f32();
            let max = highest.as_millis_f32();
            let avg = average.as_millis_f32();

            let min_fps = 1000. / min;
            let max_fps = 1000. / max;
            let avg_fps = 1000. / avg;

            let min = fmt_dur(min);
            let max = fmt_dur(max);
            let avg = fmt_dur(avg);

            let min_fps = fmt_fps(min_fps);
            let max_fps = fmt_fps(max_fps);
            let avg_fps = fmt_fps(avg_fps);

            const SEGMENT_LENGTH: usize = 19;
            const DRAW_CALLS_LENGTH: usize = 21;

            // Set width to the biggest name length, then adjust for depth
            let align = max_name_len + (max_depth - depth) * 2;

            println!(
                "{}│ {:<align$} │{:6} {:6} {:6} │ {min} ({min_fps}) │ {max} ({max_fps}) │ {avg} ({avg_fps}) │ {set_min} ({:6}) │ {set_max} ({:6}) │ {set_avg} ({:6}) │",
                "│ ".repeat(depth),
                self.name,
                min_draw_calls,
                max_draw_calls,
                avg_draw_calls,
                min_set_count,
                max_set_count,
                avg_set_count,
            );

            let values = &self.finished;

            if !values.is_empty() {
                let draw_calls = "─".repeat(DRAW_CALLS_LENGTH);
                let repeat = "─".repeat(SEGMENT_LENGTH);
                let repeat = format!("┼{repeat}").repeat(6);
                println!(
                    "{}┌{}┼{draw_calls}{repeat}┤",
                    "│ ".repeat(depth + 1),
                    "─".repeat(align)
                );
            }

            for (idx, finished) in values.iter().enumerate() {
                finished.borrow().print_internal(
                    depth + 1,
                    max_depth,
                    max_name_len,
                    idx == values.len() - 1,
                    is_last,
                );
            }

            if is_last && (!parent_last || depth == 0) {
                let intersect = if depth == 0 { "┴" } else { "┼" };
                let repeat = "─".repeat(SEGMENT_LENGTH);
                let end = if depth == 0 { "┘" } else { "┤" };

                let last_depth = values
                    .last()
                    .map(|f| f.borrow().get_depth(depth))
                    .unwrap_or(depth);
                let depth_diff = last_depth - depth;

                let repeat = format!("{intersect}{repeat}").repeat(6);
                let draw_calls = "─".repeat(DRAW_CALLS_LENGTH);

                println!(
                    "{}└{}{}{intersect}{draw_calls}{repeat}{end}",
                    "│ ".repeat(depth),
                    "─┴".repeat(depth_diff),
                    "─".repeat((align - depth_diff * 2) + 2)
                );
            }
        } else {
            eprintln!("Benchmark {} not ended", self.name);
        }
    }
}
